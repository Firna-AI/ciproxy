# SPDX-License-Identifier: MIT
# Pinned runner-images helper functions for renderer regressions.

get_github_api_json() {
    local api_path=$1
    local url="https://api.github.com/${api_path}"
    local response_path
    local http_code

    response_path=$(mktemp)
    interval=15

    for ((retries=5; retries>0; retries--)); do
        if http_code=$(curl -sSL -o "$response_path" -w '%{http_code}' "$url"); then
            if [ "$http_code" -eq 200 ]; then
                cat "$response_path"
                rm -f "$response_path"
                return 0
            fi
            echo "Request to ${url} returned HTTP status code ${http_code}: $(head -c 500 "$response_path")" >&2
            if [[ ! "$http_code" =~ ^(403|429|5[0-9][0-9])$ ]]; then
                rm -f "$response_path"
                return 1
            fi
        else
            echo "Request to ${url} failed" >&2
        fi

        if [ "$retries" -le 1 ]; then
            echo "Request to ${url} failed after all attempts" >&2
            rm -f "$response_path"
            return 1
        fi

        echo "Waiting $interval seconds before retrying (retries left: $retries)..." >&2
        sleep $interval
    done
}

get_github_releases_by_version() {
    local repo=$1
    local version=${2:-".+"}
    local allow_pre_release=${3:-false}
    local with_assets_only=${4:-false}

    page_size="100"

    if ! json=$(get_github_api_json "repos/${repo}/releases?per_page=${page_size}") || [[ -z "$json" ]]; then
        echo "Failed to get releases from ${repo}" >&2
        exit 1
    fi

    if [[ $with_assets_only == "true" ]]; then
        json=$(echo $json | jq -r '.[] | select(.assets | length > 0)')
    else
        json=$(echo $json | jq -r '.[]')
    fi

    if [[ $allow_pre_release == "true" ]]; then
        json=$(echo $json | jq -r '.')
    else
        json=$(echo $json | jq -r '. | select(.prerelease==false)')
    fi

    # Filter out rc/beta/etc releases, convert to numeric version and sort
    json=$(echo $json | jq '. | select(.tag_name | test(".*-[a-z]|beta") | not)' | jq '.tag_name |= gsub("[^\\d.]"; "")' | jq -s 'sort_by(.tag_name | split(".") | map(tonumber))')

    # Select releases matching version
    if [[ $version == "latest" ]]; then
        json_filtered=$(echo $json | jq .[-1])
    elif [[ $version == *"+"* ]] || [[ $version == *"*"* ]]; then
        json_filtered=$(echo $json | jq --arg version $version '.[] | select(.tag_name | test($version))')
    else
        json_filtered=$(echo $json | jq --arg version $version '.[] | select(.tag_name | contains($version))')
    fi

    if [[ -z "$json_filtered" ]]; then
        echo "Failed to get releases from ${repo} matching version ${version}" >&2
        echo "Available versions: $(echo "$json" | jq -r '.tag_name')" >&2
        exit 1
    fi

    echo $json_filtered
}

resolve_github_release_asset_url() {
    local repo=$1
    local url_filter=$2
    local version=${3:-".+"}
    local allow_pre_release=${4:-false}
    local allow_multiple_matches=${5:-false}

    matching_releases=$(get_github_releases_by_version "${repo}" "${version}" "${allow_pre_release}" "true")
    if [[ -z "$matching_releases" ]]; then
        exit 1
    fi
    matched_url=$(echo $matching_releases | jq -r ".assets[].browser_download_url | select(${url_filter})")

    if [[ -z "$matched_url" ]]; then
        echo "Found no download urls matching pattern: ${url_filter}" >&2
        echo "Available download urls: $(echo "$matching_releases" | jq -r '.assets[].browser_download_url')" >&2
        exit 1
    fi

    if [[ "$(echo "$matched_url" | wc -l)" -gt 1 ]]; then
        if [[ $allow_multiple_matches == "true" ]]; then
            matched_url=$(echo "$matched_url" | tail -n 1)
        else
            echo "Multiple matches found for ${version} version and ${url_filter} URL filter. Please make filters more specific" >&2
            exit 1
        fi
    fi

    echo $matched_url
}

