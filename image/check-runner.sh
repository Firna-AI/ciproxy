#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
printf 'Pinned runner: %s\n' "$RUNNER_VERSION"
curl --fail --silent --show-error \
    https://api.github.com/repos/actions/runner/releases/latest | \
    jq '{latest_runner: .tag_name, published_at, html_url}'
printf 'Check weekly. Rebuild for each new runner. Release security fixes at once.\n'
printf 'A version check cannot prove GitHub still accepts a runner. Run the smoke job.\n'
