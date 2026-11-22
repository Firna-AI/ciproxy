#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# A fake runner entry for the macOS VM test. It exits with 3 only when each
# check passes, so the exit code that reaches the host proves the checks.
# The test replaces EXPECTED_USER before it copies the file.
expected_user=EXPECTED_USER
socket=/var/run/ciproxy/assignment.sock
[[ $(id -un) == "$expected_user" ]] || exit 11
[[ $ACTIONS_RUNNER_INPUT_JITCONFIG == fake-jit-for-the-vm-test ]] || exit 12
[[ -z ${LEAK:-} ]] || exit 13
[[ $(launchctl manageruid) == $(id -u) ]] || exit 14
[[ $CIPROXY_ASSIGNMENT_SOCKET == "$socket" && -S $socket ]] || exit 15
[[ $HOME == "/Users/$expected_user" ]] || exit 16
echo "2026-11-22 00:00:00Z: Listening for Jobs"
handoff='{"handoff_protocol_version":2,"job_id":"00000000-0000-0000-0000-000000000123",'
handoff+='"runtime":{"docker_data_bytes":0,"proxy_url":"","github_token":"original-token",'
handoff+='"cache_url":"https://cache.example/","results_url":"https://results.example/",'
handoff+='"permissions":{"read":true,"write":true}}}'
reply=$(printf '%s\n' "$handoff" | nc -U -w 10 "$socket" | head -n 1)
[[ $reply == *'"decision":"github_only"'* ]] || exit 17
exit 3
