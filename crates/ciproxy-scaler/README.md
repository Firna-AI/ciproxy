# ciproxy-scaler

Pure runner placement decisions for one ciproxy workspace. The hosted
control plane uses this library to turn current facts into typed actions.
Depend on it when a caller needs routing decisions without external IO.

## Responsibilities

Choose runner destinations and retire idle capacity. Reuse
`ciproxy-protocol` IDs, VM sizes, and the complete resource fit rule.
Keep GitHub, provider integrations, storage, clocks, and action delivery
outside this crate.
Use the open agent library for the one root allocation rule.
The open protocol crate does not depend on this hosted library.

## What This Crate Does

`decision::decide` takes a `snapshot::Snapshot` and returns `actions::Action`
values. Each start names an organization, label, requested size, actual VM
size, destination, and typed reason. GitHub chooses the job that a runner
takes. A start has no job or repository assignment.

The scaler counts Ready runners before accepted Starting runners across
all places. It considers unsent deliveries in the label order.
It matches supply to demand by the exact registration target. That target
contains the organization, label, binding row, and remote scale-set ID.
An equal label name does not establish compatibility.
For remaining demand the scaler walks each label's order. It tries the
configured places in order, with own machines and providers as entries.
At own machines it tries free capacity. A waiting job moves past that place
when no machine fits or when the overflow wait expires. At a provider, it
selects an eligible unsent delivery or tries the cheapest fitting size of
that account. A current provider check permits a resume or a start. A
failed check passes demand to the next place. A disconnected or unavailable
provider is skipped. A completed check applies the account's start holds.
The workspace switch can skip all providers. A macOS label can use own
machines only.

When no listed place fits and the label allows the largest-size fallback,
the scaler uses the first provider that can start its largest size. The
size must hold that provider's complete image. A fitting machine or a
fitting catalog of a connected and available provider blocks that fallback
even during a wait or a start hold.

Each new provider start uses that provider's current image requirement.
Retained idle runners use the required root saved for their image release.
Overflow dispatch uses the public size selector for the selected account.
The selector takes eligible sizes, fit of other places, and the label's
fallback choice.
An idle undersized fallback retires when its exception no longer
applies. `SizeNoLongerAllowed` identifies that retirement. Account spend
and start refusals do not retire an existing runner whose size still fits.
A retained largest-size runner keeps its saved image geometry. It remains
eligible while its size is the largest allowed size on that account and no
place of the order fits. A provider counts as a fit only when it has a
connected account, a configured start image, and a size that fits with
that image.

Label disk means free bytes for the job. Each start reserves the image,
those free bytes, and the ext4 growth allowance as its actual total root.
Every own-machine start reserves slots and that root during the decision call.
The agent rechecks sizing and capacity under its actual newest image hold.
This covers publication between a machine report and a runner start.
Correctly sized idle roots keep their original image allocation after an update.
Waiting jobs can request warm runner retirement and a projected reservation.
That reservation cannot release actual capacity or postpone the overflow
threshold. A job that reaches a provider before own machines waits while
its check is in progress. The scaler keeps an eligible unsent delivery but
defers its resume. The waiting job blocks warm starts on machines that can
run it. It can start or request warm retirement on an own machine once the
provider check completes and the order selects own machines.

Own machines keep warm runners only when they are first in the label order.
One warm count caps each machine's shared allocation across organizations.
Existing warm runners follow current targets on connected machines before
new replenishment checks capacity. A larger image cannot suppress count or
organization changes. Claimed runners keep queue priority.
All compatible idle `ciproxy` runners count, including those started for
queue demand. Waiting jobs have priority over warm starts. The scaler
retires idle runners after label edits or organization removal. Running
jobs retain their VM. Cleanup deletes only marked sandboxes with no job.

A launched provider runner that has taken no job by its
[ready deadline](../../docs/protocol/overflow-sandbox-end.md#the-ready-deadline)
retires with `ReadyDeadlinePassed`. The deadline counts from
`ProviderRunner.launched_at`. A runner without a launch time has no deadline.
The retiring runner is no supply, so no waiting job claims it. The same
decision can start a replacement by the usual placement.

### Snapshot contract

- Supply a complete, consistent view of one workspace.
- Supply `Snapshot.root_sizing` as the fallback before an own machine
  reports its selected image. Supply `ProviderAccount.start_geometry`
  only when the provider has a configured start image.
- Supply `Machine.selected_image` from `report.images.selected().cloned()`.
  Its exact release metadata takes precedence over that fallback.
- List each machine, runner, and sandbox once. Identify each sandbox by
  its provider and its sandbox ID.
- Supply each `Sandbox.runner` as `ProviderRunner { runner,
  required_root_disk_bytes, launched_at }`. Take the required root directly
  from that delivery's `StartRecord.required_root_disk_bytes` in
  `OverflowView.starts`. Take `launched_at` from `StartRecord.launched_at`
  of the same record. Use `None` when no launch is saved. Orphans have no
  runner.
- Preserve the saved order of each label and use the same opaque
  `ProviderId` in the order, the account, and its sandboxes.
- Supply each waiting GitHub job once. Exclude jobs already running.
- Supply each label's active saved `targets` from the same snapshot as
  demand. Each `WaitingJob.target` names its exact saved binding.
- Supply each `Runner.target` from its captured registration response.
  Keep an older unrecorded target as `None`. Do not infer it from the
  current binding. Unknown or replaced idle supply retires. Running jobs
  retain their VM.
- Include every accepted start before the next decision call.
- Retain starting, retiring, and exited VMs until capacity is released.
- Supply only organizations that can receive new runners.
- Use one clock origin for `now`, queue timestamps, retry timestamps, and
  launch times.
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
- Supply active Docker reservations separately as `docker_leased_bytes`.
  Use `Some(bytes)` for measured reservations, including measured zero.
  Use `None` while their statistics are unavailable. The scaler then keeps
  physical label eligibility and the overflow wait, but starts no queue or
  warm runner on that machine. Existing supply and warm targets still apply.
  Keep these reservations in the gross disk budget. The scaler subtracts
  measured reservations once for fit and available capacity.
- Supply each provider's check state, account connection, catalog dimensions
  and prices, and largest available size. Exclude refused sizes from the
  largest choice. The scaler compares prices within one catalog.
- Supply each provider's current UTC calendar month spend for its ciproxy
  sandboxes only.
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
For `ResumeProviderStart`, reuse the saved delivery. For
`ReplaceUnsentStart`, withdraw that delivery under the workspace session
lock before starting the replacement. If dispatch has begun, keep the
delivery and skip the replacement.

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

The overflow integration supplies each provider's catalog, prices, largest
size, retry times, billing recovery, key recovery, monthly spend, and launch
times. It also applies sandbox marks, exit reports, lifetime limits,
deletion, and key retention. The control plane maps stored provider names
to opaque `ProviderId` values at the scaler boundary.

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
- [src/lifecycle.rs](src/lifecycle.rs) lets waiting jobs claim existing
  runners and retires idle ones, including at the provider ready deadline.
- [src/snapshot.rs](src/snapshot.rs) defines the caller's workspace facts.
- [src/actions.rs](src/actions.rs) defines start and retirement intents.
- [src/overflow.rs](src/overflow.rs) walks the label order for new starts.
- [src/registration.rs](src/registration.rs) defines the immutable target
  carried through demand, supply, and start actions.
- [src/provider.rs](src/provider.rs) defines provider IDs, account catalogs,
  refusals, and sandbox facts. `ProviderRunner` holds the saved root
  requirement and launch time of each retained runner.
- [src/capacity.rs](src/capacity.rs) checks complete allocated roots with the
  protocol resource API.
- [Agent root sizing](../ciproxy-agent/src/root_sizing.rs) defines the shared
  free-to-total disk calculation.
- [src/size_policy.rs](src/size_policy.rs) exposes `select_provider_size`
  for overflow dispatch and shares its fit rules with scaler placement.
- [examples/own_machine.rs](examples/own_machine.rs) runs the public API.

### Related Docs

- [Protocol crate](../ciproxy-protocol/README.md) owns wire values and slots.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Job routing](../../docs/protocol/job-routing.md) defines routing rules.
- [Labels](../../docs/protocol/labels.md) defines labels and size changes.
- [Label order](../../docs/protocol/labels-order.md) defines the places.
- [Overflow](../../docs/protocol/overflow.md) defines the provider rules.
- [Sandbox end](../../docs/protocol/overflow-sandbox-end.md) defines idle
  runner retirement and the ready deadline.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization scope.
