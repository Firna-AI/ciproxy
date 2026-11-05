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
Check complete fit, concurrent VM counts, and free slots and job disk.
Carry registration, connection, runner, drain, cache, and image messages
as JSON with a `type` tag and snake-case variant names.

The connection hello carries the same full machine state as later reports.
Reports include hardware, health, slots, runners, images, cache counts,
and Docker disk use. The report interval is ten seconds.
The guest uses separate message types for both directions over vsock.

A ciproxy workspace owns each machine, its install token, and its machine
credential. Register carries the install token without a workspace ID.
The token's authority determines the workspace. Registered returns that
workspace ID with the machine identity and credential.
The authenticated Hello names the workspace returned at registration.
A workspace can hold several GitHub organizations.
Its machines can run jobs for each of those organizations.

Runner starts, current runner reports, idle-runner stops, and guest
configuration carry the runner's organization. A warm runner knows its
organization before GitHub assigns its job.
Job facts then carry that organization and the actual assigned repository.
GitHub selects the job, so a runner start carries no repository assignment.
`RunnerSpec.size` is the original free-size request.
`RunnerReport.requested_size` retains that request across reconnects.
`RunnerReport.size` is the actual total allocation of the held image.
Image downloads carry the manifest's required `root_disk_bytes`.
`ImageReport.available` contains `ImageMetadata` for every available release.
`ImageReport::selected` resolves the newest release's exact facts.
These required fields replace the old image list and runner-report format.

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

Job facts carry expected repository, run, job, and JIT runner identities,
with the API event, head repository, and default branch. They carry no
guessed full job ref. `JobScope` separately carries a machine-checked ref
and readable scopes, or a typed unknown reason. The admin session result
returns that scope for Docker-disk policy. Machine cache reports count
unknown jobs by reason. These values grant no trust by their presence.
Memory and disk fields use bytes.
The documented resource sizes use decimal GB for slot arithmetic.
Disk capacity does not enter the CPU and memory slot formula.
Use `Resources` for a complete fit check that includes job disk.
Label sizes contain free job bytes. Actual runner sizes contain total roots.
Use the agent library's `RootSizing` to convert label sizes before resource
checks. The protocol crate has no dependency on that implementation.
Pass the capacity of the disk used for images, VMs, and machine cache.
First subtract the 140 GB image budget, then give jobs three quarters.
The host reserve keeps one quarter of the remaining disk.
`Resources::for_machine_with_image_budget` accepts a persisted override.
Use that same override for area sizing, admission, and cache budgets.
Filesystem free bytes stay a separate health measurement.

Pass each held VM's recorded size to `Resources::available` once.
Include starting, warm, running, and stopping VMs until their disk is free.
Use the recorded VM size even after its label changes.
Machine cache and unused Docker cache remain reclaimable job disk.
`ProtocolVersion::CURRENT` keeps its JSON integer representation.

## Quick Start

From the repository root:

```sh
cargo build --package ciproxy-protocol
cargo test --package ciproxy-protocol
```

For example, calculate capacity and the space left by two warm runners:

```rust
use ciproxy_protocol::resources::{GIGABYTE, Resources, Slots, VmSize};

let capacity = Resources::for_machine(16, 64 * GIGABYTE, 512 * GIGABYTE, true);
let size = VmSize {
    vcpu: 2,
    memory_bytes: 8 * GIGABYTE,
    disk_bytes: 81_635_856_384,
};
assert_eq!(capacity.slots, Slots(6));
assert_eq!(capacity.job_disk_bytes, 279 * GIGABYTE);
assert!(capacity.fits(size));
assert_eq!(capacity.concurrent_vms(size), 3);

let free = capacity.available([size, size]);
assert_eq!(free.job_disk_bytes, 115_728_287_232);
assert!(free.fits(size));

let after_third_vm = free.available([size]);
assert_eq!(after_third_vm.job_disk_bytes, 34_092_430_848);
assert!(!after_third_vm.fits(size));
```

## Development

Run `cargo fmt --all` after an edit.
Run `cargo clippy --package ciproxy-protocol --all-targets -- -D warnings`
before handoff. Each message variant has a file-based JSON snapshot.
Review snapshot changes before accepting them.

### Key Code

- [src/images.rs](src/images.rs) carries verified per-release disk metadata.
- [src/resources.rs](src/resources.rs) calculates slots and job disk.
  `Resources` checks fit, concurrency, and availability for complete sizes.
- [src/identifiers.rs](src/identifiers.rs) distinguishes workspace,
  organization, machine, runner, and repository identities.
- [src/agent.rs](src/agent.rs) defines both agent connection directions.
- [src/guest.rs](src/guest.rs) defines both guest vsock directions.
- [src/reports.rs](src/reports.rs) defines current machine state.
- [src/cache.rs](src/cache.rs) defines restore handling and
  `CacheRestoreCounts::source`, which derives the measured content source.
- [src/jobs.rs](src/jobs.rs) carries expected assignment identities and
  checked or unknown machine scopes.
- [src/secret.rs](src/secret.rs) protects secret diagnostic output.
- [src/version.rs](src/version.rs) preserves the shared version API.
- [src/_tests_/wire/](src/_tests_/wire/) holds wire tests and snapshots.

### Related Docs

- [Agent](../ciproxy-agent/README.md) consumes the shared version.
- [Control plane](../ciproxy-control-plane/README.md) consumes the same type.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Agent protocol](../../docs/protocol/agent.md) defines connection and health.
- [Job routing](../../docs/protocol/job-routing.md) defines slots and job disk.
- [Cache](../../docs/protocol/cache.md) defines cache scopes and reports.
- [Security](../../docs/protocol/security.md) defines key custody.
