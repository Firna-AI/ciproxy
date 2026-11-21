#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Print a real rootfs file's privilege metadata for snapshot smoke evidence."""

import argparse
import errno
import json
import os
import pathlib
import stat


def attribute(path, name):
    try:
        return os.getxattr(path, name).hex()
    except OSError as error:
        if error.errno == errno.ENODATA:
            return None
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rootfs", type=pathlib.Path)
    parser.add_argument("file", help="Relative path of a real file, such as usr/bin/ping")
    args = parser.parse_args()
    path = args.rootfs / args.file
    metadata = path.stat()
    print(json.dumps({
        "file": args.file,
        "uid": metadata.st_uid,
        "gid": metadata.st_gid,
        "mode": stat.S_IMODE(metadata.st_mode),
        "security_capability": attribute(path, "security.capability"),
        "posix_acl_access": attribute(path, "system.posix_acl_access"),
        "posix_acl_default": attribute(path, "system.posix_acl_default"),
    }, indent=2))


if __name__ == "__main__":
    main()
