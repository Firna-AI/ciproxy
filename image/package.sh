#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

artifacts=$(realpath "${1:?Usage: image/package.sh ARTIFACT_DIR}")
cd "$artifacts"
sha256sum rootfs.ext4 vmlinux upstream-software.json software-report.json \
    measurement.json manifest.json > firecracker-content.sha256
tar --sparse -I 'zstd -T0' -cf firecracker.tar.zst \
    rootfs.ext4 vmlinux upstream-software.json software-report.json \
    measurement.json manifest.json firecracker-content.sha256
python3 - <<'PY'
import hashlib, json, pathlib
downloads = {}
for target, name in [("firecracker", "firecracker.tar.zst"), ("boat", "rootfs.tar.zst")]:
    path = pathlib.Path(name)
    checksum = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            checksum.update(block)
    downloads[target] = {"file": name, "sha256": checksum.hexdigest(),
                         "size_bytes": path.stat().st_size}
pathlib.Path("downloads.json").write_text(json.dumps(downloads, indent=2) + "\n")
PY
sha256sum rootfs.ext4 rootfs.tar.zst vmlinux upstream-software.json software-report.json \
    measurement.json manifest.json firecracker.tar.zst firecracker-content.sha256 \
    downloads.json > SHA256SUMS
