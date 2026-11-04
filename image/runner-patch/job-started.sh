#!/usr/bin/env bash
set -uo pipefail

run_previous_hook() {
  local cache_previous_hook="${CIPROXY_PREVIOUS_JOB_STARTED_HOOK:-}"
  if [[ -z "$cache_previous_hook" ]]; then
    return 0
  fi
  case "$cache_previous_hook" in
    *.sh)
      if command -v bash >/dev/null 2>&1; then
        bash -e "$cache_previous_hook"
      else
        sh -e "$cache_previous_hook"
      fi
      ;;
    *.ps1)
      local cache_powershell=powershell
      if command -v pwsh >/dev/null 2>&1; then
        cache_powershell=pwsh
      fi
      "$cache_powershell" -command '. $env:CIPROXY_PREVIOUS_JOB_STARTED_HOOK'
      ;;
    *.js)
      local cache_node_version="${ACTIONS_RUNNER_FORCED_INTERNAL_NODE_VERSION:-node24}"
      case "$cache_node_version" in
        node20|node24) ;;
        *) cache_node_version=node24 ;;
      esac
      "${CIPROXY_RUNNER_ROOT}/externals/${cache_node_version}/bin/node" "$cache_previous_hook"
      ;;
    *)
      printf 'Unsupported existing job hook: %s\n' "$cache_previous_hook" >&2
      return 1
      ;;
  esac
}

cache_hook_status=0
run_previous_hook || cache_hook_status=$?
if [[ -n "${GITHUB_ENV:-}" && -n "${CIPROXY_CACHE_PROXY_URL:-}" ]]; then
  {
    printf 'ACTIONS_CACHE_URL=%s\n' "$CIPROXY_CACHE_PROXY_URL"
    printf 'ACTIONS_RESULTS_URL=%s\n' "$CIPROXY_CACHE_PROXY_URL"
    printf 'ACTIONS_CACHE_SERVICE_V2=true\n'
  } >> "$GITHUB_ENV" || printf 'Cache endpoint environment file unavailable\n' >&2
else
  printf 'Cache endpoint environment file unavailable\n' >&2
fi
exit "$cache_hook_status"
