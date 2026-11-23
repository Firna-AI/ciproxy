#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# Install the ciproxy guest into a macOS job image. Run as root, after the
# stock runner is in /opt/actions-runner and the runner account exists.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
image_id=${1:?Usage: sudo ./install.sh IMAGE_ID}
account=${CIPROXY_RUNNER_ACCOUNT:-runner}
runner=/opt/actions-runner
if [[ $(id -u) != 0 ]]; then
    echo "install.sh must run as root" >&2
    exit 1
fi
if [[ ! $image_id =~ ^[A-Za-z0-9._-]+$ || ! $account =~ ^[a-z_][a-z0-9_-]*$ ]]; then
    echo "invalid image ID or account name" >&2
    exit 1
fi
id "$account" >/dev/null
if [[ ! -x $runner/bin/Runner.Listener ]]; then
    echo "the stock runner is missing in $runner" >&2
    exit 1
fi
install -d -m 0755 /usr/local/bin /usr/local/libexec /etc/ciproxy
install -m 0755 "$here/ciproxy-guest" "$here/ciproxy-cache-sidecar" /usr/local/bin/
install -m 0755 "$here/start.sh" /usr/local/libexec/ciproxy-guest-start
install -m 0755 "$here/runner-entry.sh" /usr/local/libexec/ciproxy-runner
bash "$here/install-worker.sh" "$runner" "$here/worker"
chown -R "$account" "$runner"
printf 'IMAGE_ID=%s\nVSOCK_PORT=5252\nRUNNER_ACCOUNT=%s\n' "$image_id" "$account" \
    > /etc/ciproxy/guest.env
printf '%s\n' "$image_id" > /etc/ciproxy/image-id
plutil -lint "$here/dev.ciproxy.guest.plist" >/dev/null
install -m 0644 -o root -g wheel "$here/dev.ciproxy.guest.plist" /Library/LaunchDaemons/
echo "ciproxy guest installed; it starts at the next boot"
