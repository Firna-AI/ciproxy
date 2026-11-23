#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Write and check macos.tar in the form that the agent imports.

The agent accepts only regular file members at the paths of MEMBERS in
crates/ciproxy-agent/src/macos/image_bundle.rs. macOS bsdtar writes sparse
files as GNU sparse members and can add extended attributes, which the agent
refuses. So the archive is written here with plain regular members, and the
check reads the raw headers and the member list of the agent's source.

The disk member is compressed with zstd straight into the archive, so the
build needs no loose copy of the compressed disk. Its PAX size record has a
fixed width and is filled in after the stream ends.
"""

import argparse
import hashlib
import pathlib
import re
import subprocess
import tarfile
import threading

REPOSITORY = pathlib.Path(__file__).resolve().parents[2]
AGENT_BUNDLE = REPOSITORY / "crates/ciproxy-agent/src/macos/image_bundle.rs"
PAX_KEYS = {"size", "path", "mtime"}
BLOCK = 512
CHUNK = 4 * 1024 * 1024
DISK = "bundle/disk.img"
DISK_ARCHIVE = "bundle/disk.img.zst"
CONTENT_SUMS = "macos-content.sha256"
# Nineteen digits hold any size up to 9.99 EB. The stream end writes the
# real size, padded with leading zeros, in the same place.
SIZE_PLACEHOLDER = 10**19 - 1


def agent_members(source=AGENT_BUNDLE):
    """The MEMBERS list of the agent, with its constant names resolved."""
    text = pathlib.Path(source).read_text()
    constants = dict(re.findall(r'const (\w+): &str = "([^"]+)";', text))
    found = re.search(r"const MEMBERS: \[&str; (\d+)\] = \[(.*?)\];", text, re.S)
    if not found:
        raise SystemExit(f"no MEMBERS list in {source}")
    members = []
    for item in re.findall(r'"[^"]+"|\w+', found.group(2)):
        members.append(item.strip('"') if item.startswith('"') else constants[item])
    if len(members) != int(found.group(1)):
        raise SystemExit(f"the MEMBERS list of {source} has an unexpected form")
    return members


def member_info(name, size, mtime):
    info = tarfile.TarInfo(name)
    info.size, info.mode, info.mtime = size, 0o644, int(mtime)
    info.uid = info.gid = 0
    info.uname = info.gname = ""
    return info


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as source:
        for chunk in iter(lambda: source.read(CHUNK), b""):
            digest.update(chunk)
    return digest.hexdigest()


def stream_disk(out, disk, level):
    """Compress the disk into `out`. Return the disk and zstd SHA-256 and size."""
    zstd = subprocess.Popen(["zstd", "-T0", f"-{level}", "-q", "-c", "-"],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    disk_digest, failure = hashlib.sha256(), []

    def feed():
        try:
            with open(disk, "rb") as source:
                for chunk in iter(lambda: source.read(CHUNK), b""):
                    disk_digest.update(chunk)
                    zstd.stdin.write(chunk)
        except OSError as error:
            failure.append(error)
        finally:
            zstd.stdin.close()

    feeder = threading.Thread(target=feed)
    feeder.start()
    zst_digest, size = hashlib.sha256(), 0
    for chunk in iter(lambda: zstd.stdout.read(CHUNK), b""):
        zst_digest.update(chunk)
        out.write(chunk)
        size += len(chunk)
    feeder.join()
    zstd.stdout.close()
    if zstd.wait() != 0 or failure:
        raise SystemExit(f"zstd failed for {disk}: {failure}")
    return disk_digest.hexdigest(), zst_digest.hexdigest(), size


def pack(folder, archive, members, level=10):
    """Write the members as plain regular files, in the agent's order. The
    disk member is the zstd stream of bundle/disk.img. The content sums file
    is written from the hashes, so the disk is read only once."""
    folder = pathlib.Path(folder)
    if members[0] != DISK_ARCHIVE or members[-1] != CONTENT_SUMS:
        raise SystemExit("the agent's member order changed")
    disk = folder / DISK
    with open(archive, "wb") as out:
        header = member_info(DISK_ARCHIVE, SIZE_PLACEHOLDER, disk.stat().st_mtime).tobuf(
            tarfile.PAX_FORMAT, "utf-8", "surrogateescape")
        field = header.index(f"size={SIZE_PLACEHOLDER}".encode()) + len("size=")
        start = out.tell()
        out.write(header)
        disk_sum, zst_sum, size = stream_disk(out, disk, level)
        out.write(bytes(-size % BLOCK))
        end = out.tell()
        out.seek(start + field)
        out.write(f"{size:019d}".encode())
        out.seek(end)
        sums = {DISK_ARCHIVE: zst_sum, DISK: disk_sum}
        for member in members[1:-1]:
            sums[member] = sha256(folder / member)
        (folder / CONTENT_SUMS).write_text(
            "".join(f"{sums[name]}  {name}\n" for name in [DISK, *members[:-1]]))
        with tarfile.open(fileobj=out, mode="w", format=tarfile.PAX_FORMAT) as tar:
            for member in members[1:]:
                path = folder / member
                with path.open("rb") as data:
                    tar.addfile(member_info(member, path.stat().st_size,
                                            path.stat().st_mtime), data)


def octal(field):
    if field[0] & 0x80:
        return int.from_bytes(field[1:], "big")
    text = field.rstrip(b"\0 ").decode() or "0"
    return int(text, 8)


def pax_records(data):
    records, offset = {}, 0
    while offset < len(data):
        length = int(data[offset:data.index(b" ", offset)])
        key, value = data[data.index(b" ", offset) + 1:offset + length - 1].split(b"=", 1)
        records[key.decode()] = value.decode()
        offset += length
    return records


def headers(archive):
    """Yield (path, type, data offset, size) of each member, PAX resolved."""
    pax = {}
    with open(archive, "rb") as tar:
        while True:
            header = tar.read(BLOCK)
            if len(header) < BLOCK:
                raise SystemExit("the archive ends without its end blocks")
            if header == bytes(BLOCK):
                return
            kind, field = chr(header[156]), octal(header[124:136])
            name = header[0:100].rstrip(b"\0").decode()
            ustar = header[257:262] == b"ustar"
            prefix = header[345:500].rstrip(b"\0").decode() if ustar else ""
            path = f"{prefix}/{name}" if prefix else name
            if kind == "x":
                pax = pax_records(tar.read(-(-field // BLOCK) * BLOCK)[:field])
                extra = set(pax) - PAX_KEYS
                if extra:
                    raise SystemExit(f"a PAX header of {path} has other keys: {sorted(extra)}")
                continue
            # A PAX size overrides the header field, which holds 0 above 8 GiB.
            size = int(pax["size"]) if "size" in pax else field
            if "size" in pax and field not in (0, size):
                raise SystemExit(f"member {path} has a PAX size that differs from its header")
            path = pax.get("path", path)
            pax = {}
            yield path, kind, tar.tell(), size
            tar.seek(-(-size // BLOCK) * BLOCK, 1)


def check(archive, members, verify=False):
    """Fail unless the raw headers hold exactly the members, each once. With
    verify, decompress the disk member and compare it with the content sums."""
    seen, places = [], {}
    for path, kind, offset, size in headers(archive):
        if kind not in ("0", "\0"):
            raise SystemExit(f"member {path} has type {kind!r}, not a regular file")
        if path not in members:
            raise SystemExit(f"member {path} is not in the agent's member list")
        seen.append(path)
        places[path] = (offset, size)
    if sorted(seen) != sorted(members):
        raise SystemExit(f"the archive has {sorted(seen)}, not the agent's {sorted(members)}")
    if verify:
        verify_disk(archive, places)
    print(f"{archive}: {len(seen)} regular members, as the agent expects")


def read_member(archive, place):
    offset, size = place
    with open(archive, "rb") as tar:
        tar.seek(offset)
        while size:
            chunk = tar.read(min(CHUNK, size))
            if not chunk:
                raise SystemExit("the archive ends inside a member")
            size -= len(chunk)
            yield chunk


def verify_disk(archive, places):
    sums = {line[66:]: line[:64] for line in
            b"".join(read_member(archive, places[CONTENT_SUMS])).decode().splitlines()}
    zstd = subprocess.Popen(["zstd", "-dc", "-q"], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE)

    def feed():
        try:
            for chunk in read_member(archive, places[DISK_ARCHIVE]):
                zstd.stdin.write(chunk)
        finally:
            zstd.stdin.close()

    feeder = threading.Thread(target=feed)
    feeder.start()
    digest = hashlib.sha256()
    for chunk in iter(lambda: zstd.stdout.read(CHUNK), b""):
        digest.update(chunk)
    feeder.join()
    zstd.stdout.close()
    if zstd.wait() != 0 or digest.hexdigest() != sums.get(DISK):
        raise SystemExit("the decompressed disk does not match its content sum")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["pack", "check"])
    parser.add_argument("archive")
    parser.add_argument("--folder", help="the folder of the members, for pack")
    parser.add_argument("--level", type=int, default=10, help="the zstd level, for pack")
    parser.add_argument("--verify", action="store_true",
                        help="decompress the disk and compare it with the sums")
    args = parser.parse_args()
    members = agent_members()
    if args.command == "pack":
        pack(args.folder, args.archive, members, args.level)
    check(args.archive, members, args.verify or args.command == "pack")


if __name__ == "__main__":
    main()
