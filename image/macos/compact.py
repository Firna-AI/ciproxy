#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Copy a VM disk to a sparse file. Holes and zero blocks stay unallocated.

The guest frees blocks with TRIM, and the build fills no free space with
zeros, so most free space is already a hole. Zero blocks that remain are
skipped too. The copy keeps the logical size of the source.
"""

import argparse
import os
import pathlib

BLOCK = 1024 * 1024


def data_ranges(source, size):
    """Yield (start, end) of each data range. Fall back to one range."""
    offset = 0
    try:
        while offset < size:
            start = os.lseek(source, offset, os.SEEK_DATA)
            end = os.lseek(source, start, os.SEEK_HOLE)
            yield start, end
            offset = end
    except OSError:
        if offset == 0:
            yield 0, size


def compact(source_path, target_path):
    size = source_path.stat().st_size
    zero = bytes(BLOCK)
    copied = 0
    with open(source_path, "rb") as source, open(target_path, "wb") as target:
        target.truncate(size)
        for start, end in data_ranges(source.fileno(), size):
            source.seek(start)
            position = start
            while position < end:
                block = source.read(min(BLOCK, end - position))
                if not block:
                    raise SystemExit(f"short read at {position} in {source_path}")
                if block != zero[:len(block)]:
                    target.seek(position)
                    target.write(block)
                    copied += len(block)
                position += len(block)
        target.flush()
        os.fsync(target.fileno())
    return size, copied


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=pathlib.Path)
    parser.add_argument("target", type=pathlib.Path)
    args = parser.parse_args()
    size, copied = compact(args.source, args.target)
    allocated = args.target.stat().st_blocks * 512
    print(f"logical {size} bytes, data {copied} bytes, allocated {allocated} bytes")


if __name__ == "__main__":
    main()
