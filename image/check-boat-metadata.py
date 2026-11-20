#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check native Boat metadata comparison on real file privilege metadata."""

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
IMAGE = pathlib.Path(__file__).resolve().parent


class LocalBoat:
    def __init__(self, rootfs, evidence):
        self.rootfs = rootfs
        self.evidence = evidence

    def command(self, sandbox, command):
        script = shlex.quote(str(self.rootfs / "opt/ciproxy-image/metadata-smoke.py"))
        local = command.replace("/opt/ciproxy-image/metadata-smoke.py / ",
                                script + " " + shlex.quote(str(self.rootfs)) + " ")
        result = subprocess.run(["bash", "-c", local], capture_output=True, text=True)
        (self.evidence / "boat-metadata-command.log").write_text(
            command + "\n" + result.stdout + result.stderr)
        return {"exitCode": result.returncode, "stdout": result.stdout, "stderr": result.stderr}


class NativeMetadata(unittest.TestCase):
    workspace = pathlib.Path(".context/image-checks/boat-metadata")

    def setUp(self):
        self.started = time.perf_counter()
        self.assertEqual(os.geteuid(), 0, "Run this fixture with sudo for real file capabilities")
        self.workspace.mkdir(parents=True, exist_ok=True)
        self.evidence = pathlib.Path(tempfile.mkdtemp(prefix="metadata-", dir=self.workspace)).resolve()
        self.rootfs = self.evidence / "rootfs"
        for directory in ["usr/bin", "opt/ciproxy-image"]:
            (self.rootfs / directory).mkdir(parents=True)
        path = self.rootfs / "usr/bin/ping"
        path.write_bytes(b"fixture tool\n")
        path.chmod(0o755)
        os.chown(path, 1234, 2345)
        os.setxattr(path, "security.capability", CAPABILITY)
        shutil.copyfile(IMAGE / "metadata-smoke.py",
                        self.rootfs / "opt/ciproxy-image/metadata-smoke.py")
        print("Fixture evidence: " + str(self.evidence), flush=True)

    def tearDown(self):
        print(f"{self.id()}: {time.perf_counter() - self.started:.3f}s", flush=True)

    def test_native_boat_keeps_metadata_comparison_without_restoring_capabilities(self):
        ping = self.rootfs / "usr/bin/ping"
        subprocess.run(["setfacl", "-m", "u:34567:r-x", str(ping)], check=True)
        baseline = subprocess.run([sys.executable, str(IMAGE / "metadata-smoke.py"),
                                   str(self.rootfs), "usr/bin/ping"],
                                  check=True, capture_output=True, text=True)
        baseline_file = self.evidence / "baseline.json"
        baseline_file.write_text(baseline.stdout)
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
        ping.chmod(0o700)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            boat.metadata(api, args)
        changed = json.loads(output.getvalue())
        self.assertFalse(changed["matches_baseline"])
        self.assertEqual(changed["differences"]["mode"], {"before": 0o755, "after": 0o700})
        (self.evidence / "comparison.json").write_text(json.dumps({
            "native": observed, "changed_mode": changed}, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path, default=NativeMetadata.workspace)
    args = parser.parse_args()
    NativeMetadata.workspace = args.work_dir
    started = time.perf_counter()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(NativeMetadata))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    raise SystemExit(0 if result.wasSuccessful() else 1)
