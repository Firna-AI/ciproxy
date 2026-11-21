#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

registration=${1:-}
runtime=/run/ciproxy
install -d -o root -g root -m 0700 "$runtime"
if [[ -n $registration ]]; then
    install -o root -g root -m 0600 "$registration" "$runtime/registration.json"
    rm -f "$registration"
else
    rm -f "$runtime/registration.json"
fi

systemctl daemon-reload
systemctl start --no-block ciproxy-boat-start.service
