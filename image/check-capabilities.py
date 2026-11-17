#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check trusted release capabilities and the real Boat metadata comparison."""

import argparse
import contextlib
import importlib.util
import io
import json
import os
import pathlib
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import types
import unittest

sys.dont_write_bytecode = True

CAPABILITY = bytes.fromhex("0100000200200000000000000000000000000000")
DIGEST = "7190561c52979663f3d0fcfb01186b6f67e3e82c5f9614ec1d488fda53218eaa"
IMAGE = pathlib.Path(__file__).resolve().parent


class LocalBoat:
    def __init__(self, rootfs, evidence):
        self.rootfs = rootfs
        self.evidence = evidence

    def command(self, sandbox, command):
        local = command.replace("/opt/ciproxy/rootfs", shlex.quote(str(self.rootfs)))
        result = subprocess.run(["bash", "-c", local], capture_output=True, text=True)
        (self.evidence / "boat-metadata-command.log").write_text(
            command + "\n" + result.stdout + result.stderr)
        return {"exitCode": result.returncode, "stdout": result.stdout, "stderr": result.stderr}


class FileCapabilities(unittest.TestCase):
    workspace = pathlib.Path(".context/live-run-2026-11-16/image")

    def setUp(self):
        self.started = time.perf_counter()
        self.assertEqual(os.geteuid(), 0, "Run this fixture with sudo for real file capabilities")
        self.workspace.mkdir(parents=True, exist_ok=True)
        self.evidence = pathlib.Path(tempfile.mkdtemp(prefix="capabilities-", dir=self.workspace)).resolve()
        self.rootfs = self.evidence / "rootfs"
        for directory in ["usr/bin", "opt/nested", "opt/ciproxy-image", "etc/ciproxy"]:
            (self.rootfs / directory).mkdir(parents=True)
        self.paths = ["opt/nested/ping", "usr/bin/ping", "usr/bin/ping-alias"]
        for relative in self.paths[:2]:
            path = self.rootfs / relative
            path.write_bytes(b"fixture tool\n")
            path.chmod(0o755)
            os.chown(path, 1234, 2345)
            os.setxattr(path, "security.capability", CAPABILITY)
        os.link(self.rootfs / "usr/bin/ping", self.rootfs / "usr/bin/ping-alias")
        outside = self.evidence / "outside"
        outside.mkdir()
        (outside / "tool").write_bytes(b"outside fixture\n")
        os.setxattr(outside / "tool", "security.capability", CAPABILITY)
        (self.rootfs / "outside").symlink_to(outside)
        (self.rootfs / "usr/bin/ping-link").symlink_to("ping")
        for name in ["file-capabilities.py", "metadata-smoke.py"]:
            shutil.copyfile(IMAGE / name, self.rootfs / "opt/ciproxy-image" / name)
        print("Fixture evidence: " + str(self.evidence), flush=True)

    def tearDown(self):
        print(f"{self.id()}: {time.perf_counter() - self.started:.3f}s", flush=True)

    def command(self, operation):
        return subprocess.run([sys.executable, str(IMAGE / "file-capabilities.py"),
                               operation, str(self.rootfs)], capture_output=True, text=True)

    def capture(self):
        result = self.command("capture")
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads((self.rootfs / "etc/ciproxy/file-capabilities.json").read_text())

    def test_capture_records_complete_tree_without_following_links_or_mounts(self):
        mounted = self.rootfs / "other-filesystem"
        mounted.mkdir()
        subprocess.run(["mount", "-t", "tmpfs", "tmpfs", str(mounted)], check=True)
        try:
            (mounted / "tool").write_bytes(b"mounted fixture\n")
            os.setxattr(mounted / "tool", "security.capability", CAPABILITY)
            metadata = self.capture()
        finally:
            subprocess.run(["umount", str(mounted)], check=True)
        self.assertEqual(metadata, {"version": 1, "files": [
            {"path": relative, "sha256": DIGEST, "security_capability": CAPABILITY.hex()}
            for relative in self.paths]})

    def test_boat_restores_real_capabilities_and_keeps_metadata_comparison(self):
        self.capture()
        ping = self.rootfs / "usr/bin/ping"
        subprocess.run(["setfacl", "-m", "u:34567:r-x", str(ping)], check=True)
        baseline = subprocess.run([sys.executable, str(IMAGE / "metadata-smoke.py"),
                                   str(self.rootfs), "usr/bin/ping"],
                                  check=True, capture_output=True, text=True)
        baseline_file = self.evidence / "baseline.json"
        baseline_file.write_text(baseline.stdout)
        os.removexattr(ping, "security.capability")
        os.removexattr(self.rootfs / "opt/nested/ping", "security.capability")
        spec = importlib.util.spec_from_file_location("boat", IMAGE / "boat.py")
        boat = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(boat)
        args = types.SimpleNamespace(sandbox="fixture", file="usr/bin/ping", baseline=baseline_file)
        api = LocalBoat(self.rootfs, self.evidence)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            boat.metadata(api, args)
        observed = json.loads(output.getvalue())
        self.assertTrue(observed["matches_baseline"], observed)
        self.assertEqual(observed["differences"], {})
        self.assertEqual(observed["metadata"], json.loads(baseline.stdout))
        for relative in self.paths:
            self.assertEqual(os.getxattr(self.rootfs / relative, "security.capability"), CAPABILITY)
        ping.chmod(0o700)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            boat.metadata(api, args)
        changed = json.loads(output.getvalue())
        self.assertFalse(changed["matches_baseline"])
        self.assertEqual(changed["differences"]["mode"], {"before": 0o755, "after": 0o700})
        (self.evidence / "comparison.json").write_text(json.dumps({
            "restored": observed, "changed_mode": changed}, indent=2) + "\n")

    def test_restore_grants_capabilities_only_to_unchanged_release_files(self):
        self.capture()
        path = self.rootfs / "opt/nested/ping"
        path.write_bytes(b"changed after release\n")
        result = self.command("restore")
        self.assertNotEqual(result.returncode, 0, "A changed release file must not receive a capability")
        self.assertNotIn("security.capability", os.listxattr(path))
        (self.evidence / "changed-file.log").write_text(result.stdout + result.stderr)

    def test_restore_does_not_follow_replaced_directory_outside_release(self):
        self.capture()
        outside = self.evidence / "outside"
        path = outside / "ping"
        path.write_bytes(b"fixture tool\n")
        (self.rootfs / "opt/nested").rename(self.rootfs / "retained")
        (self.rootfs / "opt/nested").symlink_to(outside)
        result = self.command("restore")
        self.assertNotEqual(result.returncode, 0, "A changed directory must not grant privileges outside the release")
        self.assertNotIn("security.capability", os.listxattr(path))
        (self.evidence / "replaced-directory.log").write_text(result.stdout + result.stderr)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path, default=FileCapabilities.workspace)
    args = parser.parse_args()
    FileCapabilities.workspace = args.work_dir
    started = time.perf_counter()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(FileCapabilities))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    raise SystemExit(0 if result.wasSuccessful() else 1)
