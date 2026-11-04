#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
work_dir=${1:?Usage: image/fetch.sh WORK_DIR}
mkdir -p "$work_dir"

fetch() {
    local url=$1 checksum=$2 destination=$3
    if [[ ! -f "$destination" ]]; then
        curl --fail --location --retry 3 "$url" --output "$destination"
    fi
    printf '%s  %s\n' "$checksum" "$destination" | sha256sum --check
}

fetch "$RUNNER_IMAGES_URL" "$RUNNER_IMAGES_SHA256" "$work_dir/runner-images.tar.gz"
fetch "$SOFTWARE_MANIFEST_URL" "$SOFTWARE_MANIFEST_SHA256" "$work_dir/upstream-software.json"
fetch "$UBUNTU_URL" "$UBUNTU_SHA256" "$work_dir/ubuntu.qcow2"
fetch "$RUNNER_URL" "$RUNNER_SHA256" "$work_dir/runner.tar.gz"
fetch "$KERNEL_URL" "$KERNEL_SHA256" "$work_dir/linux.tar.xz"
fetch "$KERNEL_CONFIG_URL" "$KERNEL_CONFIG_SHA256" "$work_dir/kernel.config"
tar -xzf "$work_dir/runner-images.tar.gz" -C "$work_dir"
tar -xf "$work_dir/linux.tar.xz" -C "$work_dir"
