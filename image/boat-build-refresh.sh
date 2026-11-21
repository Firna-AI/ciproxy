#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

archive=${1:?Usage: boat-build-refresh.sh NATIVE_BUILD_BUNDLE SHA256}
checksum=${2:?SHA-256 required}
if systemctl is-active --quiet ciproxy-image-build.service; then
    printf 'The native build still runs. Keep the staged bundle and wait for its unit to exit.\n' >&2
    exit 1
fi
printf '%s  %s\n' "$checksum" "$archive" | sha256sum --check
rm -rf /opt/ciproxy-build/github-answers \
    /opt/ciproxy-build/runner-images/images/ubuntu/scripts/helpers/github-cache
tar -xzf "$archive" -C /opt/ciproxy-build
rm -f "$archive"
systemd-run --unit=ciproxy-image-build --collect \
    /usr/bin/python3 /opt/ciproxy-build/definition/boat-build-execute.py /opt/ciproxy-build --resume
printf 'Refreshed native build inputs. Resuming from recorded results.\n'
