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
from _checks_.boat_inputs import check_inputs
from _checks_.apt_alias import check_alias
from _checks_.boat_sources import check_sources
from _checks_.boat_workdir import check_workdir
from _checks_.boat_setup import check_setup
from _checks_.release_tools import check_azcopy, check_bazel, check_docker, check_pypy, check_ruby
from _checks_.ansible_tools import check_ansible
from _checks_.boat_tmp import check_tmp
from _checks_.boat_owners import check_owners, check_adaptation, check_changed_directories, check_inventory
from _checks_.boat_developer import check_developer, check_developer_warnings, check_developer_inventory
from boat import Api
from boat_build_plan import make_plan
from boat_build_report import changed_services, compare_inventory, service_inventory


WORKSPACE = pathlib.Path(".context/image-checks/boat-build")
EXECUTOR = pathlib.Path(__file__).with_name("boat-build-execute.py").resolve()


class NativeBuild(unittest.TestCase):
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

    def test_resume_uses_immutable_inputs_and_preserves_unrelated_passed_steps(self):
        check_inputs(self, WORKSPACE)

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
        root = pathlib.Path(tempfile.mkdtemp(dir=WORKSPACE)).resolve()
        upstream = root / "runner-images"
        templates = upstream / "images/ubuntu/templates"
        scripts = upstream / "images/ubuntu/scripts/build"
        helpers = upstream / "images/ubuntu/scripts/helpers"
        for folder in (templates, scripts, helpers):
            folder.mkdir(parents=True, exist_ok=True)
        trace, flag = root / "trace", root / "blocked"
        flag.touch()
        (helpers / "helper.txt").write_text("helper data\n")
        (scripts / "toolset.json").write_text('{"fixture": true}\n')
        (scripts / "first.sh").write_text('printf "first\\n" >> "$TRACE"\n')
        (scripts / "second.sh").write_text('printf "second\\n" >> "$TRACE"\n'
                                           'test ! -e "$BLOCKED" || exit 17\n')
        recipe = '''build {
  provisioner "file" {
    source = "${path.root}/../scripts/helpers"
    destination = "COPIED"
  }
  provisioner "file" {
    source = "${path.root}/../scripts/build/toolset.json"
    destination = "TOOLSET"
  }
  provisioner "shell" {
    environment_vars = ["TRACE=TRACEFILE", "BLOCKED=FLAGFILE"]
    scripts = ["${path.root}/../scripts/build/first.sh", "${path.root}/../scripts/build/second.sh"]
  }
  provisioner "shell" {
    inline = ["printf 'last\\n' >> TRACEFILE"]
  }
}
'''.replace("COPIED", str(root / "copied")).replace("TOOLSET", str(root / "copied.json"))
        recipe = recipe.replace("TRACEFILE", str(trace)).replace("FLAGFILE", str(flag))
        (templates / "build.ubuntu-24_04.pkr.hcl").write_text(recipe)
        plan = make_plan(upstream)
        artifact = root / "artifact"
        artifact.write_text("first input\n")
        plan["steps"][-1]["inputs"] = [str(artifact)]
        (root / "plan.json").write_text(json.dumps(plan))

        def run(resume=False):
            command = ["python3", str(EXECUTOR), str(root), "--no-inventory"]
            observed = subprocess.run(command + (["--resume"] if resume else []),
                                      capture_output=True, text=True)
            return observed, json.loads((root / "report.json").read_text())

        initial, report = run()
        self.assertEqual(initial.returncode, 1)
        self.assertEqual([step["status"] for step in report["provisioners"]],
                         ["passed", "passed", "passed", "failed", "pending"])
        self.assertEqual(report["provisioners"][3]["exit_code"], 17)
        self.assertEqual((root / "copied/helper.txt").read_text(), "helper data\n")
        self.assertEqual(json.loads((root / "copied.json").read_text()), {"fixture": True})
        self.assertEqual(trace.read_text(), "first\nsecond\n")
        flag.unlink()
        resumed, report = run(True)
        self.assertEqual(resumed.returncode, 0, resumed.stderr)
        self.assertEqual(report["status"], "passed")
        self.assertEqual(trace.read_text(), "first\nsecond\nsecond\nlast\n")
        (scripts / "first.sh").write_text('printf "changed\\n" >> "$TRACE"\n')
        changed, report = run(True)
        self.assertEqual(changed.returncode, 0, changed.stderr)
        self.assertEqual(trace.read_text(), "first\nsecond\nsecond\nlast\nchanged\n")
        artifact.write_text("new input\n")
        refreshed, report = run(True)
        self.assertEqual(refreshed.returncode, 0, refreshed.stderr)
        self.assertEqual(trace.read_text(), "first\nsecond\nsecond\nlast\nchanged\nlast\n")

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
