#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Guard the optional Boat NVM lookup without changing its fallback."""

import argparse
import os
import pathlib
import shutil
import sys
import tempfile


ASSIGNMENT = b'\n__ascii_nvm_bin="$(__ascii_nvm_path 2>/dev/null)"'
FALLBACK = b'\nif [ -n "$__ascii_nvm_bin" ]; then\n'
ORIGINAL = ASSIGNMENT + FALLBACK
GUARDED = ASSIGNMENT + b" || __ascii_nvm_bin=''" + FALLBACK


def guard(profile):
    try:
        if profile.is_symlink():
            print("Warning: Boat NVM profile is a link. Leave it unchanged.", file=sys.stderr)
            return
        content = profile.read_bytes()
        if content.count(GUARDED) == 1 and ORIGINAL not in content:
            return
        if content.count(ORIGINAL) != 1 or GUARDED in content:
            print("Warning: Boat NVM profile has no recognized lookup. Leave it unchanged.",
                  file=sys.stderr)
            return
        metadata = profile.stat()
        with tempfile.TemporaryDirectory(prefix=".ciproxy-nvm-", dir=profile.parent) as directory:
            candidate = pathlib.Path(directory) / profile.name
            candidate.write_bytes(content.replace(ORIGINAL, GUARDED, 1))
            os.chown(candidate, metadata.st_uid, metadata.st_gid)
            shutil.copystat(profile, candidate)
            candidate.replace(profile)
    except FileNotFoundError:
        print("Warning: Boat NVM profile is missing.", file=sys.stderr)
    except OSError:
        print("Warning: Could not read or guard the Boat NVM profile.", file=sys.stderr)
    else:
        print("Boat NVM: guarded the optional path lookup.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile", nargs="?", type=pathlib.Path, default="/etc/profile.d/nvm.sh")
    guard(parser.parse_args().profile)
