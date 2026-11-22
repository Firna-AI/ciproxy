#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Update the guest to the pinned macOS build. Run as root over SSH, with the
# password of USER_NAME on stdin. Apple Silicon needs a volume owner's
# password for a system update. The guest restarts when the update applies.
# Exit 0: the guest already runs the target. Exit 10: the install started in
# the background and restarts the guest. /var/log/ciproxy-update.log has its
# output.
set -euo pipefail

target=${1:?Usage: update-macos.sh VERSION-BUILD USER_NAME}
user_name=${2:?}
target_version=${target%-*}
target_build=${target##*-}
current=$(sw_vers -buildVersion)
if [[ $current == "$target_build" ]]; then
    echo "macOS already runs $target_version ($target_build)"
    exit 0
fi
softwareupdate --list --all 2>&1 | tee /tmp/ciproxy-updates.txt
# A label line looks like: "* Label: macOS Tahoe 26.7.3-25G342".
label=$(sed -n 's/^\* Label: \(.*\)$/\1/p' /tmp/ciproxy-updates.txt |
    grep -F -- "$target_version-$target_build" | head -n 1 || true)
if [[ -z $label ]]; then
    echo "Software Update offers no macOS $target_version ($target_build)." >&2
    echo "Apple may offer only a newer update. Change the pins, then build again." >&2
    exit 1
fi
echo "Downloading $label"
softwareupdate --download "$label"
# The install restarts the guest, which ends this SSH session. So it runs
# detached, and the host waits for the target build. The password file is
# open in the install process, so its name can go after the start.
password_file=$(mktemp /var/root/ciproxy-update.XXXXXX)
chmod 600 "$password_file"
cat > "$password_file"
echo "Installing $label from $current. The guest restarts."
nohup softwareupdate --install "$label" --agree-to-license --restart \
    --user "$user_name" --stdinpass < "$password_file" > /var/log/ciproxy-update.log 2>&1 &
sleep 10
rm -f "$password_file"
exit 10
