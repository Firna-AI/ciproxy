# ciproxy-control-plane

The command-line skeleton of the hosted ciproxy control plane.
This binary uses the same protocol version as the agent.
This crate has no open licence.

## Responsibilities

Own the hosted control plane executable and its command line.
Keep hosted behaviour separate from the public agent crates.

## What This Crate Does

Parses `--help` and `--version` with Clap.
With no arguments, logs the shared protocol version with tracing and exits.
It exposes no HTTP service and has no product behaviour.

## Quick Start

```sh
cargo run --package ciproxy-control-plane -- --help
cargo run --package ciproxy-control-plane -- --version
```

## Development

```sh
cargo test --package ciproxy-control-plane
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

### Key Code

- [src/main.rs](src/main.rs) is the composition root.
- [src/cli.rs](src/cli.rs) defines the command line and startup log.
- [src/_tests_/cli_tests.rs](src/_tests_/cli_tests.rs) checks version output.

### Related Docs

- [Shared protocol](../ciproxy-protocol/README.md) owns shared wire types.
- [Agent](../ciproxy-agent/README.md) is the other protocol consumer.
