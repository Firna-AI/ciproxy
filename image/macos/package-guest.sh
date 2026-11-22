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
release="$repository/target/aarch64-apple-darwin/release"
install -m 0755 "$release/ciproxy-guest" "$release/ciproxy-cache-sidecar" "$out/"
install -m 0644 "$work_dir/patched-worker-osx-arm64/Runner.Worker.dll" "$out/"
install -m 0644 "$definition/guest/dev.ciproxy.guest.plist" "$definition/../LICENSE" "$out/"
install -m 0755 "$definition/guest/install.sh" "$definition/guest/start.sh" \
    "$definition/guest/runner-entry.sh" "$out/"
echo "macOS guest folder: $out"
