#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Remove the build-only state of ciproxy from the guest. Run as root last.
# The upstream software reports, helpers, and tests stay, as on Linux.
set -euo pipefail

home=/Users/runner
# The auto-login file holds the build password in a reversible form, so the
# shipped image accepts no SSH password. No SSH key is installed either.
mkdir -p /etc/ssh/sshd_config.d
cat > /etc/ssh/sshd_config.d/100-ciproxy.conf <<'EOF'
PasswordAuthentication no
KbdInteractiveAuthentication no
EOF
rm -f "$home/.ssh/authorized_keys" /var/root/.ssh/authorized_keys
rm -rf "$home/Library/Caches/XcodeInstall" "$home/bootstrap" /tmp/ciproxy
rm -f "$home/.bash_history" "$home/.zsh_history" /var/root/.bash_history
rm -rf "$home/.bash_sessions" "$home/.zsh_sessions"
softwareupdate --schedule off
defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticCheckEnabled -bool false
defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticDownload -bool false
rm -rf /Library/Logs/DiagnosticReports/* /private/var/log/*.gz
sync
