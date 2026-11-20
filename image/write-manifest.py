#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Write release metadata from the measured disk and pinned source inputs."""

import argparse
import hashlib
import json
import os
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifacts", type=Path)
    parser.add_argument("rootfs", type=Path)
    args = parser.parse_args()
    definition = Path(__file__).resolve().parent
    pins = dict(line.split("=", 1) for line in (definition / "pins.env").read_text().splitlines()
                if line and not line.startswith("#"))
    measurement = json.loads((args.artifacts / "measurement.json").read_text())
    guest = args.rootfs / "usr/local/bin/ciproxy-guest"
    manifest = {"image_id": os.environ.get("IMAGE_RELEASE_ID", pins["IMAGE_ID"]),
                "inputs": pins, "root_disk_bytes": measurement["root_disk_bytes"],
                "guest_sha256": hashlib.sha256(guest.read_bytes()).hexdigest(),
                "measurements": measurement}
    (args.artifacts / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
