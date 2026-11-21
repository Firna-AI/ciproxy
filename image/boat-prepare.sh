#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

archive=${1:?Usage: boat-prepare.sh NATIVE_BUILD_BUNDLE SHA256}
checksum=${2:?SHA-256 of the native build bundle required}
printf '%s  %s\n' "$checksum" "$archive" | sha256sum --check
install -d /opt/ciproxy-build
tar -xzf "$archive" -C /opt/ciproxy-build
rm -f "$archive"
systemd-run --unit=ciproxy-image-build --collect \
    /usr/bin/python3 /opt/ciproxy-build/definition/boat-build-execute.py /opt/ciproxy-build
printf 'Native build started. Read /opt/ciproxy-build/report.json for results.\n'
