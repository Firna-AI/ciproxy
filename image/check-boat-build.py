#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Exercise native plan execution, resumption and non-failing inventory differences."""

import argparse
import contextlib
import importlib.util
import io
import json
import os
import pathlib
import subprocess
import tempfile
import time
import types
import unittest
from unittest.mock import patch

from _checks_.boat_cleanup import check_cleanup
from _checks_.apt_family import check_family
from _checks_.boat_inputs import check_inputs, check_bootstrap_chain, check_failed_step
from _checks_.apt_alias import check_alias
from _checks_.boat_sources import check_sources
from _checks_.boat_workdir import check_workdir
from _checks_.boat_setup import check_setup
from _checks_.release_tools import check_azcopy, check_bazel, check_docker, check_pypy, check_ruby
from _checks_.ansible_tools import check_ansible
from _checks_.boat_tmp import check_tmp
from _checks_.boat_owners import check_owners, check_adaptation, check_changed_directories, check_inventory
from _checks_.boat_developer import check_developer, check_developer_warnings, check_developer_inventory, check_developer_packages
from _checks_.boat_developer import fixture as developer_fixture, run_cleanup as run_developer_cleanup
from _checks_.github_lookup import check_lookup, check_warm_lookup
from _checks_.java_pins import check_java_request, check_pin_reports
from _checks_.pypy_resume import check_pypy_resume
from _checks_.boat_cli import check_prepare_cli
from _checks_.toolset_defaults import check_node_default
from _checks_.boat_permissions import check_permissions
from _checks_.boat_tar import check_tar
from _checks_.apt_epoch import check_epoch
from boat import Api
from boat_build_report import changed_services, compare_inventory, service_inventory


WORKSPACE = pathlib.Path(".context/image-checks/boat-build")
EXECUTOR = pathlib.Path(__file__).with_name("boat-build-execute.py").resolve()


class NativeBuild(unittest.TestCase):
    def test_rendered_apt_pin_keeps_the_repository_epoch(self):
        check_epoch(self, WORKSPACE)

    def test_native_tar_keeps_existing_folder_metadata_and_new_archive_metadata(self):
        check_tar(self, WORKSPACE)

    def test_native_permissions_make_the_target_writable_and_keep_provider_entries(self):
        check_permissions(self, WORKSPACE)

    def test_rendered_node_keeps_the_upstream_default_form_and_installs_the_exact_pin(self):
        check_node_default(self, WORKSPACE)

    def test_rendered_release_lookup_pages_exact_versions_and_keeps_pin_routes(self):
        check_lookup(self, WORKSPACE)

    def test_bundle_warms_cmake_release_pages_from_the_rendered_toolset(self):
        check_warm_lookup(self, WORKSPACE)

    def test_rendered_java_request_uses_the_apt_pin_with_a_newer_version_available(self):
        check_java_request(self, WORKSPACE)

    def test_apt_pin_differences_reach_boat_and_machine_reports_without_failure(self):
        check_pin_reports(self, WORKSPACE)

    def test_native_pypy_resumes_failed_pip_and_reuses_complete_versions(self):
        check_pypy_resume(self, WORKSPACE)

    def test_prepare_is_only_in_boat_cli_and_other_build_commands_stay(self):
        check_prepare_cli(self)

    def test_developer_package_purge_clears_installed_and_residual_dotnet_conffiles(self):
        sample = developer_fixture(WORKSPACE)
        paths = ("etc/dotnet/install_location", "etc/dotnet/install_location_x64", "etc/profile.d/dotnet.sh")
        for relative in paths:
            path = sample.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("/usr/lib/dotnet\n")
        state = json.loads(sample.package_state.read_text())
        state.update(installed={"dotnet-host-8.0": "8.0.11-0ubuntu1"},
                     residual={"dotnet-sdk-8.0": "8.0.110-0ubuntu1"},
                     conffiles={"dotnet-host-8.0": list(paths[:2]), "dotnet-sdk-8.0": [paths[2]]})
        sample.package_state.write_text(json.dumps(state))
        report = run_developer_cleanup(self, sample)
        self.assertEqual([relative for relative in paths if (sample.root / relative).exists()], [],
                         "A purge must clear the stale .NET conffiles.")
        dotnet = next(group for group in report["packages"] if group["pinned_tool"] == ".NET")
        self.assertEqual({package["name"] for package in dotnet["packages"]},
                         {"dotnet-host-8.0", "dotnet-sdk-8.0"})
        self.assertEqual(dotnet["action"], "purge")
        self.assertEqual(json.loads(sample.package_state.read_text())["residual"], {})

    def test_developer_packages_remove_only_safe_matches_and_record_absent_or_kept_groups(self):
        check_developer_packages(self, WORKSPACE)

    def test_bootstrap_removes_pin_shadows_and_keeps_provider_files_and_extra_tools(self):
        check_developer(self, WORKSPACE)

    def test_developer_cleanup_keeps_owned_profiles_and_warns_on_a_conffile_mismatch(self):
        check_developer_warnings(self, WORKSPACE)

    def test_inventory_reports_developer_cleanup_or_null_without_a_record(self):
        check_developer_inventory(self, WORKSPACE)

    def test_executor_records_owners_before_first_step_and_keeps_them_on_resume(self):
        check_owners(self, WORKSPACE)

    def test_native_owner_adaptation_preserves_other_tests_and_records_its_reason(self):
        check_adaptation(self, WORKSPACE)

    def test_native_owner_adaptation_rejects_a_changed_directory_list(self):
        check_changed_directories(self, WORKSPACE)

    def test_inventory_reports_recorded_owners_or_null_without_a_record(self):
        check_inventory(self, WORKSPACE)

    def test_native_archive_paths_use_owned_tmp_and_leave_runtime_configuration_unchanged(self):
        check_tmp(self, WORKSPACE)

    def test_azcopy_download_uses_the_pinned_release(self):
        check_azcopy(self, WORKSPACE)

    def test_bazel_uses_the_build_pin_and_keeps_project_runtime_selection(self):
        check_bazel(self, WORKSPACE)

    def test_ansible_bundle_keeps_the_pinned_core(self):
        check_ansible(self, WORKSPACE)

    def test_docker_cache_tags_the_pinned_digests_with_original_names(self):
        check_docker(self, WORKSPACE)

    def test_pypy_selects_the_reference_interpreter_and_python_pair(self):
        check_pypy(self, WORKSPACE)

    def test_ruby_uses_the_exact_release_tag_and_asset(self):
        check_ruby(self, WORKSPACE)

    def test_native_setup_repeats_and_merges_fresh_sources_without_losing_metadata(self):
        check_setup(self, WORKSPACE)

    def test_relative_installer_output_is_owned_and_finalization_removes_it(self):
        check_workdir(self, WORKSPACE)

    def test_native_sources_prepare_boat_and_generic_uris_for_the_shared_mirror_setup(self):
        check_sources(self, WORKSPACE)

    def test_resume_chains_immutable_inputs_without_replaying_earlier_steps(self):
        check_inputs(self, WORKSPACE)

    def test_bootstrap_input_change_replays_later_steps_and_unchanged_run_reuses_them(self):
        check_bootstrap_chain(self, WORKSPACE)

    def test_shared_apt_pin_selects_coherent_systemd_companions(self):
        check_family(self, WORKSPACE)

    def test_native_cleanup_keeps_cache_and_helper_cleanup_and_preserves_provider_data(self):
        check_cleanup(self, WORKSPACE)

    def test_shared_installer_selects_netcat_openbsd_and_keeps_the_netcat_cli(self):
        check_alias(self, WORKSPACE)

    def test_changed_active_service_is_recorded_with_docker_and_containerd(self):
        root = pathlib.Path(tempfile.mkdtemp(dir=WORKSPACE)).resolve()
        unit = root / "fixture.service"
        unit.write_text("[Service]\nExecStart=/bin/true\n")
        tool = root / "systemctl"
        tool.write_text("#!/bin/sh\ncase $1 in\n"
                        "list-units) printf 'fixture.service loaded active running fixture\\n"
                        "ciproxy-image-build.service loaded active running builder\\n' ;;\n"
                        f"show) printf 'MainPID={os.getpid()}\\nFragmentPath={unit}\\n' ;;\n"
                        "esac\n")
        tool.chmod(0o755)
        with patch.dict(os.environ, {"PATH": str(root) + os.pathsep + os.environ["PATH"]}):
            before = service_inventory()
        self.assertNotIn("ciproxy-image-build.service", before)
        before["ciproxy-image-build.service"] = dict(before["fixture.service"])
        self.assertEqual(changed_services(before), ["containerd.service", "docker.service"])
        unit.write_text("[Service]\nExecStart=/bin/false\n")
        self.assertEqual(changed_services(before), ["containerd.service", "docker.service", "fixture.service"])

    def test_host_report_retains_partial_artifact_errors_without_transferring_the_key(self):
        root = pathlib.Path(tempfile.mkdtemp(dir=WORKSPACE)).resolve()
        spec = importlib.util.spec_from_file_location("native_builder", EXECUTOR.with_name("boat-build.py"))
        builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(builder)
        bodies = []

        class Transport:
            def open(self, request, timeout):
                body = json.loads(request.data)
                bodies.append(body)
                command = body["command"]
                if command.endswith("/report.json"):
                    value = {"exitCode": 0, "stdout": '{"status":"failed","provisioners":[]}'}
                elif command.endswith("/software-report.json"):
                    value = {"exitCode": 1, "stderr": "not generated"}
                else:
                    value = {"exitCode": 0, "stdout": '{"partial":true}\n'}
                return io.BytesIO(json.dumps(value).encode())

        with patch.dict(os.environ, {"BOAT_API_KEY": "fixture-host-key"}):
            api = Api()
            api.opener = Transport()
            args = types.SimpleNamespace(action="report", sandbox="fixture", log=None, output=root)
            with contextlib.redirect_stdout(io.StringIO()):
                report = builder.remote(api, args)
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["unavailable_artifacts"], ["software-report.json"])
        self.assertEqual(json.loads((root / "report.json").read_text()), report)
        self.assertEqual(json.loads((root / "installed-inventory.json").read_text()), {"partial": True})
        self.assertEqual(len(bodies), 6)
        self.assertNotIn("fixture-host-key", json.dumps(bodies))

    def test_failed_step_resumes_without_repeating_unchanged_passed_steps(self):
        check_failed_step(self, WORKSPACE)

    def test_inventory_difference_reports_versions_and_tables_without_a_failure(self):
        reference = {"Title": "Image", "Children": [
            {"ToolName": "Docker", "Version": "28.0.4"},
            {"Title": "Java", "Children": {"Rows": ["17.0.21+9|JAVA_HOME"]}}]}
        installed = {"Title": "Image", "Children": [
            {"ToolName": "Docker", "Version": "28.0.4"},
            {"Title": "Java", "Children": {"Rows": ["17.0.22+1|JAVA_HOME"]}}]}
        self.assertEqual(compare_inventory(reference, installed), {
            "differences_fail_build": False, "difference_count": 1, "differences": [
                {"tool": "/Image/Java/", "reference": ["17.0.21+9|JAVA_HOME"],
                 "installed": ["17.0.22+1|JAVA_HOME"]}]})


class TimedResult(unittest.TextTestResult):
    def startTest(self, test):
        self.started = time.monotonic()
        super().startTest(test)

    def stopTest(self, test):
        self.stream.writeln(f"Duration {test.id()}: {time.monotonic() - self.started:.3f}s")
        super().stopTest(test)


def main():
    global WORKSPACE
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path, default=WORKSPACE)
    args = parser.parse_args()
    WORKSPACE = args.work_dir
    WORKSPACE.mkdir(parents=True, exist_ok=True)
    result = unittest.TextTestRunner(verbosity=2, resultclass=TimedResult).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeBuild))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
