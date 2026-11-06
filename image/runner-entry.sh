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
if [[ -n "${CIPROXY_ASSIGNMENT_SOCKET:-}" ]]; then
    cp bin/Runner.Worker.own.dll bin/Runner.Worker.dll
    ./bin/Runner.Worker ciproxy-handoff-protocol-2
else
    cp bin/Runner.Worker.stock.dll bin/Runner.Worker.dll
fi
: "${ACTIONS_RUNNER_INPUT_JITCONFIG:?JIT registration required}"
exec ./bin/Runner.Listener run --once
