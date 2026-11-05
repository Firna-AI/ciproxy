#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Measure an installed full rootfs and its optional published artifacts."""

import argparse
import json
import os
import pathlib
import subprocess


def du(path, apparent=False):
    flags = ["du", "--summarize", "--one-file-system", "--block-size=1"]
    if apparent:
        flags.append("--apparent-size")
    return int(subprocess.check_output(flags + [str(path)], text=True).split()[0])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rootfs", type=pathlib.Path)
    parser.add_argument("--artifacts", type=pathlib.Path)
    parser.add_argument("--job", action="store_true",
                        help="The rootfs is on this job's actual filesystem")
    args = parser.parse_args()
    rootfs = args.rootfs.resolve()
    disk = os.statvfs(rootfs)
    result = {
        "rootfs": "rootfs",
        "installed_apparent_bytes": du(rootfs, True),
        "installed_allocated_bytes": du(rootfs),
        "backing_filesystem_bytes": disk.f_blocks * disk.f_frsize,
        "backing_filesystem_available_bytes": disk.f_bavail * disk.f_frsize,
        "artifacts": {},
    }
    if args.job:
        result["free_job_bytes"] = disk.f_bavail * disk.f_frsize
    if args.artifacts:
        for name in ["rootfs.ext4", "rootfs.tar.zst", "vmlinux"]:
            path = args.artifacts / name
            info = path.stat()
            result["artifacts"][name] = {
                "logical_bytes": info.st_size,
                "allocated_bytes": info.st_blocks * 512,
            }
        headers = subprocess.check_output(
            ["dumpe2fs", "-h", str(args.artifacts / "rootfs.ext4")],
            text=True, stderr=subprocess.DEVNULL, env={**os.environ, "LC_ALL": "C"})
        fields = dict(line.split(":", 1) for line in headers.splitlines() if ":" in line)
        block_size = int(fields["Block size"])
        result["firecracker_ext4"] = {
            "filesystem_bytes": int(fields["Block count"]) * block_size,
            "offline_free_job_bytes":
                (int(fields["Free blocks"]) - int(fields["Reserved block count"])) * block_size,
        }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
