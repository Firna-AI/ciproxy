#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# Load the account's login environment and start the stock listener.
set -eo pipefail

source /etc/profile
if [[ -f "$HOME/.bash_profile" ]]; then
    source "$HOME/.bash_profile"
elif [[ -f "$HOME/.profile" ]]; then
    source "$HOME/.profile"
fi
cd /opt/actions-runner
if [[ -n "${CIPROXY_ASSIGNMENT_SOCKET:-}" ]]; then
    cp bin/Runner.Worker.own.dll bin/Runner.Worker.dll
    ./bin/Runner.Worker ciproxy-handoff-protocol-2
else
    cp bin/Runner.Worker.stock.dll bin/Runner.Worker.dll
fi
: "${ACTIONS_RUNNER_INPUT_JITCONFIG:?JIT registration required}"
exec ./bin/Runner.Listener run --once
