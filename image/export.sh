#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
work_dir=$(realpath "${1:?Usage: image/export.sh WORK_DIR [from-rootfs]}")
mode=${2:-from-source}
disk="$work_dir/qemu-output/ciproxy.qcow2"
artifacts="$work_dir/artifacts"
rootfs="$work_dir/rootfs"
source_archive="$work_dir/source-rootfs.tar.zst"
mkdir -p "$artifacts" "$rootfs"

if [[ $mode != from-rootfs ]]; then
    virt-customize -a "$disk" \
        --delete /home/packer/.ssh --delete /root/.ssh \
        --delete /var/lib/cloud --delete /tmp/ciproxy-image \
        --run-command 'rm -f /etc/ssh/ssh_host_* /etc/machine-id /var/lib/dbus/machine-id; touch /etc/machine-id; rm -rf /var/log/*'
    bash "$definition/rootfs-archive.sh" export "$disk" "$source_archive"
    sudo bash "$definition/rootfs-archive.sh" extract "$source_archive" "$rootfs"
fi
printf '/dev/vda / ext4 defaults 0 1\n' | sudo tee "$rootfs/etc/fstab" > /dev/null
sudo bash "$definition/rootfs-archive.sh" pack "$rootfs" "$artifacts/rootfs.tar.zst"
sudo python3 "$definition/measure.py" "$rootfs" > "$work_dir/installed-size.json"

sudo env ROOTFS_BYTES="${ROOTFS_BYTES:-}" python3 \
    "$definition/create-root-disk.py" "$rootfs" "$artifacts/rootfs.ext4" \
    --report "$work_dir/root-disk-sizing.json"
cp "$work_dir/vmlinux" "$artifacts/vmlinux"
cp "$work_dir/upstream-software.json" "$artifacts/upstream-software.json"
cp "$work_dir/runner-images-$RUNNER_IMAGES_COMMIT/images/ubuntu/software-report.json" \
    "$artifacts/software-report.json"
sudo python3 "$definition/measure.py" "$rootfs" --artifacts "$artifacts" \
    > "$artifacts/measurement.json"
IMAGE_RELEASE_ID="$IMAGE_ID" python3 "$definition/write-manifest.py" "$artifacts" "$rootfs"
bash "$definition/package.sh" "$artifacts"
