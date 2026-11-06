#!/usr/bin/env bash
# Keep registry credentials out of shell traces, also with `bash -x`.
set +x
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: container-image.sh IMAGE [--push] [--network NETWORK] [--platform PLATFORM]

Build the control-plane image from the repository root context.
--push reads Docker config JSON from CIPROXY_REGISTRY_AUTH_FILE.
Set CIPROXY_BUILD_REVISION to add the source revision to the image label.
Docker uses its usual DOCKER_HOST and TLS environment settings.
This command does not deploy the image.
USAGE
}

fail() {
    printf '%s\n' "$1" >&2
    exit 2
}

if [[ ${1:-} == --help || ${1:-} == -h ]]; then
    usage
    exit 0
fi
[[ $# -gt 0 ]] || { usage >&2; exit 2; }
image=$1
shift
[[ -n $image && $image != -* && $image != *[[:space:]]* && $image != *@* ]] \
    || fail 'IMAGE must be a Docker image name with an optional tag.'

push=false
build_options=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            push=true
            shift
            ;;
        --network|--platform)
            [[ $# -ge 2 && -n $2 && $2 != -* ]] \
                || fail 'Each build option requires a value.'
            build_options+=("$1" "$2")
            shift 2
            ;;
        *) fail 'Unknown option. Use --help for the supported options.' ;;
    esac
done

if $push; then
    [[ -n ${CIPROXY_REGISTRY_AUTH_FILE:-} ]] \
        || fail 'Set CIPROXY_REGISTRY_AUTH_FILE to a private Docker config file.'
    [[ -f $CIPROXY_REGISTRY_AUTH_FILE && -r $CIPROXY_REGISTRY_AUTH_FILE ]] \
        || fail 'The registry credentials file must be a readable regular file.'
fi

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repository_root=$(cd -- "$script_directory/../../.." && pwd)
docker build \
    --file "$repository_root/crates/ciproxy-control-plane/Dockerfile" \
    --tag "$image" \
    --build-arg "BUILD_REVISION=${CIPROXY_BUILD_REVISION:-unknown}" \
    "${build_options[@]}" "$repository_root"
printf 'Built %s\n' "$image"

if ! $push; then
    exit 0
fi

registry_config_directory=''
cleanup() {
    if [[ -n $registry_config_directory ]]; then
        rm -rf -- "$registry_config_directory"
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
umask 077
registry_config_directory=$(mktemp -d "${TMPDIR:-/tmp}/ciproxy-registry.XXXXXXXX")
cp -- "$CIPROXY_REGISTRY_AUTH_FILE" "$registry_config_directory/config.json"
chmod 600 "$registry_config_directory/config.json"
docker --config "$registry_config_directory" push "$image"
printf 'Pushed %s. Use its registry digest for deployment.\n' "$image"
