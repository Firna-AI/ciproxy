#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

root=${1:-}
rm -rf -- "$root/imagegeneration/helpers/github-cache" \
    "$root/tmp/ciproxy-github-cache"
rm -f -- "$root/imagegeneration/helpers/ciproxy-pins.json" \
    "$root/etc/apt/preferences.d/ciproxy-software" \
    "$root/etc/apt/apt.conf.d/90ciproxy-build" \
    "$root/etc/ciproxy/provider-docker-gid"
