#!/usr/bin/env bash
# Run only as a local operator. Each first apply asks for confirmation.
set -euo pipefail
exec python3 deploy/scripts/bootstrap.py "$@"
