#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
archive=${1:?Usage: boat-prepare.sh ROOTFS_ARCHIVE SHA256}
checksum=${2:?SHA-256 of rootfs.tar.zst required}
printf '%s  %s\n' "$checksum" "$archive" | sha256sum --check
if ! command -v zstd >/dev/null; then
    apt-get update
    apt-get install -y zstd
fi
install -d /opt/ciproxy/rootfs
bash "$definition/rootfs-archive.sh" extract "$archive" /opt/ciproxy/rootfs
rm -f "$archive"
python3 /opt/ciproxy/rootfs/opt/ciproxy-image/measure.py /opt/ciproxy/rootfs --job
printf 'The full rootfs is ready for a credential-free saved snapshot.\n'
