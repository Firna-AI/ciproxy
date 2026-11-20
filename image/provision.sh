#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
mode=${1:-machine}
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
if [[ $mode == boat ]]; then
    install -m 0644 "$definition/ciproxy-boat-start.service" \
        "$definition/ciproxy-boat-guest.service" /etc/systemd/system/
    printf 'IMAGE_ID=%s\n' "$IMAGE_ID" > /etc/ciproxy/guest.env
else
    install -m 0644 /tmp/vmlinux-ciproxy /boot/vmlinux-ciproxy
    install -m 0644 "$definition/ciproxy-guest.service" /etc/systemd/system/
    printf 'IMAGE_ID=%s\nVSOCK_PORT=5252\n' "$IMAGE_ID" > /etc/ciproxy/guest.env
fi
printf '%s\n' "$IMAGE_ID" > /etc/ciproxy/image-id
if [[ $mode == boat ]]; then
    install -m 0755 "$definition/boat-run.sh" "$definition/boat-start.sh" "$definition/measure.py" \
        "$definition/metadata-smoke.py" /opt/ciproxy-image/
    install -m 0644 "$definition/LICENSE" /opt/ciproxy-image/LICENSE
else
    find "$definition" -mindepth 1 -maxdepth 1 ! -name boat-run.sh ! -name boat-start.sh \
        -exec cp -a {} /opt/ciproxy-image/ \;
fi
find /opt/post-generation -maxdepth 1 -type f -name '*.sh' -exec bash {} \;
rm -f /etc/apt/preferences.d/ciproxy-software
if [[ $mode == boat ]]; then
    usermod --append --groups sudo,docker runner
    rm -f /tmp/ciproxy-runner.tar.gz /tmp/ciproxy-guest \
        /tmp/ciproxy-cache-sidecar /tmp/ciproxy-worker.dll
    exit 0
fi
bash "$definition/boat-build-clean-image.sh"
systemctl enable docker ciproxy-guest
systemctl disable walinuxagent cloud-init cloud-config cloud-final || true
rm -f /etc/netplan/50-cloud-init.yaml
install -d /etc/systemd/network
cat > /etc/systemd/network/10-ciproxy.network <<'EOF'
[Match]
Name=eth0
[Network]
DHCP=no
KeepConfiguration=yes
DNS=1.1.1.1
EOF
systemctl enable systemd-networkd systemd-resolved
rm -f /tmp/ciproxy-runner.tar.gz /tmp/ciproxy-guest /tmp/vmlinux-ciproxy \
    /tmp/ciproxy-cache-sidecar /tmp/ciproxy-worker.dll
