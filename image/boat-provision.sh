#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
id sandbox-workload >/dev/null 2>&1 || useradd --system --user-group \
    --no-create-home --home-dir /home/sandbox-workload \
    --shell /usr/sbin/nologin sandbox-workload
install -d -o sandbox-workload -g sandbox-workload -m 0755 /home/sandbox-workload
bash "$definition/provision.sh" boat
systemctl disable ciproxy-boat-start.service ciproxy-boat-guest.service
systemctl daemon-reload
bash "$definition/boat-build-locale.sh"
python3 "$definition/boat-build-nvm.py"
