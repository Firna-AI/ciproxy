# ciproxy-scaler

Pure runner placement decisions for one ciproxy workspace. The hosted
control plane uses this library to turn current facts into typed actions.
Depend on it when a caller needs routing decisions without external IO.

## Responsibilities

Choose runner destinations and retire idle capacity. Reuse
`ciproxy-protocol` IDs, VM sizes, and the complete resource fit rule.
Keep GitHub, Boat, storage, clocks, and action delivery outside this crate.
Use the open agent library for the one root allocation rule.
The open protocol crate does not depend on this hosted library.

## What This Crate Does

`decision::decide` takes a `snapshot::Snapshot` and returns `actions::Action`
values. Each start names an organization, label, requested size, actual VM
size, destination, and typed reason. GitHub chooses the job that a runner
takes. A start has no job or repository assignment.

The scaler counts Ready runners before Starting runners across both
destinations. It counts persisted starts before it adds capacity.
It tries free own capacity first. It starts eligible overflow
after the configured wait. Without an eligible own machine, it can start
Boat at once. It selects the cheapest fitting catalog size or uses the
label's explicit largest-size choice. Even that fallback must hold the image.

New Boat starts use the current image requirement. Retained idle runners
use the required root saved for their own image release.
Overflow dispatch uses the public pure selector for new starts.
Callers supply eligible sizes after current provider refusals are applied.
The selector takes own-machine fit and the label's current fallback choice.
An idle undersized fallback retires when its exception no longer
applies. `SizeNoLongerAllowed` identifies that retirement. Account spend
and start refusals do not retire an existing runner whose size still fits.
A retained largest-size runner keeps its held image. Its exception checks
current own-machine fit, current new-start catalog fit, and the largest
allowed size. The new image's floor does not apply to that held VM.

Label disk means free bytes for the job. Each start reserves the image,
those free bytes, and the ext4 growth allowance as its actual total root.
Every own-machine start reserves slots and that root during the decision call.
The agent rechecks sizing and capacity under its actual newest image hold.
This covers publication between a machine report and a runner start.
Correctly sized idle roots keep their original image allocation after an update.
Waiting jobs can request warm runner retirement. Each waiting job gets a
separate projected reservation. That reservation cannot release actual
capacity or postpone the overflow threshold.

One warm count caps each machine's shared allocation across organizations.
Existing warm runners follow current targets on connected machines before
new replenishment checks capacity. A larger image cannot suppress count or
organization changes. Claimed runners keep queue priority.
All compatible idle `ciproxy` runners count, including those started for
queue demand. Waiting jobs have priority over warm starts. The scaler
retires idle runners after label edits or organization removal. Running
jobs retain their VM. Cleanup deletes only marked sandboxes with no job.

### Snapshot contract

- Supply a complete, consistent view of one workspace.
- Supply `root_sizing` for the current Boat image and the own-machine
  fallback before a machine reports its selected image.
- Supply `Machine.selected_image` from `report.images.selected().cloned()`.
  Its exact release metadata takes precedence over that fallback.
- List each machine, runner, and sandbox once.
- Supply each `Sandbox.runner` as `BoatRunner { runner,
  required_root_disk_bytes }`. Take the required root directly from that
  delivery's `StartRecord.required_root_disk_bytes` in `OverflowView.starts`.
  Keep it required for retained Boat runners. Do not infer it from current
  image settings or actual catalog disk. Orphans have no runner.
- Supply each waiting GitHub job once. Exclude jobs already running.
- Include every accepted start before the next decision call.
- Retain starting, retiring, and exited VMs until capacity is released.
- Supply only organizations that can receive new runners.
- Use one clock origin for `now`, queue timestamps, and retry timestamps.
- Preserve `RunnerReport.requested_size` as `Runner.requested_size`.
  Preserve `RunnerReport.size` as the actual allocation in `Runner.size`.
  Do not derive either from current labels or the newest image.
- Supply each machine's `warm_allocation` under the documented
  [warm allocation policy](../../docs/protocol/job-routing.md#warm-runners).
  Each entry requests one runner. `warm_count` caps the whole list.
- Supply the machine's total job disk as `disk_budget_bytes`.
  Use `Resources::for_machine_with_image_budget` with its persisted image
  budget, then take `job_disk_bytes`. Subtract images before the 1:3 split.
  Include disk held by the listed VMs and space used by machine or Docker
  caches. These caches free space for new VMs.
- Supply current Boat dimensions, comparable catalog prices, and the
  largest available size. Exclude refused sizes from the largest choice.
- Supply current UTC calendar month spend for ciproxy sandboxes only.
- Keep key and billing refusals until external checks confirm recovery.

The library adds no snapshot validation or external policy defaults.
Equal-price choices follow catalog input order. Machine and queue input
order resolve equivalent choices. The protocol prescribes no priority
between equivalent choices.

### Apply the actions

Allocate runner identities and registrations outside this crate.
For own machines, send `StartRunner.requested_size` as `RunnerSpec.size`.
The returned actual report replaces the predicted complete allocation. Persist
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

Convert label free bytes with `ciproxy-agent::root_sizing::RootSizing`.
Total and free capacity checks use `ciproxy-protocol::resources::Resources`
and its `fits` method. Starts need free slots and available job disk. Each VM
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
- [src/warm.rs](src/warm.rs) separates existing target reconciliation from
  capacity checks and new warm replenishment.
- [src/snapshot.rs](src/snapshot.rs) defines the caller's workspace facts.
- [src/actions.rs](src/actions.rs) defines start and retirement intents.
- [src/boat.rs](src/boat.rs) defines catalog, refusal, and sandbox facts.
  `BoatRunner` requires the saved root for each retained Boat runner.
- [src/capacity.rs](src/capacity.rs) checks complete allocated roots with the
  protocol resource API.
- [Agent root sizing](../ciproxy-agent/src/root_sizing.rs) defines the shared
  free-to-total disk calculation.
- [src/size_policy.rs](src/size_policy.rs) exposes `select_boat_size` for
  scaler placement and overflow dispatch.
- [examples/own_machine.rs](examples/own_machine.rs) runs the public API.

### Related Docs

- [Protocol crate](../ciproxy-protocol/README.md) owns wire values and slots.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Job routing](../../docs/protocol/job-routing.md) defines routing rules.
- [Labels](../../docs/protocol/labels.md) defines labels and size changes.
- [Overflow](../../docs/protocol/overflow.md) defines the Boat rules.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization scope.
