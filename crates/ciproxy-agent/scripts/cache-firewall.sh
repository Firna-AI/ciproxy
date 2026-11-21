#!/bin/sh
# Apply the cache UID policy with the current host resolvers before each start.
set -eu

policy=$(mktemp /run/ciproxy-cache-policy.XXXXXX)
trap 'rm -f -- "$policy"' EXIT
trap 'exit 1' HUP INT TERM
cat /etc/ciproxy-cache.nft > "$policy"
while read -r kind address rest || [ -n "$kind" ]
do
    [ "$kind" = nameserver ] || continue
    address=${address%%\%*}
    case "$address" in
        ''|*[!0123456789abcdefABCDEF:.]*) continue ;;
    esac
    case "$address" in
        *:*) family=dns6 ;;
        *) family=dns4 ;;
    esac
    printf 'add element inet ciproxy_cache %s { %s }\n' "$family" "$address"
done < /etc/resolv.conf >> "$policy"
/usr/sbin/nft -f "$policy"

data=/var/lib/ciproxy/area/cache/data
install -d -o ciproxy-cache -g ciproxy-cache -m 0700 "$data"
for entry in /var/lib/ciproxy/area/cache/* \
    /var/lib/ciproxy/area/cache/.[!.]* /var/lib/ciproxy/area/cache/..?*
do
    [ -e "$entry" ] || [ -L "$entry" ] || continue
    [ "$entry" != "$data" ] || continue
    mv -n -- "$entry" "$data/"
done
chown -R -h ciproxy-cache:ciproxy-cache "$data"

if [ "${1:-}" = start ]; then
    install -d -o root -g ciproxy-cache -m 0710 /run/ciproxy-cache-auth
    install -T -o root -g ciproxy-cache -m 0640 \
        /var/lib/ciproxy/cache-admin-token /run/ciproxy-cache-auth/admin-token
fi
