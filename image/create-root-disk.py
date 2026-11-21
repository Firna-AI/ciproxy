#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Size ext4 from allocated userspace, measured metadata, and a base margin."""

import argparse
import json
import os
from pathlib import Path
import subprocess

BLOCK_BYTES = 4096
FREE_MARGIN_BYTES = 64 * 1024 ** 2


def run(*command):
    print("$ " + " ".join(map(str, command)), flush=True)
    subprocess.run(list(map(str, command)), check=True)


def geometry(disk):
    output = subprocess.check_output(["dumpe2fs", "-h", str(disk)], text=True,
                                     stderr=subprocess.DEVNULL,
                                     env={**os.environ, "LC_ALL": "C"})
    fields = dict(line.split(":", 1) for line in output.splitlines() if ":" in line)
    return {key: int(fields[key]) for key in
            ["Block count", "Free blocks", "Reserved block count", "Block size",
             "Blocks per group"]}


def round_blocks(size):
    return (size + BLOCK_BYTES - 1) // BLOCK_BYTES * BLOCK_BYTES


def create(rootfs, disk):
    allocated = int(subprocess.check_output(
        ["du", "--summarize", "--one-file-system", "--block-size=1", str(rootfs)],
        text=True).split()[0])
    override = os.environ.get("ROOTFS_BYTES")
    size = int(override) if override else round_blocks(allocated + FREE_MARGIN_BYTES)
    metadata = None
    options = ["mkfs.ext4", "-F", "-b", BLOCK_BYTES, "-m", "1", "-L", "ciproxy-root"]
    if not override:
        while True:
            # Empty layouts measure the actual metadata of this mkfs version.
            run("truncate", "-s", size, disk)
            run(*options, disk)
            values = geometry(disk)
            available = (values["Free blocks"] - values["Reserved block count"]) * BLOCK_BYTES
            metadata = (values["Block count"] - values["Free blocks"]) * BLOCK_BYTES
            if available >= allocated + FREE_MARGIN_BYTES:
                break
            needed = ((allocated + FREE_MARGIN_BYTES - available) * 100 + 98) // 99
            group_bytes = values["Blocks per group"] * BLOCK_BYTES
            size = values["Block count"] * BLOCK_BYTES + needed
            size = (size + group_bytes - 1) // group_bytes * group_bytes
    run("truncate", "-s", size, disk)
    run(*options, "-d", rootfs, disk)
    values = geometry(disk)
    return {"root_disk_bytes": disk.stat().st_size,
            "installed_allocated_bytes": allocated,
            "empty_metadata_bytes": metadata,
            "free_margin_bytes": FREE_MARGIN_BYTES if not override else None,
            "offline_free_bytes": (values["Free blocks"] - values["Reserved block count"]) * BLOCK_BYTES,
            "root_reserve_percent": 1,
            "size_override": bool(override)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rootfs", type=Path)
    parser.add_argument("disk", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    result = create(args.rootfs.resolve(), args.disk.resolve())
    args.report.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
