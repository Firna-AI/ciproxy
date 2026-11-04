#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

registration=${1:?Usage: boat-run.sh PRIVATE_REGISTRATION_FILE}
rootfs=/opt/ciproxy/rootfs
if [[ ${CIPROXY_PRIVATE_NAMESPACE:-} != 1 ]]; then
    systemctl stop docker.service docker.socket containerd.service
    exec env CIPROXY_PRIVATE_NAMESPACE=1 unshare \
        --mount --pid --fork --kill-child bash "$0" "$registration"
fi

mount --make-rprivate /
mount -t proc proc "$rootfs/proc"
mount --rbind /dev "$rootfs/dev"
mount --make-rslave "$rootfs/dev"
mount --rbind /sys "$rootfs/sys"
mount --make-rslave "$rootfs/sys"
mount -t tmpfs tmpfs "$rootfs/run"
install -d -m 0700 "$rootfs/run/ciproxy"
install -m 0600 "$registration" "$rootfs/run/ciproxy/registration.json"
rm -f "$registration"
rm -f "$rootfs/etc/resolv.conf"
cp -L /etc/resolv.conf "$rootfs/etc/resolv.conf"
chroot "$rootfs" /usr/bin/dockerd \
    --host=unix:///run/docker.sock --exec-root=/run/docker \
    --pidfile=/run/docker.pid --bridge=ciproxy0 > /tmp/ciproxy-docker.log 2>&1 &
docker_pid=$!
trap 'kill "$docker_pid" 2>/dev/null || true' EXIT
ready=false
for attempt in {1..60}; do
    if chroot "$rootfs" /usr/bin/docker info > /dev/null 2>&1; then
        ready=true
        break
    fi
    sleep 1
done
if [[ $ready != true ]]; then
    printf 'The image Docker daemon did not become ready.\n' >&2
    exit 1
fi
python3 "$rootfs/opt/ciproxy-image/measure.py" "$rootfs" --job
image_id=$(cat "$rootfs/etc/ciproxy/image-id")
chroot "$rootfs" /usr/local/bin/ciproxy-guest --image-id "$image_id" \
    boat --registration-file /run/ciproxy/registration.json
