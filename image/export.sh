#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
work_dir=$(realpath "${1:?Usage: image/export.sh WORK_DIR}")
disk="$work_dir/qemu-output/ciproxy.qcow2"
artifacts="$work_dir/artifacts"
rootfs="$work_dir/rootfs"
source_archive="$work_dir/source-rootfs.tar.zst"
mkdir -p "$artifacts" "$rootfs"

virt-customize -a "$disk" \
    --delete /home/packer/.ssh --delete /root/.ssh \
    --delete /var/lib/cloud --delete /tmp/ciproxy-image \
    --run-command 'rm -f /etc/ssh/ssh_host_* /etc/machine-id /var/lib/dbus/machine-id; touch /etc/machine-id; rm -rf /var/log/*'
bash "$definition/rootfs-archive.sh" export "$disk" "$source_archive"
sudo bash "$definition/rootfs-archive.sh" extract "$source_archive" "$rootfs"
printf '/dev/vda / ext4 defaults 0 1\n' | sudo tee "$rootfs/etc/fstab" > /dev/null
sudo bash "$definition/rootfs-archive.sh" pack "$rootfs" "$artifacts/rootfs.tar.zst"
sudo python3 "$definition/measure.py" "$rootfs" > "$work_dir/installed-size.json"

truncate -s "${ROOTFS_BYTES:-80000000000}" "$artifacts/rootfs.ext4"
sudo mkfs.ext4 -F -m 1 -d "$rootfs" -L ciproxy-root "$artifacts/rootfs.ext4"
cp "$work_dir/vmlinux" "$artifacts/vmlinux"
cp "$work_dir/upstream-software.json" "$artifacts/upstream-software.json"
cp "$work_dir/runner-images-$RUNNER_IMAGES_COMMIT/images/ubuntu/software-report.json" \
    "$artifacts/software-report.json"
sudo python3 "$definition/measure.py" "$rootfs" --artifacts "$artifacts" \
    > "$artifacts/measurement.json"
IMAGE_RELEASE_ID="$IMAGE_ID" python3 - "$artifacts" "$definition" "$rootfs" <<'PY'
import hashlib, json, os, pathlib, sys
directory = pathlib.Path(sys.argv[1])
definition = pathlib.Path(sys.argv[2])
rootfs = pathlib.Path(sys.argv[3])
pins = dict(line.split("=", 1) for line in (definition / "pins.env").read_text().splitlines()
            if line and not line.startswith("#"))
guest_digest = hashlib.sha256((rootfs / "usr/local/bin/ciproxy-guest").read_bytes()).hexdigest()
manifest = {"image_id": os.environ["IMAGE_RELEASE_ID"], "inputs": pins,
            "guest_sha256": guest_digest, "measurements":
            json.loads((directory / "measurement.json").read_text())}
(directory / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
PY
bash "$definition/package.sh" "$artifacts"
