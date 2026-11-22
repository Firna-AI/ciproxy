# SPDX-License-Identifier: Apache-2.0
# shellcheck shell=bash
# Shell helpers that run the AWS CLI v2 against Cloudflare R2. Source this file.
# The R2 keys reach only the environment of each aws process. A temporary AWS
# config file for each call sets the part size, so the user's AWS config stays
# unchanged. Do not trace a script that sources this file: a trace prints keys.

R2_PART_BYTES=$((128 * 1024 * 1024))
# The first AWS CLI v2 release with --if-none-match on s3api put-object.
R2_AWS_MIN_VERSION=2.17.34

# r2_require VARIABLE...: stop unless R2_ACCOUNT_ID, each named variable, and
# a new enough AWS CLI v2 are present.
r2_require() {
    local name
    for name in R2_ACCOUNT_ID "$@"; do
        if [[ -z ${!name:-} ]]; then
            echo "Set $name. Source the R2 env file of the build Mac first." >&2
            return 1
        fi
    done
    r2_require_aws
}

r2_require_aws() {
    local version
    if ! version=$(aws --version 2>&1); then
        echo "Install the AWS CLI v2, version $R2_AWS_MIN_VERSION or later." >&2
        return 1
    fi
    version=${version#aws-cli/}
    version=${version%% *}
    if ! r2_version_at_least "$version" "$R2_AWS_MIN_VERSION"; then
        echo "The AWS CLI is $version. Install version $R2_AWS_MIN_VERSION or later of v2." >&2
        return 1
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
