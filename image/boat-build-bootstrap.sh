#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
install -d /etc/ciproxy
if [[ ! -s /etc/ciproxy/provider-docker-gid ]]; then
    getent group docker | cut -d: -f3 > /etc/ciproxy/provider-docker-gid
fi
if [[ ! -f /opt/ciproxy-build/provider-permissions.json ]]; then
    python3 "$definition/boat-build-permissions.py" --capture
fi
export DEBIAN_FRONTEND=noninteractive
cat > /etc/apt/apt.conf.d/90ciproxy-build <<'EOF'
APT::Get::Assume-Yes "true";
APT::Get::allow-downgrades "true";
Dpkg::Options { "--force-confdef"; "--force-confold"; };
EOF
apt-get update --allow-releaseinfo-change-label
python3 "$definition/boat-build-developer.py"
apt-get install -y ca-certificates curl jq gnupg lsb-release \
    software-properties-common unzip python3
if [[ -f /etc/apt/sources.list.d/ubuntu.sources ]]; then
    python3 "$definition/boat-build-sources.py" \
        /etc/apt/sources.list.d/ubuntu.sources
fi
install -d /etc/cloud/templates
printf 'Native Boat bootstrap ready. Provider services remain active.\n'
