#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Write the macOS image measurement and manifest, in the form of the Linux ones.

The measurement adds the guest facts that the build read before shutdown and
the sizes of the VM files. The manifest copies the pins, the disk size, the
guest checksum, and the measurement. size-report.txt sums up the sizes.
"""

import argparse
import hashlib
import json
import os
import pathlib

# The files of a ciproxy-vz bundle that the image ships. The agent writes the
# machine identifier and the MAC address for each VM.
VM_FILES = ["bundle/disk.img", "bundle/aux.img", "bundle/hardware-model.bin"]


def sha256(path):
    checksum = hashlib.sha256()
    with open(path, "rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(block)
    return checksum.hexdigest()


def sizes(path):
    info = path.stat()
    return {"logical_bytes": info.st_size, "allocated_bytes": info.st_blocks * 512}


def gigabytes(value):
    return f"{value / 1e9:.2f} GB"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifacts", type=pathlib.Path)
    parser.add_argument("--guest-binary", type=pathlib.Path, required=True)
    parser.add_argument("--source-disk", type=pathlib.Path,
                        help="the build disk before compaction, for the size report")
    args = parser.parse_args()
    definition = pathlib.Path(__file__).resolve().parent
    pins = dict(line.split("=", 1) for line in (definition / "pins.env").read_text().splitlines()
                if line and not line.startswith("#"))
    guest = json.loads((args.artifacts / "guest-facts.json").read_text())
    # The guest's version is the minimum host version, because
    # Virtualization.framework runs no guest that is newer than its host.
    running = f"{guest['macos_version']}-{guest['macos_build']}"
    if running != pins["MACOS_TARGET"]:
        raise SystemExit(f"the guest runs {running}, not the pinned {pins['MACOS_TARGET']}")
    disk = args.artifacts / "bundle/disk.img"
    measurement = {
        "platform": "macos",
        "architecture": "arm64",
        "guest": guest,
        "root_disk_bytes": disk.stat().st_size,
        "free_job_bytes": guest["free_job_bytes"],
        "artifacts": {name: sizes(args.artifacts / name) for name in VM_FILES},
    }
    if args.source_disk:
        measurement["build_disk"] = sizes(args.source_disk)
    (args.artifacts / "measurement.json").write_text(json.dumps(measurement, indent=2) + "\n")
    manifest = {"image_id": os.environ.get("IMAGE_RELEASE_ID", pins["IMAGE_ID"]),
                "platform": "macos", "inputs": pins,
                "minimum_host_macos": guest["macos_version"],
                "root_disk_bytes": measurement["root_disk_bytes"],
                "free_job_bytes": measurement["free_job_bytes"],
                "guest_sha256": sha256(args.guest_binary),
                "measurements": measurement}
    (args.artifacts / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    lines = [f"image: {manifest['image_id']}",
             f"guest macOS: {guest['macos_version']} ({guest['macos_build']}), "
             f"so a host needs macOS {guest['macos_version']} or later",
             f"guest data volume used: {gigabytes(guest['data_volume_used_bytes'])}",
             f"free job space in the image: {gigabytes(guest['free_job_bytes'])}",
             f"disk.img logical: {gigabytes(disk.stat().st_size)}",
             f"disk.img allocated after compaction: {gigabytes(disk.stat().st_blocks * 512)}"]
    if args.source_disk:
        lines.append("build disk allocated before compaction: "
                     f"{gigabytes(measurement['build_disk']['allocated_bytes'])}")
    (args.artifacts / "size-report.txt").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
