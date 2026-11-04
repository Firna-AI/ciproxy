#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

artifacts=$(realpath "${1:?Usage: image/publish.sh ARTIFACT_DIR gs://BUCKET/PREFIX}")
destination=${2:?Cloud Storage release prefix required}
(
    cd "$artifacts"
    sha256sum --check SHA256SUMS
)
gcloud storage cp \
    "$artifacts/rootfs.ext4" "$artifacts/rootfs.tar.zst" \
    "$artifacts/vmlinux" "$artifacts/upstream-software.json" \
    "$artifacts/software-report.json" \
    "$artifacts/firecracker.tar.zst" "$artifacts/firecracker-content.sha256" \
    "$artifacts/downloads.json" \
    "$artifacts/manifest.json" "$artifacts/measurement.json" \
    "$artifacts/SHA256SUMS" "$destination/"
