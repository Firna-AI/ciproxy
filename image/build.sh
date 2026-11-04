#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repository=$(dirname "$definition")
source "$definition/pins.env"
work_dir=${1:?Usage: image/build.sh WORK_DIR [prepare-only]}
mkdir -p "$work_dir"
work_dir=$(realpath "$work_dir")
export PACKER_PLUGIN_PATH="$work_dir/packer-plugins"
export PACKER_CACHE_DIR="$work_dir/packer-cache"

bash "$definition/fetch.sh" "$work_dir"
mkdir -p "$work_dir/seed"
if [[ ! -f "$work_dir/build-key" ]]; then
    ssh-keygen -q -t ed25519 -N '' -f "$work_dir/build-key"
fi
cat > "$work_dir/seed/user-data" <<EOF
#cloud-config
users:
  - name: packer
    groups: [sudo]
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    ssh_authorized_keys:
      - $(cat "$work_dir/build-key.pub")
EOF
printf 'instance-id: ciproxy-image-build\n' > "$work_dir/seed/meta-data"
printf '%s  ubuntu.qcow2\n' "$UBUNTU_SHA256" > "$work_dir/ubuntu.sha256"
python3 "$definition/prepare-template.py" \
    "$work_dir/runner-images-$RUNNER_IMAGES_COMMIT" "$work_dir/packer"
if [[ ${2:-} == prepare-only ]]; then
    exit 0
fi

bash "$definition/build-kernel.sh" "$work_dir"
cargo build --manifest-path "$repository/Cargo.toml" --release --package ciproxy-guest
packer init "$work_dir/packer"
packer build -only 'ubuntu-24_04.qemu.image' \
    -var "work_dir=$work_dir" \
    -var "guest_binary=$repository/target/release/ciproxy-guest" \
    -var "image_definition=$definition" \
    -var image_os=ubuntu24 -var image_version=20261101.360.1 \
    "$work_dir/packer"
bash "$definition/export.sh" "$work_dir"
