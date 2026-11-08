# Agent integration checks and packaging

Run these commands from the repository root.
The [agent guide](README.md#development) gives the unit and compile checks.

## Docker integration checks

Docker lease integration tests call the real adapter in temporary folders.
They mock cache administration and most host commands.
The corruption case runs real `e2fsck` on temporary files.
These tests need `e2fsprogs`, without root access:

```sh
cargo test --package ciproxy-agent --test docker_regression docker_regressions
```

## Release packaging

Build and package the GNU binary for glibc 2.17 or later:

```sh
python3 -m pip install cargo-zigbuild
cargo zigbuild --locked --release --package ciproxy-agent --target x86_64-unknown-linux-gnu.2.17
scripts/cli-release.sh verify-glibc target/x86_64-unknown-linux-gnu/release/ciproxy-agent 2.17
scripts/cli-release.sh package ciproxy-agent 0.1.0 x86_64-unknown-linux-gnu target/x86_64-unknown-linux-gnu/release/ciproxy-agent .context/agent-release
```

Use the version from the agent manifest.
Packaging produces the `.tar.gz` archive and its `.sha256` companion.

## KVM and GitHub smoke checks

The [smoke guide](smoke-checks.md) gives the Docker, warm, and restart checks.
The [host guide](host-preparation.md#ext4-and-kvm-smoke) gives the ext4 check.
