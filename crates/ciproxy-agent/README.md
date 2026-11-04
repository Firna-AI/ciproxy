# ciproxy-agent

The command-line skeleton of the ciproxy agent on customer machines.
This binary depends only on `ciproxy-protocol` within the workspace.
The two crates form the boundary of the future public copy.
This crate has the Apache-2.0 licence in [LICENSE](LICENSE).

## Responsibilities

Own the agent executable and its command line.
Keep hosted control plane code out of its dependency tree.

## What This Crate Does

Parses `--help` and `--version` with Clap.
With no arguments, logs the shared protocol version with tracing and exits.
It runs no jobs and starts no service.

## Quick Start

```sh
cargo run --package ciproxy-agent -- --help
cargo run --package ciproxy-agent -- --version
```

## Development

```sh
cargo test --package ciproxy-agent
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Key Code

- [src/main.rs](src/main.rs) is the composition root.
- [src/cli.rs](src/cli.rs) defines the command line and startup log.
- [src/_tests_/cli_tests.rs](src/_tests_/cli_tests.rs) checks version output.

### Related Docs

- [Shared protocol](../ciproxy-protocol/README.md) owns shared wire types.
