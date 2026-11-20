# SPDX-License-Identifier: Apache-2.0
"""Resolve exact releases through the rendered cached helper without a network."""

import hashlib
import io
import json
import os
import shlex
import subprocess
import urllib.parse
from unittest.mock import patch

from pin_recipe import warm_github_cache
from _checks_.recipe_fixture import rendered


def responses(root):
    values = {
        "repos/Kitware/CMake/releases?per_page=100&page=1": [
            *({"tag_name": "v9.9." + str(index)} for index in range(98)),
            {"tag_name": "v13.31.6"}, {"tag_name": "v3.31.60"}],
        "repos/Kitware/CMake/releases?per_page=100&page=2": [{"tag_name": "v3.31.6", "assets": [
            {"browser_download_url": "https://example.invalid/cmake-3.31.6-linux-x86_64.sh"}]}],
        "repos/kubernetes/minikube/releases?per_page=100&page=1": [
            *({"tag_name": "v2.0." + str(index)} for index in range(99)), {"tag_name": "v1.139.0"}],
        "repos/kubernetes/minikube/releases?per_page=100&page=2": [{"tag_name": "release-v1.39.0", "assets": []}],
        "repos/fixture/pinned/releases/tags/fixed-tag": {"tag_name": "fixed-tag", "assets": []},
    }
    directory = root / "responses"
    directory.mkdir()
    for route, value in values.items():
        (directory / (hashlib.sha256(route.encode()).hexdigest() + ".json")).write_text(json.dumps(value))
    return directory, values


def invoke(upstream, directory, calls, command):
    helpers = upstream / "images/ubuntu/scripts/helpers"
    script = '''source "$HELPER_SCRIPTS/install.sh"
get_github_api_json_network() {
    printf '%s\\n' "$1" >> "$FIXTURE_CALLS"
    local key
    key=$(printf '%s' "$1" | sha256sum | cut -d' ' -f1)
    if [[ ! -f "$FIXTURE_RESPONSES/$key.json" ]]; then
        printf 'No fixture response\\n' >&2
        return 7
    fi
    cat "$FIXTURE_RESPONSES/$key.json"
}
''' + command
    return subprocess.run(["bash", "-e", "-c", script], capture_output=True, text=True,
                          env={**os.environ, "HELPER_SCRIPTS": str(helpers), "FIXTURE_RESPONSES": str(directory),
                               "FIXTURE_CALLS": str(calls)})


def check_lookup(case, workspace):
    root, upstream, pins = rendered(workspace)
    helpers = upstream / "images/ubuntu/scripts/helpers"
    pins["github"].pop("kubernetes/minikube", None)
    pins["github"]["fixture/pinned"] = "fixed-tag"
    (helpers / "ciproxy-pins.json").write_text(json.dumps(pins))
    directory, _ = responses(root)
    calls = root / "calls"
    result = invoke(upstream, directory, calls,
                    'resolve_github_release_asset_url Kitware/CMake \'endswith("linux-x86_64.sh")\' 3.31.6')
    case.assertEqual(result.returncode, 0, result.stderr)
    case.assertEqual(result.stdout.strip(), "https://example.invalid/cmake-3.31.6-linux-x86_64.sh")
    case.assertEqual(calls.read_text().splitlines(), ["repos/Kitware/CMake/releases?per_page=100&page=1",
                                                    "repos/Kitware/CMake/releases?per_page=100&page=2"])
    before = calls.read_bytes()
    cached = invoke(upstream, directory, calls, 'get_github_releases_by_version Kitware/CMake 3.31.6')
    case.assertEqual(cached.returncode, 0, cached.stderr)
    case.assertEqual(json.loads(cached.stdout)["tag_name"], "v3.31.6")
    case.assertEqual(calls.read_bytes(), before)
    prefixed = invoke(upstream, directory, calls, 'get_github_releases_by_version kubernetes/minikube v1.39.0')
    case.assertEqual(prefixed.returncode, 0, prefixed.stderr)
    case.assertEqual(json.loads(prefixed.stdout)["tag_name"], "release-v1.39.0")
    pinned = invoke(upstream, directory, calls, 'get_github_releases_by_version fixture/pinned latest')
    case.assertEqual(pinned.returncode, 0, pinned.stderr)
    case.assertEqual(json.loads(pinned.stdout)["tag_name"], "fixed-tag")
    before = calls.read_bytes()
    for version, prerelease, assets in (("latest", "true", "false"), ("3.*", "false", "true")):
        arguments = ["fixture/unpinned", version, prerelease, assets]
        command = '''preserved=false
if declare -F upstream_get_github_releases_by_version > /dev/null; then preserved=true; fi
upstream_get_github_releases_by_version() {
    jq -cn --argjson preserved "$preserved" --args \\
        '{preserved: $preserved, arguments: $ARGS.positional}' "$@"
}
get_github_releases_by_version ''' + " ".join(shlex.quote(value) for value in arguments)
        fallback = invoke(upstream, directory, calls, command)
        case.assertEqual(fallback.returncode, 0, fallback.stderr)
        case.assertEqual(json.loads(fallback.stdout), {"preserved": True, "arguments": arguments})
    case.assertEqual(calls.read_bytes(), before)


def check_warm_lookup(case, workspace):
    root, upstream, _ = rendered(workspace)
    directory, values = responses(root)
    routes = []

    def open_fixture(request, timeout):
        parsed = urllib.parse.urlsplit(request.full_url)
        route = parsed.path.lstrip("/") + ("?" + parsed.query if parsed.query else "")
        routes.append(route)
        return io.BytesIO(json.dumps(values[route]).encode())

    with patch("pin_recipe.urllib.request.urlopen", side_effect=open_fixture):
        warm_github_cache(upstream, {"github": {}})
    case.assertEqual(routes, ["repos/Kitware/CMake/releases?per_page=100&page=1",
                             "repos/Kitware/CMake/releases?per_page=100&page=2"])
    calls = root / "calls"
    result = invoke(upstream, directory, calls, 'get_github_releases_by_version Kitware/CMake 3.31.6')
    case.assertEqual(result.returncode, 0, result.stderr)
    case.assertEqual(json.loads(result.stdout)["tag_name"], "v3.31.6")
    case.assertFalse(calls.exists(), "The warmed exact lookup must not use the network boundary.")
