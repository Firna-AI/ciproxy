#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Print the guest facts of the measurement as one JSON line. Run over SSH.
set -euo pipefail

# The data volume shares the APFS container, so its available bytes are the
# free job bytes of the image.
read -r used_kib free_kib < <(df -k /System/Volumes/Data | awk 'NR == 2 {print $3, $4}')
printf '{"macos_version":"%s","macos_build":"%s",' \
    "$(sw_vers -productVersion)" "$(sw_vers -buildVersion)"
printf '"data_volume_used_bytes":%s,"free_job_bytes":%s,' \
    "$((used_kib * 1024))" "$((free_kib * 1024))"
printf '"sip":"%s","xcode":"%s"}\n' \
    "$(csrutil status | sed 's/.*: //; s/\.$//')" \
    "$( (xcodebuild -version 2>/dev/null || echo none) | tr '\n' ' ' | sed 's/ $//')"
