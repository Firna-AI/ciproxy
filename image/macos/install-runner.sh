#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Install the stock runner in the guest. Run as root with RUNNER_SHA256 set.
set -euo pipefail

archive=${1:?Usage: install-runner.sh RUNNER_ARCHIVE}
: "${RUNNER_SHA256:?RUNNER_SHA256 is required}"
printf '%s  %s\n' "$RUNNER_SHA256" "$archive" | shasum -a 256 --check
rm -rf /opt/actions-runner
mkdir -p /opt/actions-runner
tar -xzf "$archive" -C /opt/actions-runner
chown -R runner:staff /opt/actions-runner
/opt/actions-runner/bin/Runner.Listener --version
