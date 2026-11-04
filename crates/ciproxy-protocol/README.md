# ciproxy-protocol

Shared wire types for the ciproxy agent and control plane.
Depend on this library when both binaries need the same wire value.
This crate has the Apache-2.0 licence in [LICENSE](LICENSE).

## Responsibilities

Own values shared by the agent, the control plane, and the guest program.
Keep the library independent of the hosted control plane and other
workspace crates. This library performs no network, disk, or runtime work.

## What This Crate Does

Compute VM slot requirements and machine capacity after the host reserve.
Carry registration, connection, runner, drain, cache, and image messages
as JSON with a `type` tag and snake-case variant names.

The connection hello carries the same full machine state as later reports.
Reports include hardware, health, slots, runners, images, cache counts,
and Docker disk use. The report interval is ten seconds.
The guest uses separate message types for both directions over vsock.

A ciproxy workspace owns each machine, its install token, and its machine
credential. Registration and the authenticated hello name that workspace.
A workspace can hold several GitHub organizations.
Its machines can run jobs for each of those organizations.

Runner starts, current runner reports, idle-runner stops, and guest
configuration carry the runner's organization. A warm runner knows its
organization before GitHub assigns its job.
Job facts then carry that organization and the actual assigned repository.
GitHub selects the job, so a runner start carries no repository assignment.

Cache reports keep measured counts for every restore that the machine
cache handled, including contents fetched entirely from GitHub.
The counts separate local entries from GitHub entries and include total
restored bytes. A pure method derives the content source from these
counts. Zero entries give no observed source.
Direct GitHub use and fallback carry no counts.

`Secret` hides credentials, tokens, registrations, and URLs from `Debug`.
Its `expose_secret` method gives explicit access to the original value.
JSON serialization also carries the original value for delivery.
Do not log serialized messages that contain secrets.

Job facts carry cache-scope inputs without selecting their source or
granting shared write access. Memory and disk fields use bytes.
The documented resource sizes use decimal GB for slot arithmetic.
Disk capacity does not enter the CPU and memory slot formula.
`ProtocolVersion::CURRENT` keeps its JSON integer representation.

## Quick Start

From the repository root:

```sh
cargo build --package ciproxy-protocol
cargo test --package ciproxy-protocol
```

For example, calculate the capacity and a VM requirement:

```rust
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};

let capacity = Slots::for_machine(4, 16 * GIGABYTE, true);
let required = Slots::for_vm(VmSize {
    vcpu: 2,
    memory_bytes: 8 * GIGABYTE,
    disk_bytes: 60 * GIGABYTE,
});
assert!(capacity.fits(required));
```

## Development

Run `cargo fmt --all` after an edit.
Run `cargo clippy --package ciproxy-protocol --all-targets -- -D warnings`
before handoff. Each message variant has a file-based JSON snapshot.
Review snapshot changes before accepting them.

### Key Code

- [src/resources.rs](src/resources.rs) calculates slot counts.
- [src/identifiers.rs](src/identifiers.rs) distinguishes workspace,
  organization, machine, runner, and repository identities.
- [src/agent.rs](src/agent.rs) defines both agent connection directions.
- [src/guest.rs](src/guest.rs) defines both guest vsock directions.
- [src/reports.rs](src/reports.rs) defines current machine state.
- [src/cache.rs](src/cache.rs) defines restore handling and
  `CacheRestoreCounts::source`, which derives the measured content source.
- [src/jobs.rs](src/jobs.rs) carries job identity and cache-scope inputs.
- [src/secret.rs](src/secret.rs) protects secret diagnostic output.
- [src/version.rs](src/version.rs) preserves the shared version API.
- [src/_tests_/wire/](src/_tests_/wire/) holds wire tests and snapshots.

### Related Docs

- [Agent](../ciproxy-agent/README.md) consumes the shared version.
- [Control plane](../ciproxy-control-plane/README.md) consumes the same type.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Agent protocol](../../docs/protocol/agent.md) defines connection and health.
- [Job routing](../../docs/protocol/job-routing.md) defines slot arithmetic.
- [Cache](../../docs/protocol/cache.md) defines cache scopes and reports.
- [Security](../../docs/protocol/security.md) defines key custody.
