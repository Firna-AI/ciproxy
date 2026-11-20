#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

python3 - <<'PY'
import json
import pathlib
report = json.loads(pathlib.Path('/opt/ciproxy-build/report.json').read_text())
if report['status'] != 'passed':
    raise SystemExit('The native build is not complete. Keep its files for resume.')
PY
rm -f /etc/apt/apt.conf.d/90ciproxy-build /etc/apt/preferences.d/ciproxy-software
rm -f /etc/ciproxy/provider-docker-gid
rm -rf /opt/ciproxy-build /tmp/ciproxy-image
rm -f /tmp/ciproxy-native-build.tar.gz /tmp/ciproxy-native-refresh.tar.gz \
    /tmp/ciproxy-prepare.sh /tmp/ciproxy-refresh.sh \
    /tmp/ciproxy-runner.tar.gz /tmp/ciproxy-worker.dll \
    /tmp/ciproxy-guest /tmp/ciproxy-cache-sidecar
rm -f -- "${BASH_SOURCE[0]}"
printf 'Native build files removed. Runtime image and restart manifest remain.\n'
