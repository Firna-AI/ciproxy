# SPDX-License-Identifier: Apache-2.0
# shellcheck shell=bash
# Shell helpers that run the AWS CLI v2 against Cloudflare R2. Source this file.
# Sourcing it removes the export flag of the R2 key variables, so the keys reach
# only the environment of each aws process. A temporary AWS
# config file for each call sets the part size, so the user's AWS config stays
# unchanged. No line here expands a key under a trace: the presence tests use
# ${name:+set}, and r2_aws turns the trace off in its own subshell.

export -n R2_PUBLISH_ACCESS_KEY_ID R2_PUBLISH_SECRET_ACCESS_KEY \
    R2_XCODE_ACCESS_KEY_ID R2_XCODE_SECRET_ACCESS_KEY

R2_PART_BYTES=$((128 * 1024 * 1024))

# r2_missing VARIABLE...: print the names of the variables that are empty.
r2_missing() {
    local name missing=()
    for name in "$@"; do
        [[ -n ${!name:+set} ]] || missing+=("$name")
    done
    echo "${missing[*]:-}"
}

# r2_require VARIABLE...: stop unless R2_ACCOUNT_ID and each named variable
# are set.
r2_require() {
    local missing
    missing=$(r2_missing R2_ACCOUNT_ID "$@")
    if [[ -n $missing ]]; then
        echo "Set $missing. Source the R2 env file of the build Mac first." >&2
        return 1
    fi
}

# r2_require_aws [MINIMUM]: stop unless aws runs. With MINIMUM, also stop
# unless it is AWS CLI v2 at MINIMUM or later. Without it, warn on a CLI that
# is not v2 and continue.
r2_require_aws() {
    local minimum=${1:-} version
    if ! version=$(aws --version 2>&1); then
        echo "Install the AWS CLI v2${minimum:+, version $minimum or later}." >&2
        return 1
    fi
    version=${version#aws-cli/}
    version=${version%% *}
    if [[ -n $minimum ]] && ! r2_version_at_least "$version" "$minimum"; then
        echo "The AWS CLI is $version. Install version $minimum or later of v2." >&2
        return 1
    fi
    if [[ -z $minimum && $version != 2.* ]]; then
        echo "Warning: the AWS CLI is $version. The R2 scripts expect the AWS CLI v2." >&2
    fi
}

# r2_version_at_least VERSION MINIMUM: true when VERSION is MINIMUM or later
# and has the same major version.
r2_version_at_least() {
    local -a have want
    local index
    IFS=. read -r -a have <<<"$1"
    IFS=. read -r -a want <<<"$2"
    [[ ${have[0]:-} == "${want[0]}" ]] || return 1
    for index in 1 2; do
        [[ ${have[index]:-} =~ ^[0-9]+$ ]] || return 1
        ((have[index] > want[index])) && return 0
        ((have[index] < want[index])) && return 1
    done
    return 0
}

# r2_aws KEY_ID_VARIABLE SECRET_VARIABLE ARGUMENT...: run aws against the R2
# endpoint of R2_ACCOUNT_ID with the keys in the two named variables.
r2_aws() {
    local key_id_variable=$1 secret_variable=$2
    shift 2
    (
        set +x
        config=$(mktemp "${TMPDIR:-/tmp}/ciproxy-r2-aws.XXXXXX")
        trap 'rm -f "$config"' EXIT
        printf '[default]\ns3 =\n    preferred_transfer_client = classic\n    multipart_threshold = %s\n    multipart_chunksize = %s\n' \
            "$R2_PART_BYTES" "$R2_PART_BYTES" >"$config"
        # A profile or a session token of the user would mix with the R2 keys.
        unset AWS_PROFILE AWS_DEFAULT_PROFILE AWS_SESSION_TOKEN AWS_SECURITY_TOKEN
        export AWS_CONFIG_FILE=$config AWS_SHARED_CREDENTIALS_FILE=$config.none
        export AWS_ACCESS_KEY_ID=${!key_id_variable}
        export AWS_SECRET_ACCESS_KEY=${!secret_variable}
        # Cloudflare advises these two settings for R2 with AWS CLI 2.23.0 or later.
        export AWS_REQUEST_CHECKSUM_CALCULATION=when_required
        export AWS_RESPONSE_CHECKSUM_VALIDATION=when_required
        export AWS_PAGER=
        aws --endpoint-url "https://$R2_ACCOUNT_ID.r2.cloudflarestorage.com" --region auto "$@"
    )
}
