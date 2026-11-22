#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# Build the macOS guest folder that the image build installs. Run on an
# Apple Silicon Mac with the pinned Rust toolchain and the .NET SDK.
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repository=$(cd "$definition/../.." && pwd)
out=${1:?Usage: image/macos/package-guest.sh OUT_DIR}
if [[ $(uname -s) != Darwin || $(uname -m) != arm64 ]]; then
    echo "package the macOS guest on an Apple Silicon Mac" >&2
    exit 2
fi
work_dir="$repository/.context/ciproxy-macos-guest"
mkdir -p "$work_dir" "$out"
cargo build --manifest-path "$repository/Cargo.toml" --release \
    --target aarch64-apple-darwin --package ciproxy-guest --package ciproxy-cache-sidecar
bash "$repository/image/build-runner.sh" "$work_dir" osx-arm64
# The Worker overlay changes the stock runner's own deps file, so it needs the
# stock runner that the image installs.
runner_url=https://github.com/actions/runner/releases/download/v2.338.2/actions-runner-osx-arm64-2.338.2.tar.gz
runner_sha256=25a065af409ffa317ab4d5a3b614dd15fc9c40f2172bc1b35a1b1258e919a388
runner_archive="$work_dir/runner-osx-arm64.tar.gz"
if [[ ! -f $runner_archive ]]; then
    curl -fsSL "$runner_url" -o "$runner_archive.part"
    mv "$runner_archive.part" "$runner_archive"
fi
printf '%s  %s\n' "$runner_sha256" "$runner_archive" | shasum -a 256 -c
python3 "$repository/image/runner-patch/worker-overlay.py" \
    --worker-output "$work_dir/patched-worker-osx-arm64" \
    --runner-archive "$runner_archive" --output "$out/worker"
release="$repository/target/aarch64-apple-darwin/release"
install -m 0755 "$release/ciproxy-guest" "$release/ciproxy-cache-sidecar" "$out/"
install -m 0644 "$definition/guest/dev.ciproxy.guest.plist" "$definition/../LICENSE" "$out/"
install -m 0755 "$definition/guest/install.sh" "$definition/guest/start.sh" \
    "$definition/guest/runner-entry.sh" "$repository/image/runner-patch/install-worker.sh" "$out/"
echo "macOS guest folder: $out"
