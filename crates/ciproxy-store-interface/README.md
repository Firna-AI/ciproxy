# ciproxy-store-interface

Typed persistence traits for the hosted ciproxy control plane.
Services depend on these traits through dyn.
The PostgreSQL adapter implements them in ciproxy-store-pg.
This crate has no open licence.

## Responsibilities

Own the store API, data types, and typed error contract.
Keep database rows and Diesel out of service code.
Keep persistence separate from scheduling and external services.

## What This Crate Does

Defines object-safe traits for workspace data and each store area.
A workspace owns shared machines, labels, credentials, settings, and Boat.
Each GitHub organization keeps its installation and repository access.
Scale sets serve one organization and one shared workspace label.
Runners and jobs keep their organization identity.
Resource queries require the workspace identifier.
Organization-specific queries also require the organization identifier.
Token authentication finds workspace authority from a precomputed hash.

TokenHash and EncryptedBoatKey accept bytes from trusted callers.
Their Debug output hides the bytes.
Account metadata excludes encrypted keys.
Token metadata excludes hashes.
KMS encryption and token hashing belong to callers.

The store preserves runner placements and actual VM sizes for job history.
It accepts optional ref metadata without choosing its provenance.
The later job-facts service must supply trusted GitHub metadata.
Job cache metadata separates content origin from machine-handled measurements.
One optional count bundle holds restored entries and bytes together.
Measured GitHub-only fetches and zero-entry restores keep that bundle.
Direct GitHub use and fallback have no measured count bundle.
Organization removal keeps shared resources while another App remains.
Removal of the last App revokes machine and install credentials.
Live sandbox-end tokens and retired Boat keys stay available for cleanup.
Sandbox-end tokens stop working when their runner ends.
Organization and empty-workspace records have 30-day deletion deadlines.
Reinstall clears the workspace deadline and preserves its retained data.
The purge query deletes one expired workspace and its owned records.
GitHub-ID lookup returns current and retained registrations.
The caller selects a workspace from the install intent.
A new registration in another workspace leaves old history in place.
Removal and retention queries change stored records only.
They do not stop machines, call GitHub, or delete Boat sandboxes.

The GitHub area stores visible repository metadata and default branches.
It atomically registers a new workspace with labels and a pending first
Admin assignment. Its demand records use the UUID from scale-set jobId.
They keep the existing numeric Actions history identity separate.
An atomic job update also saves a pending notification revision.
Replay preserves terminal state and exact nanosecond assignment order.
Confirmation of an older revision cannot consume a newer update.
The demand retention query removes completed records after 90 days,
including pending deliveries. When GitHub omitted its completion time,
the first caller-supplied completion observation starts that period.
Replay preserves that observation without fabricating a GitHub time.
Trusted run facts preserve raw API fields. A branch name is not a full ref.
GitHub lifecycle transitions save verified identity, App state, prepared
resources, and immutable notification snapshots together. Only completed
remote preparation permits activation. Failed delivery stays pending in
sequence order even after organization rows change or a workspace purges.
Consumer acceptance permits deletion. A crash before confirmation requires
idempotent delivery. A pending-organization scan supports restart recovery.
Each `Removed` event keeps its required `RemovalCause`, `Owner` or
`WorkspaceTakeout`, with no default.
Verified installation snapshots save login and App state together.
The scoped write returns the saved record. It cannot change another
workspace or account, replace a newer installation, or revive removal.
Known-run facts queries require workspace, organization, and run identity.
Remote scale-set upserts preserve the internal row and lock key.
Organization mutation leases wait on dedicated connections. They retain
all GitHub identity bits in the separate two-key advisory namespace.

MeteringStore reads coherent workspace facts across all organizations.
It stores trusted creation times, registration times, and cost histories.
It stores explicit Free and Stripe billing-period boundaries.
Anonymous daily aggregates keep actual VM sizes and Boat costs.
Anonymous timestamp counts preserve exact period boundaries.
Job and runner cursors support atomic signed aggregate corrections.
BoatAccounting holds source costs and cursors from one coherent snapshot.
SavingsSnapshot holds registrations, range totals, and monthly costs
from one coherent snapshot.
Its registrations include removed machines for historical cost dates.
The savings read excludes jobs, start counts, and accounting cursors.
Deleting a job or runner deletes its cursor and preserves its aggregates.
The metering service supplies all calculation rules and retention dates.
Boat accounting returns known price intervals and bounded measured prefixes.
It remains available when another overflow cost lacks a fee or final reading.
The metering refresh can then complete its retention pass.
Savings snapshots return `AccountingIncomplete` for gaps in their requested
date range. They never publish a missing cost as zero or as outage time.

BillingStore adds the confirmed tier, add-ons, paid bounds, and Free anchor.
It exposes retry and ended-banner dates without changing job execution.
BillingSession owns exclusive command and event reconciliation.
It retains each owned renewal outcome separately from the billing banner.
Each renewal retains whether the provider attempted its payment.
Verified automatic collection stops remain available after a service restart.
Each stop event retains its eligible, superseded, or completed decision.
The subscription-ID lookup resolves a billing workspace before provider reads.
State and event receipts commit atomically.
App removal stores a durable billing cancellation request.
Services consume these boundaries through dyn.

OverflowStore adds exclusive workspace sessions for Boat execution.
Pending connections use fresh store-issued grants before external work.
Last-App removal revokes these grants, and activation consumes one live grant.
Reusing a key-version identity cannot reuse an earlier grant.
Pending acceptance retains immutable demand, sealed input, and its saved fee.
It fixes no provider parameters and counts no provider cost bound.
Prepared dispatch carries the selected provider parameters and exact monthly
amounts together.
Its transaction locks saved settings and returns a distinct spend-limit refusal.
Its startup query derives the organization and key from the saved delivery.
It checks tenant ownership, active App state, enabled overflow, and the
accepted non-retired key together.
It also requires the saved scale set and the label's current Boat switch.
All current label dimensions must match the original requested dimensions.
Each delivery retains its image root bytes and required total root bytes.
Authority compares catalog disk with those saved totals.
Even a largest-size fallback must hold the complete image.
Reservation uses the same policy as saved-delivery startup.
The current-label query supplies resource policy facts without store policy.
The atomic dispatch operation rechecks authority and budget before `Creating`.
It preserves all pending fields and fixes size, lifetime, and dispatch time.
Later recovery uses those exact committed parameters.
Unsent deliveries can retry preparation without provider recovery.
Cancelled unsent deliveries retain no sealed input or callback authority.
Account observations keep typed catalogs, retry times, and refusal state.
Runner input stays encrypted until launch or cleanup ends its use.
Callback lookup derives the workspace from a token digest.
A transactional claim accepts one report for one live runner.
There is no independent expiry before that runner ends.
Old keys stay while uncertain creates or live sandboxes reference them.
Per-month runtime observations keep incomplete costs explicit.
Each delivery separates its saved start-fee bound from a confirmed fee.
Spend and savings select the confirmed fee, or else the saved fee bound.
The selected fee does not become a provider-confirmed charge.
Runtime bounds never supply measured runtime facts.
Running observations and total runtime costs survive a failed monthly read.
The overflow service allocates confirmed monthly costs or conservative bounds.
The store does not turn a cross-month estimate into a measured price interval.
Measured billing coverage stays separate from lifecycle end and its proven
provider lifetime bound.
Finite measured intervals clamp to their latest provider observation.
One known-sandbox value contains both identity and lifetime proof.
The overflow service supplies prices, calculations, KMS, and provider calls.

The gateway queries atomically register a machine and its hashed credential.
GatewayStore reuses TokenStore authority lookup to derive the workspace from
the install hash. Registration rechecks the token under its transaction
locks before it inserts the machine.
They retain the full typed protocol report with a connection identifier.
A `MachineSnapshot` returns it as a `ReceivedReport` with its receipt time.
Later lifecycle contact does not change that time.
Report writes preserve desired drain and warm settings.
They reject replaced sessions and removed registrations.
Gateway token issuance and registration serialize with workspace removal.

The dashboard area stores verified GitHub memberships, workspace roles,
access requests, opaque session digests, and encrypted OAuth grants.
A person record stays only with a role or an open request. Intentional
organization removal keeps a separate durable marker and no owner-removal
retention deadline. Retired labels preserve history while active names
remain unique.

Credential-only rotation writes preserve verified memberships and their check
time. Current machine cost inputs contain no effective timestamp. Their atomic
write reads the injected clock after acquiring its locks.

Scale-set bindings distinguish Active, Draining, and Removed. Listener discovery
includes active bindings and draining bindings with tracked work or pending
delivery. Exact context survives cleanup for final acknowledgements and runner
removal. Confirmed removal applies only to the exact saved binding and remote ID.

OwnRunnerStore exposes one coherent workspace routing snapshot.
Its dedicated session lease serializes own-machine decisions.
Runner intents retain exact slot sets and the original whole-GB label
request. The shared Runner record is the byte-precise actual allocation.
Metering facts, cursors, and totals use that same actual resource type.
No JIT or original runtime-token credential enters these records.
`OwnRunner.confirmed_at` keeps the time of the first execution evidence of a
runner, from a command result or a trusted agent or GitHub observation.
`observe_own_runner` takes a required `OwnRunnerObservation`: the state, the
GitHub ID, and the observation time. In one transaction, it saves the state
and the ID, and it sets `confirmed_at` only when it is empty.
It filters by workspace, organization, and runner only. Without the runner
or its command record, it returns `NotFound` and saves nothing.
`record_own_command_result` saves the result of the current command. It
also saves that first evidence, and only an applied result ends delivery.
`retire_own_runner` saves the trusted GitHub ID, `Stopping`, and the stop
command in one transaction.

## Quick Start

Use the same interface for PostgreSQL and service test doubles:

```rust
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

fn workspace_labels(
    store: &dyn Store,
    workspace_id: Uuid,
) -> Result<Vec<Label>> {
    store.labels(workspace_id)
}
```

Shared services hold `Arc<dyn Store>`.
Keep a ScaleSetSession handle alive for the complete GitHub session.
Call its check_connection method before continued use after a connection
error or during listener health checks.
Stop the listener when this query fails.

## Development

Run commands from the repository root:

```sh
cargo build --package ciproxy-store-interface
cargo test --package ciproxy-store-interface
cargo fmt --all -- --check
cargo clippy --package ciproxy-store-interface --all-targets -- -D warnings
```

### Key Code

- [src/dashboard](src/dashboard) owns dashboard identity and access queries.
- [src/own_runners.rs](src/own_runners.rs) owns exact execution records.

- [src/store.rs](src/store.rs) defines Store and its connection factory.
- [src/billing/store.rs](src/billing/store.rs) owns durable billing sessions.
- [src/billing/renewals.rs](src/billing/renewals.rs) defines durable renewal
  outcomes and verified automatic collection stops.
- [src/billing/collection.rs](src/billing/collection.rs) defines per-event
  cancellation decisions and their terminal states.
- [src/workspaces.rs](src/workspaces.rs) defines shared customer ownership.
- [src/organizations.rs](src/organizations.rs) defines GitHub App records.
- [src/error.rs](src/error.rs) owns typed failures and safe diagnostics.
- [src/secrets.rs](src/secrets.rs) owns opaque hashes and ciphertext.
- [src/scale_sets.rs](src/scale_sets.rs) defines session lock ownership.
- [src/jobs.rs](src/jobs.rs) defines metadata-only job records.
- [src/github.rs](src/github.rs) defines repository snapshots and durable
  UUID demand and pending role delivery.
- [src/github_lifecycle.rs](src/github_lifecycle.rs) defines atomic App
  transitions and immutable ordered lifecycle delivery.
- [src/metering](src/metering) defines metering facts and history APIs.
- [src/gateway.rs](src/gateway.rs) defines atomic registration and reports.
- [src/overflow](src/overflow) defines durable Boat sessions and deliveries.
- [src/overflow/admission.rs](src/overflow/admission.rs) defines required
  monthly amounts, current label facts, prepared dispatch, and its outcomes.
- [src/overflow/connection.rs](src/overflow/connection.rs) defines pending
  connection grant identities.

### Related Docs

- [Architecture](../../docs/architecture.md) gives the service boundaries.
- [Security](../../docs/protocol/security.md) defines isolation and custody.
- [Cache](../../docs/protocol/cache.md#cache-tiers) defines handling and counts.
- [Agent](../../docs/protocol/agent.md) defines credentials and removal.
- [Overflow](../../docs/protocol/overflow.md) defines Boat key rotation.
- [App removal](../../docs/protocol/app-removal.md) defines retention.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Metering](../ciproxy-metering/README.md) documents trusted producer calls.
