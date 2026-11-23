#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -u

proc_root=${CIPROXY_PROC_ROOT:-/proc}
sys_root=${CIPROXY_SYS_ROOT:-/sys}

warn() {
    printf 'Warning: %s\n' "$1" >&2
}

# Bash calls this function from the EXIT trap.
# shellcheck disable=SC2317
cleanup_swap() {
    if (( swap_enabled )) && ! swapoff "$device" >/dev/null; then
        warn "Could not turn off swap on $device after setup failed."
        return
    fi
    if ! zramctl --reset "$device"; then
        warn "Could not reset $device after setup failed."
    fi
}

zram_available() {
    [[ -d $sys_root/class/zram-control || -d $sys_root/block/zram0 ]]
}

if ! active=$(awk '$1 ~ /^\/dev\/zram[0-9]+$/ { print $1; exit }' "$proc_root/swaps"); then
    warn 'Could not read active swaps.'
    exit 0
fi
if [[ -n $active ]]; then
    exit 0
fi

if ! zram_available; then
    if ! command -v modprobe >/dev/null || ! modprobe zram >/dev/null || ! zram_available; then
        warn 'zram is unavailable. Skipping swap setup.'
        exit 0
    fi
fi

if ! memory_kib=$(awk '$1 == "MemTotal:" { print $2; exit }' "$proc_root/meminfo") ||
    [[ ! $memory_kib =~ ^[1-9][0-9]*$ ]]; then
    warn 'Could not read MemTotal for zram swap.'
    exit 0
fi
cap_kib=$((8 * 1024 * 1024))
if (( memory_kib > cap_kib )); then
    memory_kib=$cap_kib
fi
size_bytes=$((memory_kib * 1024))

swap_enabled=0
if ! device=$(zramctl --find --algorithm zstd --size "$size_bytes"); then
    warn 'Could not configure zram with zstd. Trying the kernel default compressor.'
    if ! device=$(zramctl --find --size "$size_bytes"); then
        warn 'Could not configure zram swap.'
        exit 0
    fi
fi
trap cleanup_swap EXIT

if ! mkswap "$device" >/dev/null; then
    warn "Could not format $device as swap."
    exit 0
fi
if ! swapon --priority 100 "$device" >/dev/null; then
    warn "Could not enable swap on $device."
    exit 0
fi
swap_enabled=1
trap - EXIT

# The read-ahead setting only tunes the swap, so its failure keeps the swap.
if ! printf '0\n' 2>/dev/null > "$proc_root/sys/vm/page-cluster"; then
    warn 'Could not set vm.page-cluster to 0.'
fi
exit 0
