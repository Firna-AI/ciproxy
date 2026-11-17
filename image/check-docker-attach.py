#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check sticky Docker mounts and the root fallback without mounting a disk."""

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


class DockerAttach(unittest.TestCase):
    workspace = pathlib.Path(".context/live-run-2026-11-16/image")

    def setUp(self):
        self.started = time.perf_counter()

    def tearDown(self):
        print(f"{self.id()}: {time.perf_counter() - self.started:.3f}s", flush=True)

    def run_attach(self, failure=""):
        self.assertEqual(os.geteuid(), 0, "Run this fixture with sudo for chroot")
        self.workspace.mkdir(parents=True, exist_ok=True)
        root = pathlib.Path(tempfile.mkdtemp(prefix="docker-attach-", dir=self.workspace)).resolve()
        source = root / "root"
        image = pathlib.Path(__file__).resolve().parent
        provision_sandbox(source, image)
        for name in ["var/lib/docker", "mnt/ciproxy-docker", "state"]:
            (source / name).mkdir(parents=True)
        (source / "proc").mkdir()
        (source / "dev/fd").symlink_to("/proc/self/fd")
        (source / "var/lib/docker/root-data").write_text("original Docker data\n")
        for name in ["docker-mounted", "preparation-mounted", "daemon", "failed"]:
            (source / "state" / name).write_text("0")
        shutil.copyfile(image / "docker-attach.sh", source / "attach.sh")
        stub = source / "usr/bin/attach-tool"
        stub.write_text('''#!/usr/bin/bash
tool=${0##*/}
printf '%s %s\\n' "$tool" "$*" >> /state/events
if [[ "$tool $*" == "$ATTACH_FAILURE" && $(< /state/failed) == 0 ]]; then
    printf 1 > /state/failed
    exit 17
fi
case "$tool" in
    mountpoint) [[ $(< /state/docker-mounted) == 1 ]] ;;
    mount)
        if [[ $1 == --move ]]; then
            printf 'A shared mount cannot move.\\n' >&2
            exit 32
        fi
        case $2 in
            /mnt/ciproxy-docker) printf 1 > /state/preparation-mounted ;;
            /var/lib/docker) printf 1 > /state/docker-mounted ;;
        esac ;;
    umount)
        case $1 in
            /mnt/ciproxy-docker) printf 0 > /state/preparation-mounted ;;
            /var/lib/docker) printf 0 > /state/docker-mounted ;;
        esac ;;
    systemctl)
        case $1 in
            stop) printf 0 > /state/daemon ;;
            start) printf 1 > /state/daemon ;;
            is-active) [[ $(< /state/daemon) == 1 ]] ;;
        esac ;;
    cp) printf '%s\\n' "$(< /var/lib/docker/root-data)" > /mnt/ciproxy-docker/root-data ;;
    touch) : > "$1" ;;
    stat) printf '100 4096\\n' ;;
esac
''')
        stub.chmod(0o755)
        for name in ["mountpoint", "mount", "umount", "systemctl", "cp", "touch",
                     "stat", "mkdir", "blockdev", "sync"]:
            destination = source / "usr/bin" / name
            destination.unlink(missing_ok=True)
            destination.symlink_to(stub.name)
        subprocess.run(["mount", "-t", "proc", "proc", str(source / "proc")], check=True)
        try:
            result = subprocess.run([shutil.which("chroot"), str(source), "/usr/bin/bash", "/attach.sh", "1024"],
                                    capture_output=True, text=True,
                                    env={**os.environ, "LC_ALL": "C", "PATH": "/usr/bin",
                                         "ATTACH_FAILURE": failure})
        finally:
            subprocess.run(["umount", str(source / "proc")], check=True)
        (root / "command.log").write_text(result.stdout + result.stderr)
        print("Fixture evidence: " + str(root), flush=True)
        return source, result

    def test_shared_mount_attaches_device_at_docker(self):
        root, result = self.run_attach()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((root / "state/docker-mounted").read_text(), "1")
        self.assertEqual((root / "state/preparation-mounted").read_text(), "0")
        self.assertEqual((root / "state/daemon").read_text(), "1")
        self.assertEqual((root / "mnt/ciproxy-docker/root-data").read_text(), "original Docker data\n")
        events = (root / "state/events").read_text().splitlines()
        self.assertLess(events.index("umount /mnt/ciproxy-docker"),
                        events.index("mount /dev/vdb /var/lib/docker"))

    def test_attachment_failure_restarts_docker_on_root(self):
        for failure in ["systemctl stop docker.socket docker.service",
                        "mount /dev/vdb /mnt/ciproxy-docker",
                        "cp -a --reflink=auto /var/lib/docker/. /mnt/ciproxy-docker/",
                        "stat --file-system --format=%a %S /mnt/ciproxy-docker",
                        "umount /mnt/ciproxy-docker",
                        "mount /dev/vdb /var/lib/docker",
                        "systemctl start docker.socket docker.service",
                        "systemctl is-active --quiet docker.service"]:
            with self.subTest(failure=failure):
                root, result = self.run_attach(failure)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual((root / "state/docker-mounted").read_text(), "0")
                self.assertEqual((root / "state/preparation-mounted").read_text(), "0")
                self.assertEqual((root / "state/daemon").read_text(), "1")
                self.assertEqual((root / "var/lib/docker/root-data").read_text(), "original Docker data\n")
                self.assertEqual((root / "state/failed").read_text(), "1")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path, default=DockerAttach.workspace)
    args, remaining = parser.parse_known_args()
    DockerAttach.workspace = args.work_dir
    started = time.perf_counter()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(DockerAttach))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    raise SystemExit(0 if result.wasSuccessful() else 1)
