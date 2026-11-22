#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Build the macOS job image on an Apple Silicon Mac. Run from the repository
# root: bash image/macos/build.sh WORK_DIR [PHASE...]. With no phase, every
# phase runs in order. The README of this folder explains each phase.
set -Eeuo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repository=$(cd "$definition/../.." && pwd)
# DISK_BYTES and IMAGE_RELEASE_ID in the environment override the pins.
disk_override=${DISK_BYTES:-}
source "$definition/pins.env"
DISK_BYTES=${disk_override:-$DISK_BYTES}
work_dir=$(mkdir -p "${1:?Usage: image/macos/build.sh WORK_DIR [PHASE...]}" && cd "$1" && pwd)
shift
vm_tool="$work_dir/tools/ciproxy-image-vm"
source "$definition/build-lib.sh"
phases=("$@")
[[ ${#phases[@]} -gt 0 ]] || phases=(fetch tools install setup recovery configure provision
    facts package)
trap 'vm_stop' EXIT
phase=start
trap 'echo "build.sh: phase $phase failed at line $LINENO (exit $?)" >&2' ERR

phase_fetch() {
    bash "$definition/fetch.sh" "$work_dir"
}

phase_tools() {
    mkdir -p "$work_dir/tools"
    cargo build --release --manifest-path "$repository/Cargo.toml" --package ciproxy-image-vm
    cp "$repository/target/release/ciproxy-image-vm" "$vm_tool"
    codesign --force --sign - \
        --entitlements "$repository/crates/ciproxy-vz/vz.entitlements" "$vm_tool"
    swiftc -O -o "$work_dir/tools/ocr" "$definition/ocr.swift"
}

phase_install() {
    rm -rf "$bundle"
    "$vm_tool" install --restore-image "$work_dir/inputs/restore.ipsw" --bundle "$bundle" \
        --disk-bytes "$DISK_BYTES"
}

phase_setup() {
    make_secrets
    vm_start
    python3 "$definition/setup-assistant.py" "$work_dir"
    wait_ssh 300
    guest_ssh 'sudo shutdown -h now' || true
    vm_wait_stopped 300
}

phase_recovery() {
    vm_start recovery
    python3 "$definition/recovery-sip.py" "$work_dir"
    vm_wait_stopped 300
}

phase_configure() {
    vm_start
    wait_ssh 600
    guest_scp /tmp/ "$definition/configure-guest.sh" "$definition/update-macos.sh"
    guest_ssh 'sudo bash /tmp/configure-guest.sh'
    local status=0
    guest_ssh "sudo bash /tmp/update-macos.sh $MACOS_TARGET $guest_user" \
        < "$secrets/password" || status=$?
    # 255: the restart closed the SSH session. 10: the install ended before
    # the restart. Both wait for the target build.
    if [[ $status == 10 || $status == 255 ]]; then
        wait_for_build "${MACOS_TARGET##*-}" 3600
        status=0
    fi
    [[ $status == 0 ]] || return "$status"
    local running
    running=$(guest_ssh 'echo "$(sw_vers -productVersion)-$(sw_vers -buildVersion)"')
    [[ $running == "$MACOS_TARGET" ]] || {
        echo "the guest runs $running, not $MACOS_TARGET" >&2
        return 1
    }
}

phase_provision() {
    phase_provision_check
    vm_running || vm_start
    wait_ssh 600
    local template port=8765 ssh_port=2222 server tunnel guard status=0
    template=$(python3 "$definition/prepare-template.py" "$work_dir/upstream" \
        --template "$RUNNER_IMAGES_TEMPLATE" --xcode-links "$XCODE_LINKS" \
        --xcode-xip "$XCODE_XIP_NAME" --xcode-sha256 "$XCODE_SHA256")
    # The guest downloads the Xcode archive in place of upstream's storage
    # account. The host serves it on its loopback address, and the tunnel of
    # guest_tunnel brings it to the guest's 127.0.0.1. So the agent's pf
    # rules, which block guests from the host, do not matter. Packer reaches
    # the guest through the same tunnel.
    python3 -m http.server "$port" --bind 127.0.0.1 --directory "$work_dir/inputs/xcode" \
        > "$run_dir/xcode-server.log" 2>&1 &
    server=$!
    guest_tunnel "$ssh_port" "$port" &
    tunnel=$!
    local wait
    for wait in $(seq 60); do
        nc -z 127.0.0.1 "$ssh_port" 2>/dev/null && break
        sleep 1
    done
    space_guard &
    guard=$!
    export PACKER_PLUGIN_PATH="$work_dir/packer-plugins" PACKER_CACHE_DIR="$work_dir/packer-cache"
    # Secrets go through the environment, not the command line.
    PKR_VAR_vm_password=$(cat "$secrets/password")
    PKR_VAR_github_api_pat=${GITHUB_API_PAT:-$(cat "$secrets/github-api-pat" 2>/dev/null || true)}
    export PKR_VAR_vm_password PKR_VAR_github_api_pat
    # Packer's debug log, for SSH and plugin errors. Packer masks the values
    # of sensitive variables, such as the password, in it.
    export PACKER_LOG=1 PACKER_LOG_PATH="$run_dir/packer-debug.log"
    (cd "$(dirname "$template")" && "$work_dir/inputs/packer" build -color=false -timestamp-ui \
        -var "source_vm_name=127.0.0.1" -var "source_vm_port=$ssh_port" \
        -var "build_id=$IMAGE_ID" \
        -var "vm_username=$guest_user" -var "ciproxy_ssh_key=$ssh_key" \
        -var "xcode_install_storage_url=http://127.0.0.1:$port/" -var "xcode_install_sas=build" \
        -var "ciproxy_inputs=$work_dir/inputs" -var "ciproxy_definition=$definition" \
        -var "ciproxy_runner_sha256=$RUNNER_SHA256" "$(basename "$template")") \
        2>&1 | tee "$run_dir/packer.log" || status=$?
    stop_helpers "$server" "$tunnel" "$guard"
    unset PKR_VAR_vm_password PKR_VAR_github_api_pat
    return "$status"
}

phase_provision_check() {
    [[ -f "$work_dir/inputs/xcode/$XCODE_XIP_NAME" ]] || {
        echo "No Xcode archive in $work_dir/inputs/xcode. The guest stays as it is." >&2
        echo "Set XCODE_DIR and run the fetch phase, or build the test image." >&2
        return 1
    }
}

# Auto-login of runner, as upstream's configure-autologin.sh sets it:
# sysadminctl fails over SSH with SACSetAutoLoginPassword error 22. The host
# encodes the password with upstream's kcpassword.py, because the guest has
# no Python before the upstream recipe.
enable_autologin() {
    python3 - "$work_dir/upstream/images/macos/assets/bootstrap-provisioner" \
        "$secrets/password" "$secrets/kcpassword" <<'PY'
import pathlib, sys
sys.path.insert(0, sys.argv[1])
from kcpassword import encode_data
password = pathlib.Path(sys.argv[2]).read_text().strip()
pathlib.Path(sys.argv[3]).write_bytes(bytes(encode_data(password)))
PY
    chmod 600 "$secrets/kcpassword"
    guest_scp /tmp/ciproxy/ "$secrets/kcpassword"
    guest_ssh 'sudo install -m 600 -o root -g wheel /tmp/ciproxy/kcpassword /etc/kcpassword &&
        rm -f /tmp/ciproxy/kcpassword &&
        sudo defaults write /Library/Preferences/com.apple.loginwindow autoLoginUser runner &&
        sudo defaults write /Library/Preferences/com.apple.loginwindow \
            autoLoginUserScreenLocked -bool false &&
        [[ $(sudo defaults read /Library/Preferences/com.apple.loginwindow autoLoginUser) == runner ]] &&
        sudo test -s /etc/kcpassword && echo "auto-login of runner is on"'
}

# Shrink the configured guest to a disk of DISK_BYTES. Opt-in, for a small
# test image. It removes the guest's Recovery container.
phase_shrink() {
    vm_running || vm_start
    wait_ssh 600
    guest_scp /tmp/ "$definition/shrink-disk.sh"
    guest_ssh "sudo bash /tmp/shrink-disk.sh $DISK_BYTES" | tee "$run_dir/shrink.json"
    guest_ssh 'sudo rm -f /tmp/shrink-disk.sh; sudo shutdown -h now' || true
    vm_wait_stopped 300
    python3 -c 'import os, sys; os.truncate(sys.argv[1], int(sys.argv[2]))' \
        "$bundle/disk.img" "$DISK_BYTES"
    # The firmware does not start a guest whose GPT names sectors after the
    # cut, so the host fixes the GPT first. The first start proves that the
    # cut disk boots.
    python3 "$definition/fix-gpt.py" "$bundle/disk.img"
    vm_start
    wait_ssh 900
    guest_ssh 'yes | sudo diskutil repairDisk disk0 >/dev/null; diskutil list disk0'
    guest_ssh 'sudo shutdown -h now' || true
    vm_wait_stopped 300
}

# A test image for the end-to-end test: the configured guest with the stock
# runner, the guest service, and auto-login, without Xcode and the upstream
# toolset. It has the release archive form and a manifest with test_image.
phase_test_image() {
    command -v zstd >/dev/null || { echo "zstd is missing. Install it first." >&2; return 1; }
    vm_running || vm_start
    wait_ssh 600
    local artifacts="$work_dir/artifacts"
    export IMAGE_RELEASE_ID="${IMAGE_RELEASE_ID:-$TEST_IMAGE_ID}"
    guest_ssh 'rm -rf /tmp/ciproxy && mkdir -p /tmp/ciproxy'
    guest_scp /tmp/ciproxy/ "$work_dir/inputs/runner.tar.gz" "$definition/install-runner.sh" \
        "$definition/clean-image.sh"
    scp -q -r "${ssh_options[@]}" "$work_dir/inputs/guest" "$guest_user@$(guest_ip 5):/tmp/ciproxy/"
    guest_ssh "sudo RUNNER_SHA256=$RUNNER_SHA256 bash /tmp/ciproxy/install-runner.sh \
        /tmp/ciproxy/runner.tar.gz && cd /tmp/ciproxy/guest && sudo ./install.sh $IMAGE_RELEASE_ID"
    enable_autologin
    mkdir -p "$artifacts/bundle"
    guest_ssh 'bash -s' < "$definition/guest-facts.sh" > "$artifacts/guest-facts.json"
    cat "$artifacts/guest-facts.json"
    cp "$work_dir/inputs/upstream-software.json" "$artifacts/"
    printf '{"test_image":true,"software":"%s"}\n' \
        "macOS with the stock runner and the ciproxy guest, without Xcode and the toolset" \
        > "$artifacts/software-report.json"
    guest_ssh 'sudo bash /tmp/ciproxy/clean-image.sh && sudo shutdown -h now' || true
    vm_wait_stopped 300
    bundle_artifacts "$artifacts"
    python3 "$definition/write-manifest.py" "$artifacts" --test-image \
        --guest-binary "$work_dir/inputs/guest/ciproxy-guest"
    bash "$definition/package.sh" "$artifacts"
}

phase_facts() {
    vm_running || vm_start
    wait_ssh 600
    mkdir -p "$work_dir/artifacts/bundle"
    guest_ssh 'bash -s' < "$definition/guest-facts.sh" > "$work_dir/artifacts/guest-facts.json"
    cp "$work_dir/upstream/images/image-output/software-report.json" "$work_dir/artifacts/"
    cp "$work_dir/inputs/upstream-software.json" "$work_dir/artifacts/"
    cat "$work_dir/artifacts/guest-facts.json"
}

phase_package() {
    vm_running && { guest_ssh 'sudo shutdown -h now' || true; vm_wait_stopped 300; }
    local artifacts="$work_dir/artifacts"
    mkdir -p "$artifacts/bundle"
    bundle_artifacts "$artifacts"
    python3 "$definition/write-manifest.py" "$artifacts" \
        --guest-binary "$work_dir/inputs/guest/ciproxy-guest"
    bash "$definition/package.sh" "$artifacts"
}

for phase in "${phases[@]}"; do
    printf '== phase %s\n' "$phase"
    "phase_${phase//-/_}"
done
