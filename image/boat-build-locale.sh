#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

root=${1:-}
locale_dir="$root/usr/local/lib/ciproxy/locales/en_US.utf8"
umask 022
install -d -m 0755 "$locale_dir"
localedef --no-archive -i en_US -f UTF-8 "$locale_dir"

install -m 0644 /dev/stdin "$root/etc/profile.d/ciproxy-boat-locale.sh" <<'PROFILE'
if [ "${HOME:-}" = /home/user ]; then
    case ":${LOCPATH:-}:" in
        *:/usr/local/lib/ciproxy/locales:*) ;;
        *) LOCPATH="/usr/local/lib/ciproxy/locales${LOCPATH:+:$LOCPATH}" ;;
    esac
    case ":$LOCPATH:" in
        *:/usr/lib/locale:*) ;;
        *) LOCPATH="$LOCPATH:/usr/lib/locale" ;;
    esac
    export LOCPATH
fi
PROFILE
