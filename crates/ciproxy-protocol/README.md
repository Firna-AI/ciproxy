# ciproxy-protocol

Shared wire types for the ciproxy agent and control plane.
Depend on this library when both binaries need the same wire value.
This crate has the Apache-2.0 licence in [LICENSE](LICENSE).

## Responsibilities

Own types shared across the agent boundary.
Keep the library independent of the hosted control plane.

## What This Crate Does

Defines `ProtocolVersion::CURRENT` and its JSON integer representation.
The skeleton defines no product messages or runtime behaviour.

## Quick Start

From the repository root:

```sh
cargo build --package ciproxy-protocol
cargo test --package ciproxy-protocol
```

## Development

Run `cargo fmt --all` after an edit.
Run `cargo clippy --workspace --all-targets -- -D warnings` before handoff.

### Key Code

- [src/lib.rs](src/lib.rs) exposes the version module.
- [src/version.rs](src/version.rs) defines the shared version type.
- [src/_tests_/version_tests.rs](src/_tests_/version_tests.rs) checks its wire form.

### Related Docs

- [Agent](../ciproxy-agent/README.md) consumes the shared version.
- [Control plane](../ciproxy-control-plane/README.md) consumes the same type.
