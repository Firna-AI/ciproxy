# Agent integration checks and packaging

Run these commands from the repository root.
The [agent guide](README.md#development) gives the unit and compile checks.

## Image download checks

The default `image_download_regression` target uses the production client
builder with a local HTTP server. It uses a controlled clock for read limits.
It checks that a body with regular data outlasts the former total limit.
It checks that a stalled body fails at the read limit.
HTTPS remains required by the production client and download adapter.

```sh
cargo test --package ciproxy-agent --test image_download_regression
```

## Docker, image, lock, and state checks

The default `docker_regression` target runs production code in temporary
folders. It needs `e2fsprogs`, without root access. It covers four areas:

- Docker leases, preparation, and recovery call the real adapter. They mock
  cache administration and most host commands. The corruption case runs
  real `e2fsck` on temporary files. Admission checks reclaim idle bases
  before disk creation and each growth step.
- Image recovery keeps the runtime claim of its caller while it creates the
  image folder. Image accounting excludes VM drive files and counts jail
  support files.
- Run refuses a second runtime claim until its owner ends. Uninstall prints
  one notice and waits for that claim without a workload mount.
- The real state store saves an identity with its credential and endpoint.
  A new store instance then loads the same values, as after a restart.

The target also compiles copies of the unit tests in its included
production modules. The package unit tests already run those tests.
The `docker_regressions` filter selects only the tests of the four areas:

```sh
cargo test --package ciproxy-agent --test docker_regression docker_regressions
```

## Release packaging

Build and package the GNU binary for glibc 2.17 or later:

```sh
python3 -m pip install cargo-zigbuild
cargo zigbuild --locked --release --package ciproxy-agent --target x86_64-unknown-linux-gnu.2.17
crates/ciproxy-agent/scripts/cli-release.sh verify-glibc target/x86_64-unknown-linux-gnu/release/ciproxy-agent 2.17
crates/ciproxy-agent/scripts/cli-release.sh package ciproxy-agent 0.1.0 x86_64-unknown-linux-gnu target/x86_64-unknown-linux-gnu/release/ciproxy-agent .context/agent-release
```

Use the version from the agent manifest.
Packaging produces the `.tar.gz` archive and its `.sha256` companion.

Build, sign, and package the macOS binary on an Apple Silicon Mac:

```sh
export MACOSX_DEPLOYMENT_TARGET=14.0
cargo build --locked --release --package ciproxy-agent --target aarch64-apple-darwin
crates/ciproxy-agent/scripts/macos-release.sh sign target/aarch64-apple-darwin/release/ciproxy-agent
crates/ciproxy-agent/scripts/macos-release.sh verify target/aarch64-apple-darwin/release/ciproxy-agent 14.0 false 0.1.0
crates/ciproxy-agent/scripts/cli-release.sh package ciproxy-agent 0.1.0 aarch64-apple-darwin target/aarch64-apple-darwin/release/ciproxy-agent .context/agent-release
```

The agent uses Virtualization.framework on macOS. So each macOS binary needs
the entitlements in [ciproxy-agent.entitlements](macos/ciproxy-agent.entitlements),
also for local tests. Without the `MACOS_*` signing inputs, `sign` gives an
ad-hoc signature with these entitlements. Such a binary is not for release.
[Agent updates and releases](updates-and-releases.md#macos-signing) gives
the release signing.

## KVM and GitHub smoke checks

The [smoke guide](smoke-checks.md) gives the Docker, warm, and restart checks.
The [host guide](host-preparation.md#ext4-and-kvm-smoke) gives the ext4 check.
