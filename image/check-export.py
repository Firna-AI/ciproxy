#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check source mount order and both finalized artifacts without KVM."""

import argparse
import json
import os
import pathlib
import re
import shutil
import struct
import sys
import time
import unittest

sys.dont_write_bytecode = True

from _checks_.export_fixture import (
    CAPABILITY, RELEASE_FSTAB, SOURCE_FSTAB, create, disk_attribute, disk_file, run,
)


def acl_entries(value):
    data = bytes.fromhex(value)
    assert struct.unpack_from("<I", data)[0] == 2
    return [(tag, permissions, identifier if tag in (2, 8) else None)
            for tag, permissions, identifier in struct.iter_unpack("<HHI", data[4:])]


def check_export(workspace):
    assert os.geteuid() == 0, "Run this fixture check with sudo for chroot and file capabilities"
    image = pathlib.Path(__file__).resolve().parent
    root, source, work, environment = create(workspace, image)
    print("Fixture evidence: " + str(root), flush=True)
    run([shutil.which("chroot"), str(source), "/usr/bin/bash", "/image/provision.sh"], root,
        "provision", {**os.environ, "PATH": "/usr/bin"})
    assert (source / "etc/fstab").read_text() == SOURCE_FSTAB, (
        "source fstab changed during provisioning before disk inspection")
    run(["bash", str(image / "export.sh"), str(work)], root, "export", environment)
    events = [json.loads(line) for line in (root / "events.jsonl").read_text().splitlines()]
    assert [event["tool"] for event in events] == ["virt-customize", "guestfish"]
    assert all(event["source_fstab"] == SOURCE_FSTAB for event in events)
    assert (source / "etc/fstab").read_text() == SOURCE_FSTAB
    assert (work / "rootfs/etc/fstab").read_text() == RELEASE_FSTAB
    intermediate = list(work.glob("*.tar.zst"))
    assert len(intermediate) == 1, "Keep one source archive outside the final artifact directory"
    names = run(["tar", "-I", "zstd", "-tf", str(intermediate[0])], root, "source-archive-list")
    fstab_name = next(name for name in names.splitlines() if name.lstrip("./") == "etc/fstab")
    assert run(["tar", "-I", "zstd", "-xOf", str(intermediate[0]), fstab_name], root,
               "source-archive-fstab") == SOURCE_FSTAB

    artifacts = work / "artifacts"
    measurement = json.loads((artifacts / "measurement.json").read_text())
    manifest = json.loads((artifacts / "manifest.json").read_text())
    assert measurement["rootfs"] == "rootfs", "Release measurement must use a neutral root label"
    assert manifest["measurements"]["rootfs"] == "rootfs", (
        "The manifest must not contain the build machine's extraction path")
    boat = root / "boat"
    boat.mkdir()
    run(["bash", str(image / "rootfs-archive.sh"), "extract",
         str(artifacts / "rootfs.tar.zst"), str(boat)], root, "boat-extract")
    assert (boat / "etc/fstab").read_text() == RELEASE_FSTAB, (
        "Boat archive did not come from the finalized filesystem")
    disk = artifacts / "rootfs.ext4"
    geometry = run(["dumpe2fs", "-h", str(disk)], root, "ext4-geometry")
    fields = dict(line.split(":", 1) for line in geometry.splitlines() if ":" in line)
    assert int(fields["Reserved block count"]) == int(fields["Block count"]) // 100, (
        "The job disk must reserve one percent of its blocks for root")
    assert disk_file(disk, "/etc/fstab", root, "ext4-fstab").decode() == RELEASE_FSTAB
    expected_acl = [(1, 7, None), (2, 5, 34567), (4, 5, None),
                    (16, 5, None), (32, 5, None)]
    for tree in [work / "rootfs", boat]:
        tool = tree / "privileged-tool"
        assert (tool.stat().st_uid, tool.stat().st_gid, tool.stat().st_mode & 0o7777) == (1234, 2345, 0o755)
        assert os.getxattr(tool, "security.capability").hex() == CAPABILITY
        assert acl_entries(os.getxattr(tool, "system.posix_acl_access").hex()) == expected_acl
        assert tool.stat().st_ino == (tree / "tool-alias").stat().st_ino
        assert (tree / "sparse-data").stat().st_blocks * 512 < 8388608
    assert disk_attribute(disk, "/privileged-tool", "security.capability", root,
                          "ext4-capability") == CAPABILITY
    assert acl_entries(disk_attribute(disk, "/privileged-tool", "system.posix_acl_access",
                                      root, "ext4-acl")) == expected_acl
    tool_stat = run(["debugfs", "-R", "stat /privileged-tool", str(disk)], root, "ext4-tool")
    alias_stat = run(["debugfs", "-R", "stat /tool-alias", str(disk)], root, "ext4-alias")
    assert re.search(r"Inode: (\d+)", tool_stat).group(1) == re.search(r"Inode: (\d+)", alias_stat).group(1)
    sparse_stat = run(["debugfs", "-R", "stat /sparse-data", str(disk)], root, "ext4-sparse")
    assert int(re.search(r"Blockcount: (\d+)", sparse_stat).group(1)) * 512 < 8388608
    run(["bash", "-c", 'cd "$1" && sha256sum --check SHA256SUMS', "check", str(artifacts)],
        root, "release-checksums")
    (root / "result.json").write_text(json.dumps({
        "fixture_only": True, "source_mounts_preserved": True,
        "firecracker_fstab": RELEASE_FSTAB, "boat_fstab": RELEASE_FSTAB,
        "capabilities_acl_owners_hardlinks_sparse_preserved": True,
        "real_tools": ["tar", "zstd", "mkfs.ext4", "debugfs", "measure.py", "package.sh"],
        "stubbed_tools": ["provisioning tools inside chroot", "virt-customize", "guestfish", "sudo"],
        "not_proved": "Real libguestfs inspection and mounting, a full image build, or VM boot",
    }, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=pathlib.Path,
                        default=pathlib.Path(".context/ciproxy-mvp/orchestrator-fix-1"))
    args = parser.parse_args()
    started = time.perf_counter()
    case = unittest.FunctionTestCase(lambda: check_export(args.work_dir),
                                     description="source mounts survive until both artifacts are finalized")
    result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite([case]))
    print(f"Regression command duration: {time.perf_counter() - started:.3f}s")
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    sys.exit(main())
