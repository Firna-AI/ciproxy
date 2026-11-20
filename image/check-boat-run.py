#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check native Boat daemon refresh and private guest registration delivery."""

import argparse
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

sys.dont_write_bytecode = True

from _checks_.boat_fixture import prepare_root


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

    def launch(self, registration=None):
        command = [shutil.which("chroot"), str(self.root), "/usr/bin/bash", "/image/boat-run.sh"]
        if registration:
            command.append(registration)
        result = subprocess.run(command, capture_output=True, text=True,
                                env={"PATH": "/usr/bin:/bin", "LC_ALL": "C", "TZ": "UTC"})
        (self.root / "state/launch.log").write_text(result.stdout + result.stderr)
        events = (self.root / "state/events").read_text().splitlines()
        return result, events

    def assert_refresh(self, events):
        self.assertEqual(events, [
            "daemon-reload", "start --no-block ciproxy-boat-start.service", "daemon-reload",
            "stop docker.service docker.socket", "restart containerd.service",
            "try-restart example.service", "try-restart user@1000.service",
            "start docker.service docker.socket",
            "start ciproxy-boat-guest.service",
        ])

    def test_native_start_refreshes_daemons_then_delivers_private_input(self):
        source = self.root / "private-registration.json"
        source.write_text("fixture-private-registration\n")
        source.chmod(0o600)
        result, events = self.launch("/private-registration.json")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_refresh(events)
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


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path, default=BoatRun.workspace)
    parser.add_argument("--guest", type=pathlib.Path, default=BoatRun.guest)
    args = parser.parse_args()
    BoatRun.workspace = args.work_dir
    BoatRun.guest = args.guest.resolve()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(BoatRun))
    raise SystemExit(0 if result.wasSuccessful() else 1)
