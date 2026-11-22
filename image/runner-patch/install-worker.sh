#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Install the Worker overlay of worker-overlay.py into an official runner, then
# check the result. Run as root in the image, on Linux or macOS. It needs no
# Python, and it runs on the Bash 3.2 of macOS.
set -euo pipefail

usage="Usage: install-worker.sh RUNNER_DIR OVERLAY_DIR"
runner=${1:?$usage}
overlay=${2:?$usage}
bin=$runner/bin
deps=$bin/Runner.Worker.deps.json

fail() {
    echo "install-worker.sh: $*" >&2
    exit 1
}

sha256() {
    if command -v shasum >/dev/null; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    else
        sha256sum "$1" | cut -d ' ' -f 1
    fi
}

# The overlay deps file is the runner's own deps file with a few package
# entries changed. So it fits only the runner that it was made from. A second
# run finds the overlay deps file already in place.
if ! cmp -s "$overlay/Runner.Worker.deps.json" "$deps"; then
    if [[ $(sha256 "$deps") != "$(cat "$overlay/stock-deps.sha256")" ]]; then
        fail "the runner in $runner is not the runner that the overlay was made from"
    fi
fi
if [[ ! -f $bin/Runner.Worker.stock.dll ]]; then
    install -m 0644 "$bin/Runner.Worker.dll" "$bin/Runner.Worker.stock.dll"
fi
install -m 0644 "$overlay/Runner.Worker.dll" "$bin/Runner.Worker.own.dll"
while IFS= read -r name; do
    case $name in
        "" | .* | */*) fail "invalid overlay file name: $name" ;;
    esac
    install -m 0644 "$overlay/$name" "$bin/$name"
done < "$overlay/install-files.txt"

# Check the runner after the copy: each overlay file is in place, and the deps
# file names only the raised version of each raised package.
while IFS= read -r name; do
    cmp -s "$overlay/$name" "$bin/$name" || fail "$bin/$name differs from the overlay"
done < "$overlay/install-files.txt"
while read -r package version; do
    pattern=${package//./\\.}
    found=$(grep -o "\"$pattern/[^\"]*\"" "$deps" | sort -u || true)
    [[ $found == "\"$package/$version\"" ]] || fail "$deps lists $package as: $found"
    found=$(grep -o "\"$pattern\": \"[^\"]*\"" "$deps" | sort -u || true)
    [[ -z $found || $found == "\"$package\": \"$version\"" ]] ||
        fail "$deps needs $package as: $found"
done < "$overlay/packages.txt"
echo "Worker overlay installed in $runner"
