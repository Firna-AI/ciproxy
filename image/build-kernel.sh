#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
work_dir=$(realpath "${1:?Usage: image/build-kernel.sh WORK_DIR}")
kernel_dir="$work_dir/linux-$KERNEL_VERSION"
if [[ -z ${SOURCE_DATE_EPOCH:-} ]]; then
    SOURCE_DATE_EPOCH=$(git -C "$definition" show -s --format=%ct HEAD 2>/dev/null || true)
fi
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-0}
export KBUILD_BUILD_USER=builder
export KBUILD_BUILD_HOST=localhost
export KBUILD_BUILD_VERSION=1
KBUILD_BUILD_TIMESTAMP=$(LC_ALL=C date -u --date="@$SOURCE_DATE_EPOCH" '+%a %b %e %T UTC %Y')
export KBUILD_BUILD_TIMESTAMP
cp "$work_dir/kernel.config" "$kernel_dir/.config"
make -C "$kernel_dir" olddefconfig
make -C "$kernel_dir" -j "${BUILD_CPUS:-4}" vmlinux
cp "$kernel_dir/vmlinux" "$work_dir/vmlinux"
sha256sum "$work_dir/vmlinux"
