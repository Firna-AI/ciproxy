# ciproxy-scaler

Pure runner placement decisions for one ciproxy workspace. The hosted
control plane uses this library to turn current facts into typed actions.
Depend on it when a caller needs routing decisions without external IO.

## Responsibilities

Choose runner destinations and retire idle capacity. Reuse
`ciproxy-protocol` IDs, VM sizes, and the complete resource fit rule.
Keep GitHub, Boat, storage, clocks, and action delivery outside this crate.
The open protocol crate does not depend on this hosted library.

## What This Crate Does

`decision::decide` takes a `snapshot::Snapshot` and returns `actions::Action`
values. Each start names an organization, label, requested size, actual VM
size, destination, and typed reason. GitHub chooses the job that a runner
takes. A start has no job or repository assignment.

The scaler counts ready runners and persisted starts before it adds
capacity. It tries free own capacity first. It starts eligible overflow
after the configured wait. Without an eligible own machine, it can start
Boat at once. It selects the cheapest fitting catalog size or uses the
label's explicit largest-size choice.

Every own-machine start reserves slots and job disk during the decision call.
Waiting jobs can request warm runner retirement. Each waiting job gets a
separate projected reservation. That reservation cannot release actual
capacity or postpone the overflow threshold.

One warm count caps each machine's shared allocation across organizations.
All compatible idle `ciproxy` runners count, including those started for
queue demand. Waiting jobs have priority over warm starts. The scaler
retires idle runners after label edits or organization removal. Running
jobs retain their VM. Cleanup deletes only marked sandboxes with no job.

### Snapshot contract

- Supply a complete, consistent view of one workspace.
- List each machine, runner, and sandbox once.
- Supply each waiting GitHub job once. Exclude jobs already running.
- Include every accepted start before the next decision call.
- Retain starting, retiring, and exited VMs until capacity is released.
- Supply only organizations that can receive new runners.
- Use one clock origin for `now`, queue timestamps, and retry timestamps.
- Preserve both the runner's original requested size and its actual size.
- Supply each machine's `warm_allocation` under the documented
  [warm allocation policy](../../docs/protocol/job-routing.md#warm-runners).
  Each entry requests one runner. `warm_count` caps the whole list.
- Supply the machine's total job disk as `disk_budget_bytes`. Include disk
  held by the listed VMs and space used by the machine cache or Docker
  disks. The machine cache and unused Docker disks free space for new VMs.
- Supply current Boat dimensions, comparable catalog prices, and the
  largest available size. Exclude refused sizes from the largest choice.
- Supply current UTC calendar month spend for ciproxy sandboxes only.
- Keep key and billing refusals until external checks confirm recovery.

The library adds no snapshot validation or external policy defaults.
Equal-price choices follow catalog input order. Machine and queue input
order resolve equivalent choices. The protocol prescribes no priority
between equivalent choices.

### Apply the actions

Allocate runner identities and registrations outside this crate. Persist
each accepted start as `Starting` before another tick. Concurrent callers
must serialize decisions and reservation updates for the workspace.

Treat a stop or deletion as an intent. Remove the GitHub registration and
confirm the runner is still idle before releasing its VM. If GitHub gave
it a job, keep that job and rebuild the snapshot. Keep retiring capacity
in subsequent snapshots until the agent or backend confirms its removal.
Coalesce pending stop and deletion requests during action delivery.

### Integration boundary

The GitHub integration supplies durable demand and organization facts.
The control plane builds the consistent snapshot and applies machine
actions. Its snapshot builder applies the documented
[warm allocation policy](../../docs/protocol/job-routing.md#warm-runners)
and supplies the per-machine targets. The scaler enforces those targets.
Its tests check allocation enforcement. The snapshot builder owns the
history calculation.

The overflow integration supplies the Boat catalog, prices, largest size,
retry times, billing recovery, key recovery, and monthly spend. It also
applies sandbox marks, exit reports, lifetime limits, deletion, and key
retention.

Total and free capacity checks use `ciproxy-protocol::resources::Resources`
and its `fits` method. Starts need free slots and free job disk. Each VM
holds its original disk until removal, including warm and retiring VMs.
The scaler keeps actual and projected reservations separate. Cache
reclamation and host disk measurement remain external.

## Quick Start

Run these commands from the repository root:

```sh
cargo build --package ciproxy-scaler
cargo test --package ciproxy-scaler
cargo run --package ciproxy-scaler --example own_machine
```

The example supplies a hand-worked snapshot. It asserts and prints one
typed own-machine start. It needs no host, credential, secret, or network.

## Development

Run `cargo fmt --all -- --check` and
`cargo clippy --package ciproxy-scaler --all-targets -- -D warnings`.
Tests use the public decision entry point with literal states and actions.

### Key Code

- [src/decision.rs](src/decision.rs) owns the public `decide` entry point.
- [src/snapshot.rs](src/snapshot.rs) defines the caller's workspace facts.
- [src/actions.rs](src/actions.rs) defines start and retirement intents.
- [src/boat.rs](src/boat.rs) defines catalog, refusal, and sandbox facts.
- [src/capacity.rs](src/capacity.rs) delegates complete fit to the protocol.
- [examples/own_machine.rs](examples/own_machine.rs) runs the public API.

### Related Docs

- [Protocol crate](../ciproxy-protocol/README.md) owns wire values and slots.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Job routing](../../docs/protocol/job-routing.md) defines routing rules.
- [Labels](../../docs/protocol/labels.md) defines labels and size changes.
- [Overflow](../../docs/protocol/overflow.md) defines the Boat rules.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization scope.
