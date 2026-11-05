#!/usr/bin/env bash
set -euo pipefail
if [[ ${1:-} == --help ]]; then
    exec cargo run --quiet --package ciproxy-overflow --example kms_roundtrip -- \
        --boat-key-file /nonexistent/boat-key \
        --access-token-file /nonexistent/gcp-token \
        --kms-key-resource projects/p/locations/l/keyRings/r/cryptoKeys/k --help
fi
: "${BOAT_API_KEY_FILE:?Set BOAT_API_KEY_FILE to a private credential file}"
: "${GCP_ACCESS_TOKEN_FILE:?Set GCP_ACCESS_TOKEN_FILE to a private OAuth token file}"
: "${KMS_KEY_RESOURCE:?Set KMS_KEY_RESOURCE to the Cloud KMS crypto key name}"
exec cargo run --quiet --package ciproxy-overflow --example kms_roundtrip -- \
    --boat-key-file "$BOAT_API_KEY_FILE" \
    --access-token-file "$GCP_ACCESS_TOKEN_FILE" \
    --kms-key-resource "$KMS_KEY_RESOURCE" "$@"
