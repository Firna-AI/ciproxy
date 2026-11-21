#!/usr/bin/env bash
# Apache-2.0.
set -euo pipefail
source crates/ciproxy-agent/scripts/smoke-common.sh
run_smoke restart
