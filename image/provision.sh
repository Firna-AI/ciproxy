#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
install -d /etc/ciproxy /opt/actions-runner /usr/local/libexec /opt/ciproxy-image
id runner >/dev/null 2>&1 || useradd --create-home --shell /bin/bash --groups sudo,docker runner
printf 'runner ALL=(ALL) NOPASSWD:ALL\n' > /etc/sudoers.d/ciproxy-runner
chmod 0440 /etc/sudoers.d/ciproxy-runner
printf '%s  /tmp/ciproxy-runner.tar.gz\n' "$RUNNER_SHA256" | sha256sum --check
tar -xzf /tmp/ciproxy-runner.tar.gz -C /opt/actions-runner
install -m 0644 /opt/actions-runner/bin/Runner.Worker.dll /opt/actions-runner/bin/Runner.Worker.stock.dll
install -m 0644 /tmp/ciproxy-worker.dll /opt/actions-runner/bin/Runner.Worker.own.dll
chown -R runner:runner /opt/actions-runner
install -m 0755 /tmp/ciproxy-guest /usr/local/bin/ciproxy-guest
install -m 0755 /tmp/ciproxy-cache-sidecar /usr/local/bin/ciproxy-cache-sidecar
install -m 0755 "$definition/runner-entry.sh" /usr/local/libexec/ciproxy-runner
install -m 0755 "$definition/docker-attach.sh" /usr/local/libexec/ciproxy-docker-attach
install -m 0755 "$definition/runner-patch/container-proxy-url.sh" /usr/local/libexec/ciproxy-container-proxy-url
install -m 0644 /tmp/vmlinux-ciproxy /boot/vmlinux-ciproxy
install -m 0644 "$definition/ciproxy-guest.service" /etc/systemd/system/
printf '%s\n' "$IMAGE_ID" > /etc/ciproxy/image-id
printf 'IMAGE_ID=%s\nVSOCK_PORT=5252\n' "$IMAGE_ID" > /etc/ciproxy/guest.env
cp -a "$definition/." /opt/ciproxy-image/
find /opt/post-generation -maxdepth 1 -type f -name '*.sh' -exec bash {} \;
systemctl enable docker ciproxy-guest
systemctl disable walinuxagent cloud-init cloud-config cloud-final || true
rm -f /etc/netplan/50-cloud-init.yaml
install -d /etc/systemd/network
cat > /etc/systemd/network/10-ciproxy.network <<'EOF'
[Match]
Name=eth0
[Network]
DHCP=yes
KeepConfiguration=static
DNS=1.1.1.1
EOF
systemctl enable systemd-networkd systemd-resolved
rm -f /tmp/ciproxy-runner.tar.gz /tmp/ciproxy-guest /tmp/vmlinux-ciproxy \
    /tmp/ciproxy-cache-sidecar /tmp/ciproxy-worker.dll
