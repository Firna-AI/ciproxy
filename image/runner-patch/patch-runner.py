#!/usr/bin/env python3
"""Patch the runner's managed string table without moving PE data."""

import argparse
import hashlib
import os
from pathlib import Path
import tempfile


NAMES = (
    "ACTIONS_CACHE_URL",
    "ACTIONS_CACHE_SERVICE_V2",
    "ACTIONS_RESULTS_URL",
)


def patch(data: bytes) -> tuple[bytes, bool]:
    result = data
    changed = False
    for name in NAMES:
        replacement = "Z" + name[1:]
        original_count = 0
        patched_count = 0
        for encoding in ("utf-16le", "ascii"):
            original = name.encode(encoding)
            patched = replacement.encode(encoding)
            original_count += result.count(original)
            patched_count += result.count(patched)
            result = result.replace(original, patched)
        if original_count and patched_count:
            raise ValueError(f"mixed patched and original strings for {name}")
        if not original_count and not patched_count:
            raise ValueError(f"runner has no supported string for {name}")
        changed = changed or bool(original_count)
    if len(result) != len(data):
        raise ValueError("patch changed the DLL length")
    return result, changed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dll", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data = args.dll.read_bytes()
    patched, changed = patch(data)
    if args.check:
        if changed:
            parser.exit(1, "runner is not patched\n")
        print("runner is patched")
        return
    destination = args.output or args.dll
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as file:
        temporary = Path(file.name)
        file.write(patched)
        file.flush()
        os.fsync(file.fileno())
    try:
        temporary.chmod(args.dll.stat().st_mode)
        temporary.replace(destination)
    finally:
        temporary.unlink(missing_ok=True)
    print(f"{'patched' if changed else 'already patched'}: {destination}")
    print(f"bytes: {len(patched)}")
    print(f"sha256: {hashlib.sha256(patched).hexdigest()}")


if __name__ == "__main__":
    main()
