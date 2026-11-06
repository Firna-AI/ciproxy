#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# The trusted supervisor calls this before action steps, after VMM drive switch.
set -euo pipefail
required_free_bytes=${1:?Required free bytes are missing}

if mountpoint -q /var/lib/docker; then exit 0; fi
restore_root() {
    umount /var/lib/docker 2>/dev/null || true
    umount /mnt/ciproxy-docker 2>/dev/null || true
    systemctl start docker.socket docker.service || true
}
trap restore_root ERR
systemctl stop docker.socket docker.service
blockdev --rereadpt /dev/vdb || true
mkdir -p /mnt/ciproxy-docker
mount /dev/vdb /mnt/ciproxy-docker
if [[ ! -f /mnt/ciproxy-docker/.ciproxy-initialized ]]; then
    cp -a --reflink=auto /var/lib/docker/. /mnt/ciproxy-docker/
    touch /mnt/ciproxy-docker/.ciproxy-initialized
    sync -f /mnt/ciproxy-docker
fi
read -r free_blocks block_bytes < <(stat --file-system --format='%a %S' /mnt/ciproxy-docker)
(( free_blocks * block_bytes >= required_free_bytes ))
mount --move /mnt/ciproxy-docker /var/lib/docker
systemctl start docker.socket docker.service
systemctl is-active --quiet docker.service
trap - ERR
