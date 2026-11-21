#!/usr/bin/env python3
"""Select the image deployed by the exact successful upstream preview."""

import argparse
import os
import subprocess
import sys

from image_outputs import outputs
from image_registry import immutable
from promotion_event import check_checkout, validate
from workload_snapshot import evidence, load


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("preview-evidence", "promote"))
    args = parser.parse_args()
    try:
        source = os.environ.get("CIPROXY_SOURCE_SHA", "")
        validate("production", source, os.environ)
        check_checkout(source)
        if args.command == "preview-evidence":
            outputs(evidence("main-preview", "production"))
        else:
            image = load("main-preview", "production").get("control_plane_image", "")
            if not image:
                print("Preview records no deployed control-plane image. A disabled preview runtime "
                      "cannot supply an image to an enabled production runtime.", file=sys.stderr)
            outputs({"image": immutable(image) if image else ""})
    except ValueError as error:
        print(f"Image promotion: {error}", file=sys.stderr)
        return 1
    except (KeyError, OSError, subprocess.CalledProcessError):
        print("Image promotion failed. Check the source and bound preview evidence.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
