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
if ! python3 /opt/ciproxy-image/measure.py / --job; then
    printf '%s\n' 'Warning: Boat job disk measurement failed.' >&2
fi
if ! systemctl start ciproxy-swap.service; then
    printf '%s\n' 'Warning: Boat job swap setup failed.' >&2
fi
systemctl start ciproxy-boat-guest.service
