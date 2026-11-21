#!/usr/bin/env python3
"""Apache-2.0. Prove shared physical extents of two distinct regular files."""

import argparse
import errno
import fcntl
import json
import os
import stat
import struct
import sys

FIEMAP = 0xC020660B
SHARED = 0x2000
LAST = 1
HEADER = struct.Struct("=QQIIII")
EXTENT = struct.Struct("=QQQQQIIII")


def extents(path):
    """Read kernel FIEMAP records, with a bounded evidence sample."""
    records = []
    start = 0
    with open(path, "rb") as stream:
        for _ in range(64):
            buffer = bytearray(HEADER.size + 256 * EXTENT.size)
            HEADER.pack_into(buffer, 0, start, (1 << 64) - 1 - start, 1, 0, 256, 0)
            fcntl.ioctl(stream.fileno(), FIEMAP, buffer, True)
            count = HEADER.unpack_from(buffer)[3]
            batch = []
            for index in range(count):
                item = EXTENT.unpack_from(buffer, HEADER.size + index * EXTENT.size)
                batch.append(dict(logical=item[0], physical=item[1], length=item[2], flags=item[5]))
            records.extend(batch)
            if not batch or batch[-1]["flags"] & LAST:
                return records, False
            start = batch[-1]["logical"] + batch[-1]["length"]
    return records, True


def shared_ranges(source, clone):
    """Require shared blocks at equal logical offsets, not matching block numbers alone."""
    found = []
    for left in source:
        if not left["flags"] & SHARED:
            continue
        for right in clone:
            if not right["flags"] & SHARED:
                continue
            begin = max(left["logical"], right["logical"])
            end = min(left["logical"] + left["length"], right["logical"] + right["length"])
            if begin >= end:
                continue
            physical = left["physical"] + begin - left["logical"]
            if physical == right["physical"] + begin - right["logical"]:
                found.append(dict(logical=begin, physical=physical, length=end - begin))
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source")
    parser.add_argument("clone")
    args = parser.parse_args()
    source = os.stat(args.source, follow_symlinks=False)
    clone = os.stat(args.clone, follow_symlinks=False)
    if not stat.S_ISREG(source.st_mode) or not stat.S_ISREG(clone.st_mode):
        raise ValueError("both extent inputs must be regular files")
    if source.st_dev != clone.st_dev or source.st_ino == clone.st_ino:
        raise ValueError("a reflink needs distinct files on the same filesystem")
    if clone.st_nlink != 1:
        raise ValueError("the VM root must not have another hard-link owner")
    base, base_partial = extents(args.source)
    copy, copy_partial = extents(args.clone)
    shared = shared_ranges(base, copy)
    print(json.dumps(dict(source=args.source, clone=args.clone, device=source.st_dev,
                         source_inode=source.st_ino, clone_inode=clone.st_ino,
                         clone_ctime_ns=clone.st_ctime_ns, clone_links=clone.st_nlink,
                         source_extents=base, clone_extents=copy,
                         sample_truncated=base_partial or copy_partial,
                         shared_ranges=shared), indent=2))
    if not shared:
        raise ValueError("no shared physical extent at an equal logical offset; reflink proof failed")


if __name__ == "__main__":
    try:
        main()
    except OSError as error:
        if error.errno in (errno.EOPNOTSUPP, errno.ENOTTY, errno.ENOSYS):
            print("SKIP: the host cannot provide FIEMAP extent evidence", file=sys.stderr)
            sys.exit(77)
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
    except ValueError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
