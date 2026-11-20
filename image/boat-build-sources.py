#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Prepare known Ubuntu archive URIs for upstream mirror-list setup."""

import argparse
import pathlib
import re


def normalize(contents):
    origin = re.compile(r"(?<!\S)(?:https?://archive[.]ubuntu[.]com/ubuntu/?|"
                        r"mirror\+file:/etc/apt/boat-mirrors[.]txt)(?!\S)")
    return "".join(origin.sub("http://azure.archive.ubuntu.com/ubuntu/", line)
                   if re.match(r"\s*URIs:", line) else line
                   for line in contents.splitlines(keepends=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sources", type=pathlib.Path)
    args = parser.parse_args()
    args.sources.write_text(normalize(args.sources.read_text()))


if __name__ == "__main__":
    main()
