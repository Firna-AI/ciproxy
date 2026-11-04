#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
work_dir=$(realpath "${1:?Usage: image/build-kernel.sh WORK_DIR}")
kernel_dir="$work_dir/linux-$KERNEL_VERSION"
cp "$work_dir/kernel.config" "$kernel_dir/.config"
make -C "$kernel_dir" olddefconfig
make -C "$kernel_dir" -j "${BUILD_CPUS:-4}" vmlinux
cp "$kernel_dir/vmlinux" "$work_dir/vmlinux"
sha256sum "$work_dir/vmlinux"
