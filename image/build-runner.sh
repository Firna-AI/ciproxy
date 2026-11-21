#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/pins.env"
work_dir=$(realpath "${1:?Usage: image/build-runner.sh WORK_DIR}")
commit=d1eb12d4b115c301201de27d667284e1a80f2ab5
digest=a0fa836ec067173a6de1d5afc1ae7c9f5e23cd576884385d6d020c7a529c6e5e
source_dir="$work_dir/runner-$commit"
archive="$work_dir/runner-source.tar.gz"
if [[ ! -f "$archive" ]]; then
    curl -fsSL "https://codeload.github.com/actions/runner/tar.gz/$commit" -o "$archive"
fi
printf '%s  %s\n' "$digest" "$archive" | sha256sum --check
if [[ ! -d "$source_dir" ]]; then tar -xzf "$archive" -C "$work_dir"; fi
python3 "$definition/runner-patch/prepare-source.py" "$source_dir"
(
    cd "$source_dir/src"
    dotnet publish Runner.Worker/Runner.Worker.csproj -c Release -r linux-x64 \
        --self-contained false -p:Version="$RUNNER_VERSION" -p:PackageRuntime=linux-x64 \
        -o "$work_dir/patched-worker"
)
