#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 PROXY_URL COMMAND [ARG ...]" >&2
  exit 2
fi

cache_proxy_url="$1"
shift
cache_patch_directory="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cache_runner_command="$1"
if [[ "$cache_runner_command" != */* ]]; then
  cache_runner_command="$(command -v -- "$cache_runner_command")"
fi
export CIPROXY_RUNNER_ROOT="${CIPROXY_RUNNER_ROOT:-$(CDPATH= cd -- "$(dirname -- "$cache_runner_command")" && pwd)}"
if [[ "${ACTIONS_RUNNER_HOOK_JOB_STARTED:-}" != "$cache_patch_directory/job-started.sh" ]]; then
  export CIPROXY_PREVIOUS_JOB_STARTED_HOOK="${ACTIONS_RUNNER_HOOK_JOB_STARTED:-}"
fi
export CIPROXY_CACHE_PROXY_URL="${cache_proxy_url%/}/"
export ACTIONS_RUNNER_HOOK_JOB_STARTED="$cache_patch_directory/job-started.sh"
export ACTIONS_CACHE_URL="${cache_proxy_url%/}/"
export ACTIONS_RESULTS_URL="${cache_proxy_url%/}/"
export ACTIONS_CACHE_SERVICE_V2=true
exec "$@"
