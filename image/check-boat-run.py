#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check native Boat daemon refresh and private guest registration delivery."""

import argparse
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

sys.dont_write_bytecode = True

from _checks_.boat_fixture import MEASURE_PATH, prepare_root


class BoatRun(unittest.TestCase):
    workspace = pathlib.Path(".context/image-checks/boat-run")
    guest = pathlib.Path("target/debug/ciproxy-guest")

    def setUp(self):
        self.started = time.perf_counter()
        self.assertEqual(os.geteuid(), 0, "Run this fixture with sudo for isolation")
        self.workspace.mkdir(parents=True, exist_ok=True)
        self.root = pathlib.Path(tempfile.mkdtemp(prefix="boat-native-", dir=self.workspace)).resolve()
        prepare_root(self.root, self.guest)

    def tearDown(self):
        print(f"{self.id()}: {time.perf_counter() - self.started:.3f}s", flush=True)
        print("Fixture evidence: " + str(self.root), flush=True)

    def run_in_root(self, *command, **environment):
        return subprocess.run([shutil.which("chroot"), str(self.root), *command], capture_output=True, text=True,
                              env={"PATH": "/usr/bin:/bin", "LC_ALL": "C", "TZ": "UTC",
                                   "PYTHONDONTWRITEBYTECODE": "1", **environment})

    def launch(self, registration=None, **environment):
        command = ["/usr/bin/bash", "/image/boat-run.sh"]
        if registration:
            command.append(registration)
        result = self.run_in_root(*command, **environment)
        (self.root / "state/launch.log").write_text(result.stdout + result.stderr)
        events = (self.root / "state/events").read_text().splitlines()
        return result, events

    def assert_refresh(self, events):
        self.assertEqual(events, [
            "daemon-reload", "start --no-block ciproxy-boat-start.service", "daemon-reload",
            "stop docker.service docker.socket", "restart containerd.service",
            "try-restart example.service", "try-restart user@1000.service",
            "start docker.service docker.socket",
            "start ciproxy-swap.service",
            "start ciproxy-boat-guest.service",
        ])

    def test_native_start_refreshes_daemons_then_delivers_private_input(self):
        source = self.root / "private-registration.json"
        source.write_text("fixture-private-registration\n")
        source.chmod(0o600)
        (self.root / "state/sitecustomize.py").write_text('''import os
from pathlib import Path
from types import SimpleNamespace

def fixed_statvfs(path):
    Path("/state/statvfs-path").write_text(str(path))
    return SimpleNamespace(
        f_bsize=8192, f_frsize=4096, f_blocks=1000, f_bavail=200, f_bfree=300,
    )

os.statvfs = fixed_statvfs
''')
        result, events = self.launch("/private-registration.json", PYTHONPATH="/state")
        self.assertFalse((self.root / "state/du-calls").exists(),
                         "Boat job measurement must not walk the live root.")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_refresh(events)
        self.assertEqual((self.root / "state/statvfs-path").read_text(), "/")
        self.assertEqual(json.loads(result.stdout), {
            "backing_filesystem_bytes": 4096000,
            "backing_filesystem_available_bytes": 819200,
            "free_job_bytes": 819200,
        })
        delivered = self.root / "state/delivered-registration.json"
        self.assertEqual(delivered.read_text(), "fixture-private-registration\n")
        self.assertEqual(delivered.stat().st_mode & 0o777, 0o600)
        self.assertEqual(delivered.stat().st_uid, 0)
        self.assertFalse(source.exists())
        self.assertNotIn("fixture-private-registration", result.stdout + result.stderr)

    def test_missing_registration_reaches_native_guest_after_refresh(self):
        result, events = self.launch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_refresh(events)
        self.assertEqual((self.root / "state/guest-exit").read_text(), "1")
        self.assertIn("[ciproxy-guest] registration file is missing",
                      (self.root / "state/guest.log").read_text())
        self.assertFalse((self.root / "run/ciproxy/registration.json").exists())

    def test_failed_job_measurement_warns_and_starts_guest(self):
        (self.root / "state/sitecustomize.py").write_text('''import os

def unavailable(path):
    raise OSError("fixture statvfs failure")

os.statvfs = unavailable
''')
        result, events = self.launch(PYTHONPATH="/state")
        self.assertIn("fixture statvfs failure", result.stderr)
        self.assert_refresh(events)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Warning: Boat job disk measurement failed.", result.stderr)

    def test_failed_swap_service_warns_and_starts_guest(self):
        result, events = self.launch(CIPROXY_FIXTURE_FAIL_SWAP="1")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Warning: Boat job swap setup failed.", result.stderr)
        self.assert_refresh(events)

    def test_non_job_measurement_propagates_failed_du(self):
        result = self.run_in_root("/usr/bin/python3", MEASURE_PATH, "/")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertTrue((self.root / "state/du-calls").is_file())
        self.assertIn("fixture du failure", result.stderr)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path, default=BoatRun.workspace)
    parser.add_argument("--guest", type=pathlib.Path, default=BoatRun.guest)
    args = parser.parse_args()
    BoatRun.workspace = args.work_dir
    BoatRun.guest = args.guest.resolve()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(BoatRun))
    raise SystemExit(0 if result.wasSuccessful() else 1)
