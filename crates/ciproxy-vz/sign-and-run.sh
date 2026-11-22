#!/bin/sh
# Sign a binary ad hoc with the virtualization entitlement, then run it.
# Use it as the cargo runner for the macOS tests of ciproxy-vz.
set -eu
binary=$1
shift
codesign --force --sign - --entitlements "$(dirname "$0")/vz.entitlements" "$binary"
exec "$binary" "$@"
