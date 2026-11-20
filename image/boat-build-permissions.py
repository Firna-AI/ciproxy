#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Retain existing provider permissions during upstream tool permission setup."""

import argparse
import json
import os
import pathlib
import stat


ROOTS = ("/opt", "/usr/share", "/usr/local/bin", "/usr/local/lib/node_modules")
BASELINE = pathlib.Path("/opt/ciproxy-build/provider-permissions.json")


def paths(root):
    yield root
    for directory, folders, files in os.walk(root, followlinks=False):
        folders[:] = [name for name in folders if not pathlib.Path(directory, name).is_symlink()
                      and pathlib.Path(directory, name) != BASELINE.parent]
        for name in folders + files:
            path = pathlib.Path(directory, name)
            if not path.is_symlink():
                yield path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?")
    parser.add_argument("--capture", action="store_true")
    args = parser.parse_args()
    if args.capture:
        before = {str(path): stat.S_IMODE(path.stat().st_mode)
                  for root in ROOTS if pathlib.Path(root).exists() for path in paths(pathlib.Path(root))}
        BASELINE.write_text(json.dumps(before) + "\n")
    else:
        before = json.loads(BASELINE.read_text())
        for path in paths(pathlib.Path(args.root)):
            if str(path) not in before:
                path.chmod(0o777)


if __name__ == "__main__":
    main()
