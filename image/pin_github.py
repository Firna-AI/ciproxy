# SPDX-License-Identifier: Apache-2.0
"""Match exact release versions with one tag rule for host and guest lookups."""

import re
import shlex


VERSION_PATTERN = r"(?<![0-9.])([0-9]+(?:\.[0-9]+)*(?:[-+][0-9A-Za-z.-]+)?)$"


def exact_version(version):
    value = version.removeprefix("v")
    return value if re.fullmatch(VERSION_PATTERN, value) else None


def matching_release(releases, version):
    expected = exact_version(version)
    for release in releases:
        match = re.search(VERSION_PATTERN, release.get("tag_name", ""))
        if match and match[1] == expected:
            return release
    return None


def lookup_script():
    return '''
# Shared exact release selections. Metadata can be prepared on the build host.
get_github_releases_by_version() {
    local repo=$1 version=${2:-} tag pattern page=1 releases release
    tag=$(jq -r --arg repo "$repo" '.github[$repo] // empty' "$HELPER_SCRIPTS/ciproxy-pins.json")
    if [[ -n "$tag" ]]; then
        get_github_api_json "repos/${repo}/releases/tags/${tag}"
        return $?
    fi
    pattern=@PATTERN@
    version=$(jq -nr --arg version "$version" --arg pattern "$pattern" \\
        '$version | ltrimstr("v") | select(test("^" + $pattern))')
    if [[ -z "$version" ]]; then
        upstream_get_github_releases_by_version "$@"
        return $?
    fi
    while true; do
        releases=$(get_github_api_json "repos/${repo}/releases?per_page=100&page=${page}") || return $?
        release=$(jq -c --arg version "$version" --arg pattern "$pattern" \\
            '[.[] | select((.tag_name | match($pattern).captures[0].string) == $version)][0] // empty' \\
            <<< "$releases") || return $?
        if [[ -n "$release" ]]; then printf '%s\\n' "$release"; return 0; fi
        if [[ $(jq 'length' <<< "$releases") == 0 ]]; then break; fi
        page=$((page + 1))
    done
    printf 'No exact GitHub release found for %s.\\n' "$repo" >&2
    return 1
}
'''.replace("@PATTERN@", shlex.quote(VERSION_PATTERN))
