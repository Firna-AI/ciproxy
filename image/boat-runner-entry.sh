#!/bin/bash
# SPDX-License-Identifier: Apache-2.0

coproc CIPROXY_RUNNER_STDERR {
    exec >/dev/null 2>&1
    /usr/bin/python3 -I /usr/local/libexec/ciproxy-boat-runner-stderr.py || exec /usr/bin/cat
}
if [[ -n ${CIPROXY_RUNNER_STDERR[1]:-} ]]; then
    exec 2>&"${CIPROXY_RUNNER_STDERR[1]}" || :
fi
export BASH_ENV=/usr/local/libexec/ciproxy-boat-runner-errors.sh
exec /usr/local/libexec/ciproxy-runner-common "$@"
