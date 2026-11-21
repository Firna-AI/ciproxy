#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

operation=${1:?Usage: rootfs-archive.sh export DISK ARCHIVE or extract ARCHIVE DIRECTORY}
source_path=${2:?Source disk or archive required}
destination=${3:?Destination archive or directory required}
tar_metadata=(--numeric-owner --xattrs --xattrs-include='*' --acls --sparse)

case "$operation" in
    export)
        guestfish --ro -a "$source_path" -i \
            tar-out / - numericowner:true xattrs:true acls:true | \
            zstd -T0 -o "$destination"
        ;;
    extract)
        tar "${tar_metadata[@]}" -I zstd -xf "$source_path" -C "$destination"
        ;;
    *)
        printf 'Use export DISK ARCHIVE or extract ARCHIVE DIRECTORY.\n' >&2
        exit 2
        ;;
esac
