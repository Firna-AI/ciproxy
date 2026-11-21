#!/usr/bin/env bash
set -euo pipefail
if [[ ${1:-} == --help ]]; then
    exec cargo run --quiet --package ciproxy-overflow --example boat_live -- \
        --boat-key-file "${BOAT_API_KEY_FILE:-/nonexistent/boat-key}" --help
fi
: "${BOAT_API_KEY_FILE:?Set BOAT_API_KEY_FILE to a private credential file}"
exec cargo run --quiet --package ciproxy-overflow --example boat_live -- \
    --boat-key-file "$BOAT_API_KEY_FILE" "$@"
