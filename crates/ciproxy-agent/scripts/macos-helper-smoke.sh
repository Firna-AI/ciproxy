#!/usr/bin/env bash
# Run one macOS VM helper under launchd as root, with no control plane.
#
# Usage: sudo macos-helper-smoke.sh SIGNED_AGENT BUNDLE
#
# SIGNED_AGENT is an agent binary signed with the virtualization
# entitlement. BUNDLE is a VM bundle in the ciproxy-vz layout, with its own
# machine-identifier.bin and mac-address.txt, on the data volume. The
# script clones it, starts the helper, checks the control socket and the
# vsock fd passing, stops the VM, and removes every file that it made.
# It refuses a Mac with an installed agent. Take the test Mac's VM lock
# first.
set -euo pipefail

if [[ "$(uname -s)" != Darwin || "${EUID}" -ne 0 || $# -ne 2 ]]; then
  echo "SKIP: run as root on macOS with an agent binary and a bundle" >&2
  exit 77
fi
agent="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
source_bundle=$2
client="$(cd "$(dirname "$0")" && pwd)/macos_vm_control.py"
control=/var/db/ciproxy
if [[ -e "${control}/state.json" || -e "${control}/area.json" ]]; then
  echo "FAIL: an agent is installed on this Mac; uninstall it first" >&2
  exit 1
fi

id=$(uuidgen | tr -d - | tr '[:upper:]' '[:lower:]')
label="dev.ciproxy.vm.${id}"
folder="${control}/area/vms/${id}"
plist="${control}/vms/${id}.plist"
log="/tmp/ciproxy-helper-${id}.log"

cleanup() {
  launchctl bootout "system/${label}" 2>/dev/null || true
  rm -rf "${folder}" "${plist}"
  rmdir "${control}/area/vms" "${control}/area" "${control}/vms" "${control}" \
    2>/dev/null || true
}
trap cleanup EXIT

mkdir -p "${folder}" "${control}/vms"
chmod 700 "${control}" "${control}/area" "${control}/area/vms" "${folder}" \
  "${control}/vms"
cp -c -R "${source_bundle}" "${folder}/bundle"
cat >"${folder}/vm.json" <<JSON
{"format":1,"seat":0,"cpus":4,"memory_bytes":8589934592,"bundle":"${folder}/bundle","start":"cold","saved_state":null}
JSON
cat >"${plist}" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>${label}</string>
  <key>ProgramArguments</key>
  <array><string>${agent}</string><string>vm-run</string><string>${id}</string></array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><false/>
  <key>ExitTimeOut</key><integer>30</integer>
  <key>StandardErrorPath</key><string>${log}</string>
</dict>
</plist>
PLIST
chmod 600 "${plist}"

started=$(date +%s)
launchctl bootstrap system "${plist}"
socket="${folder}/control.sock"
for _ in $(seq 1 60); do
  if [[ -S "${socket}" ]] &&
    [[ "$(python3 "${client}" "${socket}" state)" == "OK running" ]]; then
    break
  fi
  sleep 1
done
echo "state after $(($(date +%s) - started)) s: $(python3 "${client}" "${socket}" state)"

connected=no
for _ in $(seq 1 90); do
  answer=$(python3 "${client}" "${socket}" connect 5252 || true)
  if [[ "${answer}" == OK* ]]; then
    echo "connect after $(($(date +%s) - started)) s:"
    echo "${answer}"
    connected=yes
    break
  fi
  sleep 1
done
[[ "${connected}" == yes ]] || echo "connect: ${answer} (no guest listener)"

echo "stop: $(python3 "${client}" "${socket}" stop)"
if ! wait_answer=$(python3 "${client}" "${socket}" wait 120 2>/dev/null); then
  echo "wait: the guest did not stop in 120 s; stop the helper"
  launchctl bootout "system/${label}" || true
  sleep 2
else
  echo "wait: ${wait_answer}"
fi
for _ in $(seq 1 30); do
  [[ -f "${folder}/exit.json" ]] && break
  sleep 1
done
echo "exit.json: $(cat "${folder}/exit.json" 2>/dev/null || echo missing)"
echo "helper log (last lines):"
tail -n 20 "${log}" || true
rm -f "${log}"
