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
zram_options=(ZRAM ZRAM_BACKEND_ZSTD ZRAM_DEF_COMP_ZSTD)
config_args=()
for option in "${zram_options[@]}"; do
    config_args+=(--enable "$option")
done
"$kernel_dir/scripts/config" --file "$kernel_dir/.config" "${config_args[@]}"
make -C "$kernel_dir" olddefconfig
for option in "${zram_options[@]}"; do
    if ! grep -Fqx "CONFIG_${option}=y" "$kernel_dir/.config"; then
        printf 'Error: CONFIG_%s must be set to y after olddefconfig.\n' "$option" >&2
        exit 1
    fi
done
make -C "$kernel_dir" -j "${BUILD_CPUS:-4}" vmlinux
cp "$kernel_dir/vmlinux" "$work_dir/vmlinux"
sha256sum "$work_dir/vmlinux"
