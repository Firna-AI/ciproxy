#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -eo pipefail

source /etc/profile
if [[ -f /home/runner/.profile ]]; then
    source /home/runner/.profile
fi
set -a
source /etc/environment
set +a
cd /opt/actions-runner
exec ./bin/Runner.Listener run --once --jitconfig "${1:?JIT registration required}"
