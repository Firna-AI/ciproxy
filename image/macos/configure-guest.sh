#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Settings of the base guest before the upstream recipe. Run as root over SSH.
# The upstream recipe sets auto-login, power, and update settings again later.
set -euo pipefail

# macOS refuses the name UTC. GMT is the same zone.
systemsetup -settimezone GMT >/dev/null
pmset -a sleep 0 displaysleep 0 disksleep 0
# Remote Login stays on after a restart.
launchctl enable system/com.openssh.sshd
csrutil status
fdesetup status
sw_vers
