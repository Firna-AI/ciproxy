#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Capture release file capabilities and restore them after Boat snapshots."""

import argparse
import errno
import hashlib
import json
import os
from pathlib import Path
import stat

METADATA = "etc/ciproxy/file-capabilities.json"


def digest(source):
    checksum = hashlib.sha256()
    for block in iter(lambda: source.read(1024 * 1024), b""):
        checksum.update(block)
    return checksum.hexdigest()


def capture(rootfs):
    device = rootfs.stat().st_dev
    files = []

    def fail(error):
        raise error

    for directory, directories, names in os.walk(rootfs, followlinks=False, onerror=fail):
        parent = Path(directory)
        directories[:] = sorted(name for name in directories
                                if not (parent / name).is_symlink()
                                and (parent / name).stat().st_dev == device
                                and not (parent / name).is_mount())
        for name in sorted(names):
            path = parent / name
            metadata = path.lstat()
            if not stat.S_ISREG(metadata.st_mode) or metadata.st_dev != device:
                continue
            try:
                capability = os.getxattr(path, "security.capability", follow_symlinks=False)
            except OSError as error:
                if error.errno != errno.ENODATA:
                    raise
                continue
            with path.open("rb") as source:
                checksum = digest(source)
            files.append({"path": path.relative_to(rootfs).as_posix(), "sha256": checksum,
                          "security_capability": capability.hex()})
    destination = rootfs / METADATA
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps({"version": 1, "files": sorted(files, key=lambda item: item["path"])},
                                      indent=2) + "\n")


def restore(rootfs):
    metadata = json.loads((rootfs / METADATA).read_text())
    if metadata["version"] != 1:
        raise ValueError("Unsupported file capability metadata version")
    for item in metadata["files"]:
        relative = Path(item["path"])
        if relative.is_absolute() or not relative.parts or ".." in relative.parts:
            raise ValueError("File capability path must stay within the release tree")
        path = rootfs / relative
        if path.resolve(strict=True) != path:
            raise ValueError("File capability path must not follow a symbolic link")
        with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW), "rb") as source:
            if not stat.S_ISREG(os.fstat(source.fileno()).st_mode):
                raise ValueError("File capability target must be a regular release file")
            if digest(source) != item["sha256"]:
                raise ValueError("File capability target does not match the trusted release")
            os.setxattr(source.fileno(), "security.capability", bytes.fromhex(item["security_capability"]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["capture", "restore"])
    parser.add_argument("rootfs", type=Path)
    args = parser.parse_args()
    globals()[args.operation](args.rootfs.resolve())


if __name__ == "__main__":
    main()
