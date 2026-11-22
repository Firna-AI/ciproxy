# SPDX-License-Identifier: Apache-2.0
# Shared helpers of build.sh: the VM run, the guest address, and SSH.
# build.sh sets definition, work_dir, and vm_tool before it sources this file.

run_dir="$work_dir/run"
secrets="$work_dir/secrets"
bundle="$work_dir/vm"
ssh_key="$secrets/id_ed25519"
guest_user=runner

ssh_options=(-i "$ssh_key" -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null
    -o LogLevel=ERROR -o ConnectTimeout=5 -o ServerAliveInterval=15)

make_secrets() {
    mkdir -p "$secrets"
    chmod 700 "$secrets"
    if [[ ! -f "$secrets/password" ]]; then
        (umask 077; LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 20 > "$secrets/password")
        (umask 077; LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 8 > "$secrets/vnc-password")
        ssh-keygen -q -t ed25519 -N '' -C ciproxy-image-build -f "$ssh_key"
    fi
}

# Start the VM in the background. With "recovery", start macOS Recovery.
vm_start() {
    local mode=${1:-normal}
    mkdir -p "$run_dir"
    rm -f "$run_dir/status.json" "$run_dir/stop"
    local recovery=""
    [[ $mode == recovery ]] && recovery=--recovery
    "$vm_tool" run --bundle "$bundle" ${recovery:+"$recovery"} \
        --vnc-password-file "$secrets/vnc-password" --status-file "$run_dir/status.json" \
        --stop-file "$run_dir/stop" --cpus "${BUILD_CPUS:-4}" --memory-gib "${BUILD_MEMORY_GIB:-8}" \
        >> "$run_dir/vm.log" 2>&1 &
    echo $! > "$run_dir/vm.pid"
    for _ in $(seq 120); do
        [[ -s "$run_dir/status.json" ]] && return 0
        vm_running || { tail -n 20 "$run_dir/vm.log" >&2; return 1; }
        sleep 1
    done
    echo "the VM did not report its VNC port" >&2
    return 1
}

vm_running() {
    [[ -f "$run_dir/vm.pid" ]] && kill -0 "$(cat "$run_dir/vm.pid")" 2>/dev/null
}

# Wait until the guest turns itself off. Stop it after the timeout.
vm_wait_stopped() {
    local timeout=${1:-600}
    for _ in $(seq "$timeout"); do
        vm_running || return 0
        sleep 1
    done
    echo "the guest did not turn off in $timeout s, so the build stops it" >&2
    vm_stop
    return 1
}

vm_stop() {
    vm_running || return 0
    touch "$run_dir/stop"
    for _ in $(seq 60); do
        vm_running || return 0
        sleep 1
    done
    kill "$(cat "$run_dir/vm.pid")" 2>/dev/null || true
}

vnc_port() {
    python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["vnc_port"])' \
        "$run_dir/status.json"
}

guest_ip() {
    "$vm_tool" ip --bundle "$bundle" --wait-seconds "${1:-300}"
}

guest_ssh() {
    ssh "${ssh_options[@]}" "$guest_user@$(guest_ip 5)" "$@"
}

guest_scp() {
    local target=$1
    shift
    scp -q "${ssh_options[@]}" "$@" "$guest_user@$(guest_ip 5):$target"
}

# Wait until SSH answers. A restart in the guest can take some minutes.
wait_ssh() {
    local timeout=${1:-900} started=$SECONDS
    guest_ip "$timeout" >/dev/null
    while (( SECONDS - started < timeout )); do
        guest_ssh true 2>/dev/null && return 0
        sleep 5
    done
    echo "no SSH to the guest in $timeout s" >&2
    return 1
}

# The host address on the VM network, from the guest's default route.
host_address() {
    guest_ssh "route -n get default | awk '/gateway:/ {print \$2}'"
}
