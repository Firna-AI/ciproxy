#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Make the GPT of a cut VM disk match its new length.

After the shrink cuts disk.img, the primary GPT header still names the old
last sectors, and the backup GPT is gone. The VM firmware then does not
start the guest. This rewrites the primary header, writes the backup
entries and header at the new end, and fixes the protective MBR. It stops
when a partition does not fit inside the new disk.
"""

import argparse
import struct
import zlib

SECTOR = 512


def header_bytes(fields, size):
    """Pack a GPT header with its CRC32 over the first `size` bytes."""
    header = bytearray(struct.pack("<8sIIII QQQQ 16s QIII", *fields))
    header[16:20] = struct.pack("<I", zlib.crc32(bytes(header[:size])) & 0xFFFFFFFF)
    return bytes(header)


def fix(path):
    with open(path, "r+b") as disk:
        sectors = disk.seek(0, 2) // SECTOR
        disk.seek(SECTOR)
        header = disk.read(SECTOR)
        (signature, revision, size, _crc, reserved, current, _alternate, first, _last,
         guid, entries_lba, count, entry_size, entries_crc) = struct.unpack(
            "<8sIIII QQQQ 16s QIII", header[:92])
        if signature != b"EFI PART" or current != 1:
            raise SystemExit(f"{path} has no primary GPT header")
        table_bytes = count * entry_size
        table_sectors = -(-table_bytes // SECTOR)
        disk.seek(entries_lba * SECTOR)
        table = disk.read(table_bytes)
        if zlib.crc32(table) & 0xFFFFFFFF != entries_crc:
            raise SystemExit(f"{path} has a GPT partition table with a wrong checksum")
        alternate = sectors - 1
        backup_entries = alternate - table_sectors
        last = backup_entries - 1
        for index in range(count):
            entry = table[index * entry_size:(index + 1) * entry_size]
            if entry[:16] != bytes(16) and struct.unpack("<Q", entry[40:48])[0] > last:
                raise SystemExit(f"partition {index} ends after the last usable sector {last}")
        fields = [signature, revision, size, 0, reserved, 1, alternate, first, last, guid,
                  entries_lba, count, entry_size, entries_crc]
        disk.seek(SECTOR)
        disk.write(header_bytes(fields, size))
        backup = fields.copy()
        backup[5], backup[6], backup[10] = alternate, 1, backup_entries
        disk.seek(backup_entries * SECTOR)
        disk.write(table)
        disk.seek(alternate * SECTOR)
        disk.write(header_bytes(backup, size).ljust(SECTOR, b"\0"))
        disk.seek(0)
        mbr = bytearray(disk.read(SECTOR))
        if mbr[450] == 0xEE:
            mbr[458:462] = struct.pack("<I", min(sectors - 1, 0xFFFFFFFF))
            disk.seek(0)
            disk.write(mbr)
    print(f"GPT fixed: {sectors} sectors, last usable {last}, backup header {alternate}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("disk")
    fix(parser.parse_args().disk)


if __name__ == "__main__":
    main()
