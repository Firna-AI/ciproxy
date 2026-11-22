#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Settings of the base guest before the upstream recipe. Run as root over SSH.
# The upstream recipe sets auto-login, power, and update settings again later.
set -euo pipefail

# macOS refuses the name UTC. GMT is the same zone. systemsetup fails with
# "Error:-99" over SSH, so the build sets the zone link itself. macOS reads
# the zone from the /etc/localtime link.
systemsetup -settimezone GMT >/dev/null 2>&1 || true
zoneinfo=/var/db/timezone/zoneinfo
[[ -e $zoneinfo/GMT ]] || zoneinfo=/usr/share/zoneinfo
[[ -e $zoneinfo/GMT ]] || { echo "no GMT zone file in the guest" >&2; exit 1; }
ln -sf "$zoneinfo/GMT" /etc/localtime
zone=$(readlink /etc/localtime || true)
if [[ $zone != */GMT || $(date +%Z) != GMT ]]; then
    echo "the time zone is not GMT: link '$zone', date says $(date +%Z)" >&2
    ls -l /etc/localtime >&2 || true
    exit 1
fi
echo "time zone: $zone"
pmset -a sleep 0 displaysleep 0 disksleep 0
# Remote Login stays on after a restart.
launchctl enable system/com.openssh.sshd
csrutil status
fdesetup status
sw_vers
