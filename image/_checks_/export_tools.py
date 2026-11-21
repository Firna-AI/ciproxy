#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Model libguestfs transport without claiming to inspect or mount a disk."""

import json
import os
import pathlib
import subprocess
import sys


def main():
    tool = pathlib.Path(sys.argv[0]).name
    if tool == "sudo":
        os.execvp(sys.argv[1], sys.argv[1:])
    source = pathlib.Path(os.environ["CIPROXY_TEST_SOURCE"])
    fstab = (source / "etc/fstab").read_text()
    assert fstab == os.environ["CIPROXY_TEST_FSTAB"], (
        f"source mount configuration changed before {tool}")
    with open(os.environ["CIPROXY_TEST_EVENTS"], "a") as events:
        events.write(json.dumps({"tool": tool, "source_fstab": fstab}) + "\n")
    if tool == "guestfish":
        subprocess.run(["tar", "--numeric-owner", "--xattrs", "--acls", "--sparse",
                        "-cf", "-", "-C", str(source), "."], check=True)


if __name__ == "__main__":
    main()
