#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

systemctl daemon-reload
systemctl stop docker.service docker.socket
systemctl restart containerd.service
if [[ -f /etc/ciproxy/boat-restart-services ]]; then
    while IFS= read -r unit; do
        case "$unit" in
            ''|\#*|docker.service|docker.socket|containerd.service) continue ;;
        esac
        systemctl try-restart "$unit"
    done < /etc/ciproxy/boat-restart-services
fi
systemctl start docker.service docker.socket
python3 /opt/ciproxy-image/measure.py / --job
systemctl start ciproxy-boat-guest.service
