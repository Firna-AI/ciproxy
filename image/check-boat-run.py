#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check that Boat restores release privileges before Docker or the runner."""

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

from _checks_.export_fixture import provision_sandbox


class BoatRun(unittest.TestCase):
    workspace = pathlib.Path(".context/live-run-2026-11-16/image")

    def setUp(self):
        self.started = time.perf_counter()

    def tearDown(self):
        print(f"{self.id()}: {time.perf_counter() - self.started:.3f}s", flush=True)

    def run_boat(self, fail_restore=False):
        self.assertEqual(os.geteuid(), 0, "Run this fixture with sudo for chroot")
        self.workspace.mkdir(parents=True, exist_ok=True)
        root = pathlib.Path(tempfile.mkdtemp(prefix="boat-run-", dir=self.workspace)).resolve()
        source = root / "root"
        image = pathlib.Path(__file__).resolve().parent
        provision_sandbox(source, image)
        for name in ["state", "tmp", "opt/ciproxy/rootfs/etc/ciproxy"]:
            (source / name).mkdir(parents=True)
        for name in ["capabilities-ready", "dockerd-started", "guest-started"]:
            (source / "state" / name).write_text("0")
        os.mkfifo(source / "state/docker-ready")
        (source / "opt/ciproxy/rootfs/etc/ciproxy/image-id").write_text("fixture\n")
        (source / "registration.json").write_text("{}\n")
        shutil.copyfile(image / "boat-run.sh", source / "boat-run.sh")
        stub = source / "usr/bin/boat-tool"
        stub.write_text('''#!/usr/bin/bash
tool=${0##*/}
printf '%s %s\\n' "$tool" "$*" >> /state/events
case "$tool" in
    python3)
        if [[ $1 == /opt/ciproxy/rootfs/opt/ciproxy-image/file-capabilities.py && $2 == restore ]]; then
            [[ $FAIL_RESTORE != 1 ]] || exit 23
            printf 1 > /state/capabilities-ready
        fi ;;
    chroot)
        case $2 in
            /usr/bin/dockerd)
                if [[ $(< /state/capabilities-ready) == 1 ]]; then
                    printf 1 > /state/dockerd-started
                fi
                printf 'ready\\n' > /state/docker-ready ;;
            /usr/bin/docker) read -r readiness < /state/docker-ready ;;
            /usr/local/bin/ciproxy-guest)
                [[ $(< /state/capabilities-ready) == 1 ]] || exit 27
                printf 1 > /state/guest-started ;;
        esac ;;
    cat) while IFS= read -r line; do printf '%s\\n' "$line"; done < "$1" ;;
esac
''')
        stub.chmod(0o755)
        for name in ["python3", "chroot", "cat", "mount", "install", "rm", "cp"]:
            destination = source / "usr/bin" / name
            destination.unlink(missing_ok=True)
            destination.symlink_to(stub.name)
        result = subprocess.run([shutil.which("chroot"), str(source), "/usr/bin/bash",
                                 "/boat-run.sh", "/registration.json"], capture_output=True, text=True,
                                env={**os.environ, "PATH": "/usr/bin", "LC_ALL": "C",
                                     "CIPROXY_PRIVATE_NAMESPACE": "1",
                                     "FAIL_RESTORE": "1" if fail_restore else "0"})
        (root / "command.log").write_text(result.stdout + result.stderr)
        print("Fixture evidence: " + str(root), flush=True)
        return source, result

    def test_boat_restores_capabilities_before_docker_and_guest_start(self):
        root, result = self.run_boat()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((root / "state/dockerd-started").read_text(), "1")
        self.assertEqual((root / "state/guest-started").read_text(), "1")

    def test_failed_restore_stops_before_docker_and_guest_start(self):
        root, result = self.run_boat(fail_restore=True)
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual((root / "state/dockerd-started").read_text(), "0")
        self.assertEqual((root / "state/guest-started").read_text(), "0")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path, default=BoatRun.workspace)
    args = parser.parse_args()
    BoatRun.workspace = args.work_dir
    started = time.perf_counter()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(BoatRun))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    raise SystemExit(0 if result.wasSuccessful() else 1)
