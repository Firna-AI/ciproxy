#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Shrink the guest's disk layout to fit a disk of DISK_BYTES. Run as root over
# SSH. Then turn the guest off, and cut disk.img to DISK_BYTES on the host.
# Prints one JSON line. Changes nothing when the used space does not fit.
#
# The Recovery container sits at the end of the disk, after the main APFS
# container, so it is removed. The image then has no paired Recovery. The
# backup GPT is lost with the cut. The first start after the cut runs
# "diskutil repairDisk disk0", which writes it at the new end.
set -euo pipefail

disk_bytes=${1:?Usage: shrink-disk.sh DISK_BYTES}
margin=2000000000
plist=$(mktemp)
value() {
    plutil -extract "$1" raw -o - "$plist"
}

# The update leaves its download and a local snapshot, which hold space.
rm -rf /Library/Updates/* 2>/dev/null || true
tmutil thinlocalsnapshots / 999999999999 4 >&2 || true

diskutil list -plist disk0 > "$plist"
count=$(value AllDisksAndPartitions.0.Partitions)
main="" recovery=""
for index in $(seq 0 $((count - 1))); do
    content=$(value "AllDisksAndPartitions.0.Partitions.$index.Content")
    identifier=$(value "AllDisksAndPartitions.0.Partitions.$index.DeviceIdentifier")
    case $content in
        Apple_APFS) main=$identifier ;;
        Apple_APFS_Recovery) recovery=$identifier ;;
    esac
done
[[ -n $main ]] || { echo "no main APFS container on disk0" >&2; exit 1; }

diskutil info -plist "$main" > "$plist"
offset=$(value PartitionMapPartitionOffset)
diskutil info -plist disk0 > "$plist"
block=$(value DeviceBlockSize)
# Keep 33 blocks for the backup GPT and 1 MiB of slack at the end.
target=$(( (disk_bytes - offset - 33 * block - 1048576) / 1048576 * 1048576 ))
# The partition is the container's physical store. The container itself has
# its own reference, so find it in the full list.
diskutil apfs list -plist > "$plist"
container=""
for index in $(seq 0 $(( $(value Containers) - 1 ))); do
    stores=$(value "Containers.$index.PhysicalStores")
    for store in $(seq 0 $((stores - 1))); do
        if [[ $(value "Containers.$index.PhysicalStores.$store.DeviceIdentifier") == "$main" ]]; then
            container=$index
        fi
    done
done
if [[ -z $container ]]; then
    echo "no APFS container has the physical store $main" >&2
    diskutil list >&2
    exit 1
fi
used=$(( $(value "Containers.$container.CapacityCeiling") - $(value "Containers.$container.CapacityFree") ))
if (( used + margin > target )); then
    echo "the container uses $used bytes, which does not fit $target bytes with a" \
        "margin of $margin. Choose a larger DISK_BYTES." >&2
    exit 1
fi

if [[ -n $recovery ]]; then
    diskutil eraseVolume free free "$recovery" >&2
fi
diskutil apfs resizeContainer "$main" "$target" >&2
diskutil info -plist "$main" > "$plist"
size=$(value TotalSize)
end=$(( offset + size ))
rm -f "$plist"
(( end + 33 * block <= disk_bytes )) || { echo "the container ends at $end" >&2; exit 1; }
printf '{"container":"%s","container_bytes":%s,"container_used_bytes":%s,' \
    "$main" "$size" "$used"
printf '"recovery_removed":%s,"disk_bytes":%s}\n' \
    "$([[ -n $recovery ]] && echo true || echo false)" "$disk_bytes"
