#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
unset BASH_ENV
set -E

ciproxy_boat_runner_error() {
    local status=$1 source_name=${2##*/} line=$3
    if [[ ! $source_name =~ ^[A-Za-z0-9_.-]{1,31}$ ]]; then
        source_name=unknown
    fi
    printf '[ciproxy] runner entry error: %s:%d status=%d\n' "$source_name" "$line" "$status" >&2 || :
}

trap 'ciproxy_boat_runner_error "$?" "${BASH_SOURCE[0]:-unknown}" "$LINENO"' ERR
