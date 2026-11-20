# SPDX-License-Identifier: Apache-2.0
"""Render one pinned software recipe for machine and Boat builds."""

import hashlib
import json
import pathlib
import shutil
import urllib.request

from pin_dynamic import apply_dynamic
from pin_packages import apply_packages
from pin_scripts import apply_scripts
from pin_toolset import apply_toolset


PINNED_RECIPE_DIRECTORY = dict(line.split("=", 1) for line in
    pathlib.Path(__file__).with_name("paths.env").read_text().splitlines()
    if line and not line.startswith("#"))["PINNED_RECIPE_DIRECTORY"]


def render(upstream, output):
    shutil.copytree(upstream, output, dirs_exist_ok=True)
    definition = pathlib.Path(__file__).parent
    pins = json.loads((definition / "software-pins.json").read_text())
    apt = json.loads((definition / "software-apt-pins.json").read_text())
    apply_toolset(output, pins)
    apply_packages(output, pins, apt)
    apply_dynamic(output, pins)
    apply_scripts(output, pins)
    helper = output / "images/ubuntu/scripts/helpers/install.sh"
    content = helper.read_text().replace("get_github_api_json() {", "get_github_api_json_network() {", 1)
    content += '''
get_github_api_json() {
    local api_path=$1 cache key
    key=$(printf '%s' "$api_path" | sha256sum | cut -d' ' -f1)
    cache="$HELPER_SCRIPTS/github-cache/$key.json"
    if [[ -f "$cache" ]]; then cat "$cache"; return 0; fi
    mkdir -p "$HELPER_SCRIPTS/github-cache"
    get_github_api_json_network "$api_path" > "$cache.tmp" || return $?
    mv "$cache.tmp" "$cache"
    cat "$cache"
}
'''
    helper.write_text(content)
    return pins


def warm_github_cache(upstream, pins):
    directory = upstream / "images/ubuntu/scripts/helpers/github-cache"
    directory.mkdir(parents=True, exist_ok=True)
    for repo, tag in pins["github"].items():
        route = "repos/" + repo + "/releases/tags/" + tag
        destination = directory / (hashlib.sha256(route.encode()).hexdigest() + ".json")
        if not destination.exists():
            request = urllib.request.Request("https://api.github.com/" + route,
                                             headers={"User-Agent": "ciproxy-image-build"})
            with urllib.request.urlopen(request, timeout=60) as response:
                destination.write_bytes(response.read())
