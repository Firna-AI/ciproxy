#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Bundle a measured macOS image: compressed disk, VM files, reports, checksums.
# Run on the build Mac after write-manifest.py. ARTIFACT_DIR/bundle holds the
# VM files that ciproxy-vz reads. macos.tar is not compressed as a whole. Only
# the disk is, as the member bundle/disk.img.zst, so its holes survive:
# extract it to a sparse bundle/disk.img and write each zero block as a hole.
# The disk is compressed straight into the archive, so the build needs free
# space for the archive only.
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

artifacts=$(cd "${1:?Usage: image/macos/package.sh ARTIFACT_DIR}" && pwd)
cd "$artifacts"
command -v zstd >/dev/null || { echo "zstd is missing. Install it first." >&2; exit 1; }
vm_files=(bundle/aux.img bundle/hardware-model.bin)
reports=(upstream-software.json software-report.json measurement.json manifest.json)
# The agent accepts only plain regular members. bsdtar writes sparse members
# and extended attributes, so a script writes the archive and macos-content.sha256,
# then checks the headers and decompresses the disk member once.
python3 "$definition/macos_archive.py" pack macos.tar --folder .
python3 - <<'PY'
import hashlib, json, pathlib
path = pathlib.Path("macos.tar")
checksum = hashlib.sha256()
with path.open("rb") as source:
    for block in iter(lambda: source.read(1024 * 1024), b""):
        checksum.update(block)
downloads = {"macos": {"file": path.name, "sha256": checksum.hexdigest(),
                       "size_bytes": path.stat().st_size}}
pathlib.Path("downloads.json").write_text(json.dumps(downloads, indent=2) + "\n")
PY
printf 'macos.tar: %s bytes\n' "$(stat -f %z macos.tar)" >> size-report.txt
# The disk's sum comes from macos-content.sha256, so the disk is not read again.
{
    grep '  bundle/disk.img$' macos-content.sha256
    shasum -a 256 "${vm_files[@]}" "${reports[@]}" macos.tar macos-content.sha256 \
        downloads.json size-report.txt
} > SHA256SUMS
cat size-report.txt
