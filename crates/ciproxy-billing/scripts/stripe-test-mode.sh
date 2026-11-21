#!/usr/bin/env bash
set -eu
cd "$(dirname "$0")/../../.."
exec cargo run --quiet --package ciproxy-billing --example stripe_test_mode -- "$@"
