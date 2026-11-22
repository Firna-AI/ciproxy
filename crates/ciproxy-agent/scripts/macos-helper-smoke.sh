#!/usr/bin/env bash
# Run macOS VM helpers under launchd as root, with no control plane.
#
# Usage: sudo macos-helper-smoke.sh SIGNED_AGENT BUNDLE [GUEST_BINARY]
#
# SIGNED_AGENT is an agent binary signed with the virtualization
# entitlement. BUNDLE is a VM bundle in the ciproxy-vz layout, with its own
# machine-identifier.bin and mac-address.txt, on the data volume. The
# script clones it, starts a helper, checks the control socket, stops the
# VM, and removes every file that it made.
#
# With GUEST_BINARY, a macOS build of ciproxy-guest, the script copies it
# into the VM over SSH as the user that ran sudo, starts it on vsock port
# 5252, and proves that CONNECT passes a working stream fd. Then it saves
# the VM, restores the state in a second helper from an APFS clone at
# another path, as a seat's saved state does, and proves CONNECT again.
# The guest account is CIPROXY_VZ_GUEST_USER, by default admin, with SSH
# by key and `sudo -n`. The script refuses a Mac with an installed agent.
# Take the test Mac's VM lock first.
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
log="/tmp/ciproxy-helper-$$.log"
ids=()

new_id() { uuidgen | tr -d - | tr '[:upper:]' '[:lower:]'; }
folder_of() { echo "${control}/area/vms/$1"; }
socket_of() { echo "$(folder_of "$1")/control.sock"; }
control_call() { python3 "${client}" "$(socket_of "$1")" "${@:2}"; }

# bootout can return once it has sent SIGTERM. Wait until launchd has
# unloaded the job: the helper's exit timeout of 30 s and a margin.
stop_helper() {
  launchctl bootout "system/dev.ciproxy.vm.$1" 2>/dev/null || true
  for _ in $(seq 1 160); do
    launchctl print "system/dev.ciproxy.vm.$1" >/dev/null 2>&1 || return 0
    sleep 0.25
  done
  echo "FAIL: the helper job $1 did not unload" >&2
  return 1
}

cleanup() {
  for each in "${ids[@]}"; do
    stop_helper "${each}" || true
    rm -rf "$(folder_of "${each}")" "${control}/vms/${each}.plist"
  done
  rmdir "${control}/area/vms" "${control}/area" "${control}/vms" "${control}" \
    2>/dev/null || true
  rm -f "${log}"
}
trap cleanup EXIT

# launch ID START [STATE]: write vm.json and the launchd file, then load it.
launch() {
  local id=$1 start=$2 state=${3:-} folder plist saved=null
  folder=$(folder_of "${id}")
  plist="${control}/vms/${id}.plist"
  if [[ -n "${state}" ]]; then saved="\"${state}\""; fi
  cat >"${folder}/vm.json" <<JSON
{"format":1,"seat":0,"cpus":4,"memory_bytes":8589934592,"bundle":"${folder}/bundle","start":"${start}","saved_state":${saved}}
JSON
  cat >"${plist}" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>dev.ciproxy.vm.${id}</string>
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
  launchctl bootstrap system "${plist}"
}

# running ID: wait up to 60 s until the helper answers `OK running`.
running() {
  for _ in $(seq 1 600); do
    if [[ -S "$(socket_of "$1")" ]] &&
      [[ "$(control_call "$1" state 2>/dev/null)" == "OK running" ]]; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

# connect ID: try CONNECT 5252 for up to 90 s and print the result.
connect() {
  local answer=""
  for _ in $(seq 1 90); do
    answer=$(control_call "$1" connect 5252 || true)
    if [[ "${answer}" == OK* ]]; then
      echo "${answer}"
      return 0
    fi
    sleep 1
  done
  echo "connect: ${answer} (no guest listener)"
  return 1
}

mkdir -p "${control}/area/vms" "${control}/vms"
chmod 700 "${control}" "${control}/area" "${control}/area/vms" "${control}/vms"
id=$(new_id)
ids+=("${id}")
mkdir -m 700 "$(folder_of "${id}")"
cp -c -R "${source_bundle}" "$(folder_of "${id}")/bundle"

started=$(date +%s)
launch "${id}" cold
running "${id}" || true
echo "state after $(($(date +%s) - started)) s: $(control_call "${id}" state)"

if [[ -n "${guest_binary}" ]]; then
  ssh_options=(-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null
    -o BatchMode=yes -o ConnectTimeout=5)
  as_user() { sudo -u "${SUDO_USER:?run the script with sudo}" "$@"; }
  mac=$(cat "$(folder_of "${id}")/bundle/mac-address.txt")
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
  if ! connect "${id}"; then
    echo "FAIL: the guest listened, but CONNECT gave no stream" >&2
    exit 1
  fi
  echo "connect after $(($(date +%s) - started)) s"

  # Save, then restore from a clone at another path, as a seat does. The
  # client has closed its stream. The guest needs a moment to return to
  # accept: a pause with a connection still open stopped the VM.
  sleep 1
  saved_at=$(date +%s)
  echo "save: $(control_call "${id}" save)"
  echo "save took $(($(date +%s) - saved_at)) s"
  stop_helper "${id}"
  first=$(folder_of "${id}")
  mv "${first}/state.vzvmsave" "${first}/bundle/state.vzvmsave"
  id=$(new_id)
  ids+=("${id}")
  mkdir -m 700 "$(folder_of "${id}")"
  cp -c -R "${first}/bundle" "$(folder_of "${id}")/bundle"
  restored_at=$(date +%s)
  launch "${id}" restore "$(folder_of "${id}")/bundle/state.vzvmsave"
  running "${id}" || true
  echo "restored state after $(($(date +%s) - restored_at)) s: $(control_call "${id}" state)"
  if [[ -f "$(folder_of "${id}")/restore-failed" ]]; then
    echo "FAIL: the restore failed and the helper booted cold" >&2
    exit 1
  fi
  if ! connect "${id}"; then
    echo "FAIL: no guest stream after the restore" >&2
    exit 1
  fi
  echo "connect after the restore: $(($(date +%s) - restored_at)) s"
else
  connect "${id}" || true
fi

# STOP only asks the guest. A guest with a desktop session can ignore it.
echo "stop: $(control_call "${id}" stop)"
if wait_answer=$(control_call "${id}" wait 60 2>/dev/null); then
  echo "wait after the guest stop: ${wait_answer}"
else
  echo "the guest did not stop in 60 s; stop the VM from the host"
  waiter_output=$(mktemp)
  control_call "${id}" wait 60 >"${waiter_output}" 2>&1 &
  waiter=$!
  sleep 1
  stop_helper "${id}"
  wait "${waiter}" || true
  echo "wait during the host stop: $(cat "${waiter_output}")"
  rm -f "${waiter_output}"
fi
for _ in $(seq 1 30); do
  [[ -f "$(folder_of "${id}")/exit.json" ]] && break
  sleep 1
done
echo "exit.json: $(cat "$(folder_of "${id}")/exit.json" 2>/dev/null || echo missing)"
echo "helper log (last lines):"
tail -n 20 "${log}" || true
for each in "${ids[@]}"; do
  stop_helper "${each}"
  if launchctl list | grep -q "dev.ciproxy.vm.${each}"; then
    echo "FAIL: the helper job ${each} is still loaded" >&2
    exit 1
  fi
done
echo "helper jobs unloaded"
