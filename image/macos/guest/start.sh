#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# Start the guest once at boot. Then end the VM, also after a failed start,
# as ExecStopPost does on Linux. A reboot would start the VM again.
set -u
. /etc/ciproxy/guest.env
/usr/local/bin/ciproxy-guest --image-id "$IMAGE_ID" \
    --runner-account "$RUNNER_ACCOUNT" vsock --port "$VSOCK_PORT"
exec /sbin/shutdown -h now
