#!/usr/bin/env bash
# Apache-2.0. Run on the registered KVM machine from the repository root.
set -euo pipefail

skip() { printf 'SKIP: %s\n' "$1" >&2; exit 77; }
fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }

prerequisites() {
  [ "$(uname -s)" = Linux ] || skip 'an x64 Linux machine is required'
  [ "$(uname -m)" = x86_64 ] || skip 'an x64 Linux machine is required'
  [ "$(id -u)" -eq 0 ] || skip 'root is required to inspect VM ownership'
  [ -r /dev/kvm ] && [ -w /dev/kvm ] || skip 'a usable /dev/kvm is required'
  for tool in gh jq rg systemctl; do
    command -v "$tool" >/dev/null 2>&1 || skip "$tool is required"
  done
  [ -n "${CIPROXY_SMOKE_REPO:-}" ] || skip 'set CIPROXY_SMOKE_REPO to the private test repository'
  [ -n "${CIPROXY_SMOKE_REF:-}" ] || skip 'set CIPROXY_SMOKE_REF to the workflow branch'
  gh auth status >/dev/null 2>&1 || skip 'gh needs permission to dispatch the private test workflow'
  systemctl is-active --quiet ciproxy-agent.service || skip 'the registered staging agent must be active'
  smoke_state="${CIPROXY_SMOKE_STATE:-/var/lib/ciproxy/state.json}"
  [ -r "$smoke_state" ] || skip 'registered machine state is missing'
  jq -e '.identity != null and .mode == "Active"' "$smoke_state" >/dev/null || skip 'the staging machine must be registered and active'
  smoke_workflow="${CIPROXY_SMOKE_WORKFLOW:-ciproxy-agent-smoke.yml}"
  smoke_id="$(date -u +%Y%m%dT%H%M%S)-$RANDOM-$RANDOM"
  smoke_evidence=".context/ciproxy-mvp/smoke-$smoke_mode-$smoke_id"
  mkdir -p "$smoke_evidence"
}

dispatch() {
  gh workflow run "$smoke_workflow" --repo "$CIPROXY_SMOKE_REPO" --ref "$CIPROXY_SMOKE_REF" \
    -f "mode=$smoke_mode" -f "smoke_id=$smoke_id"
  local attempt
  for attempt in $(seq 1 120); do
    smoke_run="$(gh run list --repo "$CIPROXY_SMOKE_REPO" --workflow "$smoke_workflow" --limit 30 \
      --json databaseId,displayTitle | jq -r --arg name "ciproxy agent smoke $smoke_id" \
      '.[] | select(.displayTitle == $name) | .databaseId' | head -n 1)"
    if [ -n "$smoke_run" ]; then
      printf 'run=%s\nmode=%s\n' "$smoke_run" "$smoke_mode" | tee "$smoke_evidence/run.txt"
      return
    fi
    sleep 1
  done
  fail 'the dispatched GitHub run did not appear'
}

wait_for_local_job() {
  local attempt job
  for attempt in $(seq 1 180); do
    job="$(gh api "repos/$CIPROXY_SMOKE_REPO/actions/runs/$smoke_run/jobs" --jq '.jobs[0].id // empty')"
    if [ -n "$job" ]; then
      smoke_vm="$(jq -c --arg job "$job" '.runners[] | select(.report.state.state == "running" and .report.state.job_id == $job) |
        {local_id, network_index, report: {runner_id: .report.runner_id, state: .report.state, image_id: .report.image_id}}' "$smoke_state")"
      if [ -n "$smoke_vm" ]; then
        printf '%s\n' "$smoke_vm" >"$smoke_evidence/vm-before.json"
        smoke_local_id="$(printf '%s\n' "$smoke_vm" | jq -r .local_id)"
        smoke_runner_id="$(printf '%s\n' "$smoke_vm" | jq -r .report.runner_id)"
        smoke_unit="ciproxy-vm-$smoke_local_id.service"
        return
      fi
    fi
    sleep 1
  done
  fail 'the job did not appear in this machine state; check staging routing and its guest image'
}

check_result() {
  gh run watch "$smoke_run" --repo "$CIPROXY_SMOKE_REPO" --exit-status
  gh run view "$smoke_run" --repo "$CIPROXY_SMOKE_REPO" --log >"$smoke_evidence/github.log"
  rg -F "ciproxy-docker-ok=$smoke_id" "$smoke_evidence/github.log" >/dev/null || fail 'the Docker step did not produce its success marker'
  local attempt
  for attempt in $(seq 1 90); do
    if ! jq -e --arg id "$smoke_local_id" '.runners[] | select(.local_id == $id)' "$smoke_state" >/dev/null; then
      [ ! -d "/var/lib/ciproxy/jailer/firecracker/$smoke_local_id" ] || fail 'the used job disk still exists'
      if [ "$smoke_mode" = warm ]; then
        local warm_attempt replacement
        for warm_attempt in $(seq 1 90); do
          replacement="$(jq -c --arg previous "$smoke_local_id" '[.runners[] |
            select(.local_id != $previous and .report.state.state == "warm") | .local_id]' "$smoke_state")"
          if [ "$replacement" != '[]' ]; then
            printf '%s\n' "$replacement" >"$smoke_evidence/warm-after.json"
            break
          fi
          sleep 1
        done
        [ -f "$smoke_evidence/warm-after.json" ] || fail 'no fresh warm VM replaced the used VM'
      fi
      printf 'PASS: %s; GitHub run %s; evidence %s\n' "$smoke_mode" "$smoke_run" "$smoke_evidence"
      return
    fi
    sleep 1
  done
  fail 'the finished VM was not removed'
}

run_smoke() {
  smoke_mode="$1"
  prerequisites
  if [ "$smoke_mode" = warm ]; then
    smoke_warm="$(jq -c '[.runners[] | select(.report.state.state == "warm") | .report.runner_id]' "$smoke_state")"
    [ "$smoke_warm" != '[]' ] || skip 'the staging gateway must keep at least one warm runner on this machine'
    printf '%s\n' "$smoke_warm" >"$smoke_evidence/warm-before.json"
  fi
  dispatch
  wait_for_local_job
  if [ "$smoke_mode" = warm ]; then
    printf '%s\n' "$smoke_warm" | jq -e --arg runner "$smoke_runner_id" 'index($runner) != null' >/dev/null || fail 'the job did not use a runner that was already warm'
  fi
  if [ "$smoke_mode" = restart ]; then
    local before after attempt
    before="$(systemctl show --property=MainPID --value "$smoke_unit")"
    [ -n "$before" ] && [ "$before" != 0 ] || fail 'the job has no independent VM process'
    systemctl restart ciproxy-agent.service
    systemctl is-active --quiet ciproxy-agent.service || fail 'the agent did not restart'
    after="$(systemctl show --property=MainPID --value "$smoke_unit")"
    [ "$before" = "$after" ] || fail 'the VM process changed during agent restart'
    for attempt in $(seq 1 60); do
      if jq -e --arg id "$smoke_local_id" '.runners[] | select(.local_id == $id and .report.state.state == "running")' "$smoke_state" >/dev/null; then
        printf 'before=%s\nafter=%s\n' "$before" "$after" >"$smoke_evidence/vm-pid.txt"
        break
      fi
      sleep 1
    done
    [ -f "$smoke_evidence/vm-pid.txt" ] || fail 'the agent did not recover the running VM'
  fi
  check_result
}
