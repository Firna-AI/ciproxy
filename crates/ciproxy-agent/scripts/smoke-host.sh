#!/usr/bin/env bash
# Apache-2.0. Run only on a disposable ext4 KVM host with staging integration.
set -euo pipefail
source crates/ciproxy-agent/scripts/smoke-common.sh

host_preflight() {
  [ "$(uname -s)" = Linux ] && [ "$(uname -m)" = x86_64 ] || skip 'an x64 Linux host is required'
  [ "$(id -u)" -eq 0 ] || skip 'root is required'
  [ -r /dev/kvm ] && [ -w /dev/kvm ] || skip 'a usable /dev/kvm is required'
  for tool in gh jq rg python3; do
    command -v "$tool" >/dev/null 2>&1 || skip "$tool is required"
  done
  local host_fs
  host_fs="$(python3 - <<'PY'
from pathlib import Path
target = str(Path('/var/lib').resolve())
mounts = []
for line in Path('/proc/self/mountinfo').read_text().splitlines():
    left, right = line.split(' - ', 1)
    where = left.split()[4]
    if target == where or target.startswith(where.rstrip('/') + '/'):
        mounts.append((len(where), right.split()[0]))
print(max(mounts)[1])
PY
)"
  [ "$host_fs" = ext4 ] || skip 'use a disposable host with /var/lib on ext4'
  [ ! -e /var/lib/ciproxy/state.json ] && [ ! -e /var/lib/ciproxy/area.xfs ] && [ ! -e /var/lib/ciproxy/area.json ] || skip 'use a fresh disposable host without an existing ciproxy installation'
  [ "${CIPROXY_SMOKE_INTEGRATION_READY:-}" = 1 ] || skip 'confirm aligned staging gateway, image, guest, and GitHub App with CIPROXY_SMOKE_INTEGRATION_READY=1'
  [ -n "${CIPROXY_SMOKE_INSTALL_TOKEN:-}" ] || skip 'set a staging CIPROXY_SMOKE_INSTALL_TOKEN'
  [ -n "${CIPROXY_SMOKE_REPO:-}" ] && [ -n "${CIPROXY_SMOKE_REF:-}" ] || skip 'set CIPROXY_SMOKE_REPO and CIPROXY_SMOKE_REF'
  case "${CIPROXY_AGENT_URL:-}" in wss://*) ;; *) skip 'set a staging CIPROXY_AGENT_URL with valid TLS' ;; esac
  case "$CIPROXY_AGENT_URL" in *$'\n'*|*$'\r'*|*' '*|*'"'*) fail 'the staging URL contains invalid environment-file characters' ;; esac
  gh auth status >/dev/null 2>&1 || skip 'gh needs private workflow dispatch access'
  host_binary="${CIPROXY_SMOKE_BINARY:-target/debug/ciproxy-agent}"
  [ -x "$host_binary" ] || skip 'build the agent or set CIPROXY_SMOKE_BINARY'
  host_evidence=".context/ciproxy-mvp/host-smoke-$(date -u +%Y%m%dT%H%M%S)-$RANDOM"
  mkdir -p "$host_evidence"
}

host_runtime_tools() {
  for tool in cp mkfs.ext4 mkfs.xfs xfs_info blkid mount losetup ip nft sysctl modprobe findmnt stat sha256sum systemctl; do
    command -v "$tool" >/dev/null 2>&1 || fail "install did not provide $tool"
  done
}

host_service_environment() {
  [ ! -e /etc/ciproxy-agent/environment ] || skip 'use a fresh host without an existing agent environment file'
  python3 - <<'PY'
import os
from pathlib import Path
directory = Path('/etc/ciproxy-agent')
directory.mkdir(mode=0o700, exist_ok=True)
if directory.is_symlink() or directory.stat().st_uid != 0:
    raise SystemExit('FAIL: the agent configuration directory must belong to root')
descriptor = os.open(directory / 'environment', os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
with os.fdopen(descriptor, 'w') as stream:
    stream.write('CIPROXY_AGENT_URL=' + os.environ['CIPROXY_AGENT_URL'] + '\n')
PY
}

host_install() {
  "$host_binary" install --token "$CIPROXY_SMOKE_INSTALL_TOKEN"
  host_binary=/usr/local/bin/ciproxy-agent
}

host_identity() {
  jq -Sc '{identity,mode}' /var/lib/ciproxy/state.json | sha256sum | cut -d ' ' -f 1
}

host_backing() { stat --format='%d:%i:%s' /var/lib/ciproxy/area.xfs; }

host_assert_preserved() {
  [ "$(host_backing)" = "$host_backing_before" ] || fail 'repeat install replaced or resized the backing file'
  [ "$(jq -r .uuid /var/lib/ciproxy/area.json)" = "$host_uuid" ] || fail 'repeat install changed the filesystem UUID'
  [ "$(host_identity)" = "$host_identity_before" ] || fail 'repeat install changed identity or drain state'
  [ "$(sha256sum "$host_sentinel" | cut -d ' ' -f 1)" = "$host_sentinel_sha" ] || fail 'repeat install lost data'
  [ -x /var/lib/ciproxy/tools/v1.17.0/jailer ] || fail 'repeat install did not repair the jailer'
  systemctl is-enabled --quiet var-lib-ciproxy-area.mount || fail 'the repaired mount is not persistent'
  [ "$(findmnt --noheadings --output FSTYPE --mountpoint /var/lib/ciproxy/area)" = xfs ] || fail 'the workload area is not mounted as XFS'
}

host_repair_probe() {
  host_backing_before="$(host_backing)"
  host_uuid="$(jq -r .uuid /var/lib/ciproxy/area.json)"
  host_identity_before="$(host_identity)"
  host_sentinel=/var/lib/ciproxy/area/host-smoke-sentinel
  printf 'ciproxy host preservation %s\n' "$host_uuid" >"$host_sentinel"
  host_sentinel_sha="$(sha256sum "$host_sentinel" | cut -d ' ' -f 1)"
  cp --reflink=always "$host_sentinel" "$host_sentinel.copy"
  python3 crates/ciproxy-agent/scripts/reflink-evidence.py "$host_sentinel" "$host_sentinel.copy" >"$host_evidence/sentinel-extents.json"
  rm /var/lib/ciproxy/tools/v1.17.0/jailer
  rm /etc/systemd/system/var-lib-ciproxy-area.mount
  host_install >"$host_evidence/repaired-install.log" 2>&1
  host_assert_preserved
}

host_job_extents() {
  local image_key root source
  image_key="$(printf '%s\n' "$smoke_vm" | jq -r .report.image_id | tr -d '\n' | sha256sum | cut -d ' ' -f 1)"
  root="/var/lib/ciproxy/area/jailer/firecracker/$smoke_local_id/root/rootfs.ext4"
  source="/var/lib/ciproxy/area/images/$image_key/rootfs.ext4"
  python3 crates/ciproxy-agent/scripts/reflink-evidence.py "$source" "$root" >"$smoke_evidence/root-extents.json"
  printf '%s\n' "$smoke_local_id" >>"$host_evidence/job-vms.txt"
}

host_two_jobs() {
  local number before after
  for number in 1 2; do
    smoke_mode=host
    CIPROXY_SMOKE_EVIDENCE="$host_evidence/job-$number" prerequisites
    dispatch
    wait_for_local_job
    host_job_extents
    if [ "$number" -eq 1 ]; then
      before="$(systemctl show --property=MainPID --value "$smoke_unit")"
      [ -n "$before" ] && [ "$before" != 0 ] || fail 'the first job has no independent VM process'
      host_install >"$smoke_evidence/reinstall-during-job.log" 2>&1
      after="$(systemctl show --property=MainPID --value "$smoke_unit")"
      [ "$before" = "$after" ] || fail 'repeat install changed the running VM process'
      host_assert_preserved
    fi
    check_result
  done
  [ "$(sort -u "$host_evidence/job-vms.txt" | wc -l)" -eq 2 ] || fail 'the two jobs did not use distinct fresh VMs'
}

host_cleanup() {
  "$host_binary" uninstall >"$host_evidence/uninstall.log" 2>&1
  [ ! -e /var/lib/ciproxy/area.xfs ] || fail 'uninstall retained the backing file'
  [ ! -e /var/lib/ciproxy/area.json ] || fail 'uninstall retained the area ownership record'
  [ ! -e /etc/systemd/system/var-lib-ciproxy-area.mount ] || fail 'uninstall retained the mount unit'
  if findmnt --mountpoint /var/lib/ciproxy/area >/dev/null; then fail 'uninstall retained the mount'; fi
}

host_main() {
  host_preflight
  host_service_environment
  host_install >"$host_evidence/first-install.log" 2>&1
  host_runtime_tools
  host_repair_probe
  host_two_jobs
  host_cleanup
  printf 'PASS: ext4 install, repair, two real reflink jobs, uninstall; evidence %s\n' "$host_evidence"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then host_main; fi
