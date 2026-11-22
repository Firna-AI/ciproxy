#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Write and check macos.tar in the form that the agent imports.

The agent accepts only regular file members at the paths of MEMBERS in
crates/ciproxy-agent/src/macos/image_bundle.rs. macOS bsdtar writes sparse
files as GNU sparse members and can add extended attributes, which the agent
refuses. So the archive is written here with plain regular members, and the
check reads the raw headers and the member list of the agent's source.
"""

import argparse
import pathlib
import re
import tarfile

REPOSITORY = pathlib.Path(__file__).resolve().parents[2]
AGENT_BUNDLE = REPOSITORY / "crates/ciproxy-agent/src/macos/image_bundle.rs"
PAX_KEYS = {"size", "path", "mtime"}
BLOCK = 512


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


def pack(folder, archive, members):
    """Write the members as plain regular files, in the agent's order."""
    folder = pathlib.Path(folder)
    with tarfile.open(archive, "w", format=tarfile.PAX_FORMAT) as tar:
        for member in members:
            path = folder / member
            info = tarfile.TarInfo(member)
            info.size = path.stat().st_size
            info.mode, info.mtime = 0o644, int(path.stat().st_mtime)
            info.uid = info.gid = 0
            info.uname = info.gname = ""
            with path.open("rb") as data:
                tar.addfile(info, data)


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


def check(archive, members):
    """Fail unless the raw headers hold exactly the members, each once."""
    seen, pax = [], {}
    with open(archive, "rb") as tar:
        while True:
            header = tar.read(BLOCK)
            if len(header) < BLOCK:
                raise SystemExit("the archive ends without its end blocks")
            if header == bytes(BLOCK):
                break
            kind, size = chr(header[156]), octal(header[124:136])
            name = header[0:100].rstrip(b"\0").decode()
            ustar = header[257:262] == b"ustar"
            prefix = header[345:500].rstrip(b"\0").decode() if ustar else ""
            path = f"{prefix}/{name}" if prefix else name
            padded = -(-size // BLOCK) * BLOCK
            if kind == "x":
                pax = pax_records(tar.read(padded)[:size])
                extra = set(pax) - PAX_KEYS
                if extra:
                    raise SystemExit(f"a PAX header of {path} has other keys: {sorted(extra)}")
                continue
            if kind not in ("0", "\0"):
                raise SystemExit(f"member {path} has type {kind!r}, not a regular file")
            path = pax.get("path", path)
            if int(pax.get("size", size)) != size:
                raise SystemExit(f"member {path} has a PAX size that differs from its header")
            pax = {}
            tar.seek(padded, 1)
            if path not in members:
                raise SystemExit(f"member {path} is not in the agent's member list")
            seen.append(path)
    if sorted(seen) != sorted(members):
        raise SystemExit(f"the archive has {sorted(seen)}, not the agent's {sorted(members)}")
    print(f"{archive}: {len(seen)} regular members, as the agent expects")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["pack", "check"])
    parser.add_argument("archive")
    parser.add_argument("--folder", help="the folder of the members, for pack")
    args = parser.parse_args()
    members = agent_members()
    if args.command == "pack":
        pack(args.folder, args.archive, members)
    check(args.archive, members)


if __name__ == "__main__":
    main()
