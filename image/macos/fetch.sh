#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Download and check the pinned inputs of the macOS image on the build Mac.
# LOCAL_RESTORE_IMAGE names a local copy of the restore image to clone instead
# of a download. XCODE_DIR names the folder with the Xcode archives, which an
# Apple Account downloads. GUEST_DIR names the folder of package-guest.sh.
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
work_dir=${1:?Usage: image/macos/fetch.sh WORK_DIR}
inputs="$work_dir/inputs"
mkdir -p "$inputs"

check() {
    printf '%s  %s\n' "$1" "$2" | shasum -a 256 -c
}

fetch() {
    local url=$1 checksum=$2 destination=$3
    if [[ ! -f "$destination" ]]; then
        # A broken download resumes from its .part file, also in a later run.
        local attempt
        for attempt in 1 2 3 4 5; do
            curl --fail --location --retry 3 --continue-at - "$url" \
                --output "$destination.part" && break
            [[ $attempt == 5 ]] && { echo "the download of $url failed" >&2; return 1; }
            sleep 10
        done
        mv "$destination.part" "$destination"
    fi
    check "$checksum" "$destination"
}

if [[ ! -f "$inputs/restore.ipsw" && -n "${LOCAL_RESTORE_IMAGE:-}" ]]; then
    cp -c "$LOCAL_RESTORE_IMAGE" "$inputs/restore.ipsw"
fi
fetch "$RESTORE_IMAGE_URL" "$RESTORE_IMAGE_SHA256" "$inputs/restore.ipsw"
fetch "$RUNNER_IMAGES_URL" "$RUNNER_IMAGES_SHA256" "$inputs/runner-images.tar.gz"
fetch "$SOFTWARE_MANIFEST_URL" "$SOFTWARE_MANIFEST_SHA256" "$inputs/upstream-software.json"
fetch "$RUNNER_URL" "$RUNNER_SHA256" "$inputs/runner.tar.gz"
fetch "$PACKER_URL" "$PACKER_SHA256" "$inputs/packer.zip"
unzip -oq "$inputs/packer.zip" packer -d "$inputs"
rm -rf "$work_dir/upstream"
mkdir -p "$work_dir/upstream"
tar -xzf "$inputs/runner-images.tar.gz" -C "$work_dir/upstream" --strip-components 1

: "${GUEST_DIR:?Set GUEST_DIR to the output folder of image/macos/package-guest.sh}"
# Xcode is optional here: the test image has none, and the provision phase
# stops when the archive is missing.
mkdir -p "$inputs/xcode"
# Apple names the download Xcode_VERSION_Universal.xip. The recipe asks for
# the upstream name, so the copy takes that name.
if [[ ! -f "$inputs/xcode/$XCODE_XIP_NAME" && -n "${XCODE_DIR:-}" ]]; then
    source_xip="$XCODE_DIR/$XCODE_XIP_NAME"
    [[ -f "$source_xip" ]] || source_xip="$XCODE_DIR/Xcode_${XCODE_VERSION}_Universal.xip"
    cp -c "$source_xip" "$inputs/xcode/$XCODE_XIP_NAME"
fi
if [[ -f "$inputs/xcode/$XCODE_XIP_NAME" ]]; then
    check "$XCODE_SHA256" "$inputs/xcode/$XCODE_XIP_NAME"
else
    echo "No Xcode archive: only the test image can be built. Set XCODE_DIR for a release."
fi
[[ -x "$GUEST_DIR/install.sh" ]] || { echo "no install.sh in $GUEST_DIR" >&2; exit 1; }
rm -rf "$inputs/guest"
cp -R "$GUEST_DIR" "$inputs/guest"
