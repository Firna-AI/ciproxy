#!/usr/bin/env bash
# Private runtime inputs must not enter shell traces.
set +x
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: container-smoke.sh IMAGE [--network NETWORK] [--add-host HOST:ADDRESS]

Run a local image with a read-only root file system and mounted secrets.
Set CIPROXY_RUNTIME_ENV_FILE to a private Docker environment file.
Set CIPROXY_RUNTIME_SECRETS_DIR to a private runtime secret directory.
The command checks readiness, liveness, and a 300-second graceful stop.
It removes its container and does not print application logs.
USAGE
}

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

if [[ ${1:-} == --help || ${1:-} == -h ]]; then
    usage
    exit 0
fi
[[ $# -gt 0 ]] || { usage >&2; exit 2; }
image=$1
shift
[[ -n $image && $image != -* && $image != *[[:space:]]* ]] \
    || fail 'IMAGE must be a local Docker image name.'

network=bridge
container_options=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            [[ $# -ge 2 && -n $2 && $2 != -* ]] \
                || fail '--network requires a value.'
            network=$2
            shift 2
            ;;
        --add-host)
            [[ $# -ge 2 && -n $2 && $2 != -* ]] \
                || fail '--add-host requires a value.'
            container_options+=("$1" "$2")
            shift 2
            ;;
        *) fail 'Unknown option. Use --help for the supported options.' ;;
    esac
done

[[ -n ${CIPROXY_RUNTIME_ENV_FILE:-} && -n ${CIPROXY_RUNTIME_SECRETS_DIR:-} ]] \
    || fail 'Set CIPROXY_RUNTIME_ENV_FILE and CIPROXY_RUNTIME_SECRETS_DIR.'
[[ -f $CIPROXY_RUNTIME_ENV_FILE && -r $CIPROXY_RUNTIME_ENV_FILE ]] \
    || fail 'The runtime environment file must be a readable regular file.'
[[ -d $CIPROXY_RUNTIME_SECRETS_DIR ]] \
    || fail 'The runtime secrets path must be a directory.'
secrets_directory=$(cd -- "$CIPROXY_RUNTIME_SECRETS_DIR" && pwd -P)

container_name="ciproxy-control-plane-smoke-$$-$RANDOM-$RANDOM"
container_id=''
container_stopped=false
cleanup() {
    exit_status=$?
    trap - EXIT
    if [[ -z $container_id ]] && [[ $(docker inspect --format \
        '{{index .Config.Labels "ciproxy.container-smoke"}}' \
        "$container_name" 2>/dev/null || true) == "$container_name" ]]; then
        container_id=$container_name
    fi
    if [[ -n $container_id ]]; then
        if ! $container_stopped; then
            docker stop --time 300 "$container_id" >/dev/null 2>&1 || true
        fi
        if ! docker rm --force "$container_id" >/dev/null 2>&1; then
            printf 'Could not remove smoke container %s.\n' "$container_name" >&2
            if [[ $exit_status -eq 0 ]]; then
                exit_status=1
            fi
        fi
    fi
    exit "$exit_status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if [[ $network != host ]]; then
    container_options+=(--publish 127.0.0.1::8080)
fi
if ! container_id=$(docker create --pull never --name "$container_name" \
    --label "ciproxy.container-smoke=$container_name" \
    --network "$network" \
    --read-only --cap-drop ALL --security-opt no-new-privileges \
    --env-file "$CIPROXY_RUNTIME_ENV_FILE" \
    --env HTTP_BIND=0.0.0.0:8080 \
    --env SECRETS_DIRECTORY=/var/run/secrets/ciproxy \
    --mount "type=bind,source=$secrets_directory,target=/var/run/secrets/ciproxy,readonly" \
    "${container_options[@]}" "$image" 2>/dev/null); then
    fail 'Could not create the smoke container. Check the local image and private inputs.'
fi
docker start "$container_id" >/dev/null
[[ $(docker inspect --format '{{.State.Running}}' "$container_id") == true ]] \
    || fail 'The container stopped before its health checks.'

health_address=127.0.0.1:8080
if [[ $network != host ]]; then
    health_address=$(docker port "$container_id" 8080/tcp)
fi
curl --fail --silent --show-error --output /dev/null \
    --retry 150 --retry-connrefused --retry-delay 2 --retry-max-time 300 \
    --max-time 5 "http://$health_address/api/health/ready"
curl --fail --silent --show-error --output /dev/null \
    --max-time 5 "http://$health_address/api/health/live"
[[ $(docker inspect --format '{{.State.Running}}' "$container_id") == true ]] \
    || fail 'The container stopped during its health checks.'

docker stop --time 300 "$container_id" >/dev/null
container_stopped=true
exit_result=$(docker inspect --format '{{.State.ExitCode}} {{.State.OOMKilled}}' \
    "$container_id")
[[ $exit_result == '0 false' ]] \
    || fail "Container exit result was $exit_result. Expected: 0 false."
docker rm "$container_id" >/dev/null
container_id=''
trap - EXIT
printf 'Container smoke passed: ready, live, exit 0, OOMKilled false.\n'
