#!/usr/bin/env bash
# Run one macOS VM helper under launchd as root, with no control plane.
#
# Usage: sudo macos-helper-smoke.sh SIGNED_AGENT BUNDLE [GUEST_BINARY]
#
# SIGNED_AGENT is an agent binary signed with the virtualization
# entitlement. BUNDLE is a VM bundle in the ciproxy-vz layout, with its own
# machine-identifier.bin and mac-address.txt, on the data volume. The
# script clones it, starts the helper, checks the control socket, stops the
# VM, and removes every file that it made.
#
# With GUEST_BINARY, a macOS build of ciproxy-guest, the script copies it
# into the VM over SSH as the user that ran sudo, starts it on vsock port
# 5252, and proves that CONNECT passes a working stream fd. The guest
# account is CIPROXY_VZ_GUEST_USER, by default admin, with SSH by key and
# `sudo -n`. The script refuses a Mac with an installed agent. Take the
# test Mac's VM lock first.
set -euo pipefail

if [[ "$(uname -s)" != Darwin || "${EUID}" -ne 0 || $# -lt 2 || $# -gt 3 ]]; then
  echo "SKIP: run as root on macOS with an agent binary and a bundle" >&2
  exit 77
fi
agent="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
source_bundle=$2
guest_binary=${3:-}
guest_user=${CIPROXY_VZ_GUEST_USER:-admin}
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

# bootout can return once it has sent SIGTERM. Wait until launchd has
# unloaded the job: the helper's exit timeout of 30 s and a margin.
stop_helper() {
  launchctl bootout "system/${label}" 2>/dev/null || true
  for _ in $(seq 1 160); do
    launchctl print "system/${label}" >/dev/null 2>&1 || return 0
    sleep 0.25
  done
  echo "FAIL: the helper job did not unload" >&2
  return 1
}

cleanup() {
  stop_helper || true
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

if [[ -n "${guest_binary}" ]]; then
  ssh_options=(-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null
    -o BatchMode=yes -o ConnectTimeout=5)
  as_user() { sudo -u "${SUDO_USER:?run the script with sudo}" "$@"; }
  mac=$(cat "${folder}/bundle/mac-address.txt")
  address=""
  for _ in $(seq 1 90); do
    address=$(python3 "${client}" lease "${mac}")
    if [[ -n "${address}" ]] &&
      as_user ssh "${ssh_options[@]}" "${guest_user}@${address}" true 2>/dev/null; then
      break
    fi
    address=""
    sleep 1
  done
  if [[ -z "${address}" ]]; then
    echo "FAIL: no SSH to the guest" >&2
    exit 1
  fi
  echo "SSH after $(($(date +%s) - started)) s at ${address}"
  as_user scp "${ssh_options[@]}" "${guest_binary}" \
    "${guest_user}@${address}:ciproxy-guest"
  as_user ssh "${ssh_options[@]}" "${guest_user}@${address}" \
    "chmod 0755 ciproxy-guest && sudo -n nohup \"\$HOME/ciproxy-guest\" \
     --image-id smoke --runner-account ${guest_user} --runner-entry /usr/bin/true \
     vsock --port 5252 </dev/null >guest.log 2>&1 &"
fi

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
if [[ -n "${guest_binary}" && "${connected}" != yes ]]; then
  echo "FAIL: the guest listened, but CONNECT gave no stream" >&2
  exit 1
fi

# STOP only asks the guest. A guest with a desktop session can ignore it.
echo "stop: $(python3 "${client}" "${socket}" stop)"
if wait_answer=$(python3 "${client}" "${socket}" wait 60 2>/dev/null); then
  echo "wait after the guest stop: ${wait_answer}"
else
  echo "the guest did not stop in 60 s; stop the VM from the host"
  waiter_output=$(mktemp)
  python3 "${client}" "${socket}" wait 60 >"${waiter_output}" 2>&1 &
  waiter=$!
  sleep 1
  stop_helper
  wait "${waiter}" || true
  echo "wait during the host stop: $(cat "${waiter_output}")"
  rm -f "${waiter_output}"
fi
for _ in $(seq 1 30); do
  [[ -f "${folder}/exit.json" ]] && break
  sleep 1
done
echo "exit.json: $(cat "${folder}/exit.json" 2>/dev/null || echo missing)"
echo "helper log (last lines):"
tail -n 20 "${log}" || true
rm -f "${log}"
stop_helper
if launchctl list | grep -q "${label}"; then
  echo "FAIL: the helper job is still loaded" >&2
  exit 1
fi
echo "helper job unloaded"
