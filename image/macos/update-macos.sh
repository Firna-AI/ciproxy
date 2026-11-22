#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Update the guest to the pinned macOS build. Run as root over SSH, with the
# password of USER_NAME on stdin. Apple Silicon needs a volume owner's
# password for a system update. The guest restarts when the update applies.
# Exit 0: the guest already runs the target. Exit 10: the install ended
# without a restart yet. A restart ends the SSH session first (exit 255).
set -euo pipefail

target=${1:?Usage: update-macos.sh VERSION-BUILD USER_NAME}
user_name=${2:?}
target_version=${target%-*}
target_build=${target##*-}
# The download asks for the password too, so read it once and give it to
# each command. The password file has no final newline.
IFS= read -r password || true
[[ -n $password ]] || { echo "no password on stdin" >&2; exit 1; }
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
printf '%s\n' "$password" | softwareupdate --download "$label"
# The restart closes this SSH session, so the host sees SSH exit 255 and
# waits for the target build.
echo "Installing $label from $current. The guest restarts."
printf '%s\n' "$password" | softwareupdate --install "$label" --agree-to-license --restart \
    --user "$user_name" --stdinpass
exit 10
