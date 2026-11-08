# ciproxy-store-pg

The Diesel and PostgreSQL adapter for the hosted ciproxy store.
Composition roots construct its factory.
Services consume the resulting dyn Store.
This crate has no open licence.

## Responsibilities

Implement database queries and private row mappings.
Integrate the shared schema and migrations.
Return the error contract of ciproxy-store-interface.

## What This Crate Does

Scopes resource reads and writes to a workspace.
Keeps GitHub installations and repository access per organization.
Keeps each scale set, runner, and job attached to its organization.
Shared machines can serve every organization of their own workspace.
Stores only caller-supplied token hashes and Boat key ciphertext.
Keeps retired Boat key versions while their sandbox runners are live.
Key erasure waits for concurrent runner inserts before it checks references.
Removing one organization preserves live sandbox-end tokens.
It preserves shared execution resources while another organization remains.
Removal of the last App revokes machine and install credentials.
It retires Boat keys and erases those that no live sandbox needs.
Live sandbox-end tokens stay usable after App removal.
They name one workspace, organization, and runner.
They stop working after that runner reaches its stopped state.
These mutations run in one transaction under a workspace row lock.
The last removal also deletes pending Boat connection grants.
Install, removal, and purge queries serialize on that workspace lock.
Retention queries delete finished jobs after 90 days.
They delete removed organization data after 30 days.
The workspace purge deletes one empty workspace after its 30-day deadline.
A reinstall clears that deadline without restoring execution credentials.
It preserves labels, settings, and job resource history.
Dashboard jobs read current label requests only while queued.
Executed jobs read the retained own-machine or Boat request by runner identity.
Missing retained requests stay unknown, including their vCPUs and decimal-GB
memory and free job disk. Actual allocation stays separate.
Job rows map machine-handled entry and byte measurements as one optional bundle.
The content source stays independent of measurement presence.
Measured GitHub-only and zero-entry restores both round-trip.
Direct GitHub use and fallback keep no measured counts.
Global GitHub-ID reads return current and retained registrations.
A partial unique index permits only one current registration.
An install into another workspace creates a separate registration.
It transfers no history or shared resource from the old workspace.
Individual and workspace removal preserve each machine's first removal time.

One helper gives workspace usage, savings, and Boat accounting read-only
repeatable-read snapshots.
Boat source costs and cost cursors share one repeatable-read snapshot.
Savings registrations, range totals, and monthly costs share one
repeatable-read read-only snapshot.
The savings read includes removed registrations without loading retained
jobs, start counts, or accounting cursors.
Daily aggregates keep VM sizes and Boat costs without job identifiers.
Timestamp aggregates keep exact starts without organization identifiers.
Cursor writes apply caller-supplied signed corrections atomically.
They return false for a stale cursor or a deleted source record.
Accounting and Boat history writes lock the workspace before its children.
Deleting organization data preserves anonymous workspace totals.
The metering service computes the rules and supplies the retention cutoff.
Call its refresh before deleting retained job or organization data.
The saved retention cutoff prevents stale refreshes from restoring old dates.

Atomic GitHub registration creates the workspace, defaults, organization,
and pending initial Admin together. A failed registration rolls back all
of them. GitHub repository refresh atomically replaces one snapshot.
Workspace creation and first GitHub registration share one transaction
helper. It inserts the workspace, initial Free billing, and settings.
It also records the trusted creation time, zero-dollar Free cost, and
initial Free period from the same billing creation anchor.
GitHub labels, pending Admin, and lifecycle events remain in the enclosing
registration transaction.
UUID scale-set demand stays separate from numeric Actions job history.
Each demand write saves a pending delivery revision in the same statement.
Stale replay cannot replace a later assignment or a terminal state.
An exact UTC text key preserves nanosecond assignment order across the
microsecond precision of PostgreSQL timestamps.
Delivery confirmation cannot consume a newer revision.
The GitHub retention query deletes completed demand after 90 days,
including pending deliveries. Active work stays.
When GitHub omitted its completion time, the first trusted observation
supplied by the caller starts the period. Exact replay keeps that instant.
The query stores this observation separately from GitHub's metadata.

Gateway queries lock the workspace before registration or token issuance.
They serialize against last-App removal and reject a retained empty workspace.
Registration inserts the machine, report, and hashed credential together.
It holds the install-token row until that transaction commits.
Independent token revocation waits for that registration before it returns.
Registrations that validate the token after revocation cannot use it.
The full last report uses JSONB, a connection identifier, and the receipt
time that only a full report writes. Snapshots read both from one row.
Replaced sessions and removed machines cannot overwrite it.
Desired drain and warm settings stay separate from observed report fields.
Machine reads keep machine fields and disks in one committed view.
Gateway snapshots include the full report from that same view.
A shared read snapshot prevents mixed report generations.
Contact writes contain only observed columns.

PgStoreFactory connects and applies pending migrations.
One connection serializes ordinary queries across clones of the store.
Each scale set lock owns a separate PostgreSQL connection.
The database releases its advisory lock when that connection closes.
The handle checks the connection so a listener can detect a lost session.
Each scale set has a distinct sequence key, without hashing its identifier.
Organization mutations use waiting leases on other dedicated connections.
Two signed 32-bit keys preserve all 64 GitHub organization-ID bits.
Their advisory namespace is separate from one-key scale-set locks.
Waiting takes no ordinary shared-query connection and creates no key row.
Trusted run-facts lookups use a tenant/organization/run index.
Remote scale-set upserts retain internal UUIDs, lock keys, and history.
Managed-group lookup can reuse a retained organization registration.
A verified installation-state update saves the current login and App state
in one statement and returns the saved organization. Workspace, account,
installation, and non-removed state guards prevent stale identity writes.
The original suspension-only API stays available to its existing callers.
The GitHub App uses one transactional lifecycle path for registration,
verified state, removal, and notification insertion. Prepared repository
snapshots and scale-set bindings commit with activation. Removal captures
credential revocation, the last-organization result, and the required
removal cause in the same commit.
Registration uses the same admission predicate as GitHub preparation.
Retained admission takes precedence over the initial holds of a new Add.
Outbox events keep immutable metadata snapshots and sequence order. They
have no cascading workspace or organization reference. Pending cleanup
survives row reuse and retention purge. Confirmation deletes an accepted
event. Pending-organization discovery supports replay after restart.

Billing uses a separate live session lock per workspace.
Its subscription-ID query uses the retained unique billing-account identity.
Each command persists before an external Stripe call.
The shared read model exposes confirmed coverage and UTC period anchors.
A transaction commits event receipts, state, and command completion together.
A failed state write rolls its receipt back.
Renewal outcomes use separate workspace-scoped invoice records.
They preserve recovery and verified automatic stops after banner fields clear.
An exclusive session saves stop evidence before a provider cancellation request.
The same renewal record retains every stop event's cancellation decision.
Superseded and completed decisions survive later events and restarts.
The last organization removal also persists a cancellation request.
It stays until billing handles it, including unfinished Checkout.

Queries are synchronous.
Async services must call them from blocking workers.
This crate contains no scheduler, listener, gateway, KMS client, Boat
client, or runtime composition.

The [overflow store guide](overflow.md) gives the Boat execution sessions,
delivery authority, dispatch, callbacks, runtime registration identities,
and Boat accounting reads.

The dashboard area stores verified GitHub memberships, workspace roles,
access requests, opaque session digests, and encrypted OAuth grants.
Session establishment commits the exact claim continuation, the consumption of
its pending results, and session rotation together. The continuation uses the
existing browser session ID and fixed expiry.
It survives person pruning without a person reference, profile, or OAuth grant.
Authentication expiry cleanup deletes expired continuations.
A claim save writes its pending result in the same transaction. Consumption
deletes only the exact revisions that the session read, so a later write
survives. A pending result stays until a successful session consumes it or its
claim is deleted.
A person record stays only with a role or an open request. Intentional
organization removal keeps a separate durable marker and no owner-removal
retention deadline. Retired labels preserve history while active names
remain unique.
Maintenance discovers workspaces with unfinished retired bindings, then
reads each organization's bindings for cleanup.

Credential-only rotation writes preserve membership freshness. Current-cost
writes read the supplied clock after locking the workspace and machine rows.
They use that time for both price interval boundaries.

Retired scale-set bindings keep completion context and durable cleanup state.
Listener discovery includes unfinished work and pending notification revisions.
Confirmed remote removal excludes the old binding from later cleanup and
discovery without deleting job history.

Own-machine queries save runner and command reservations atomically.
A repeatable-read routing snapshot keeps reports, demand, and settings
consistent. It also reads the Boat account, deliveries, and registration
identities. A separate negative advisory-key range owns execution leases.
Mutable machine, organization, and label authority is checked again under
row locks before a new reservation commits.
The command records retain exact byte dimensions and logical slot sets.
No conversion through integer GB changes the held root allocation.
The first confirmation of a runner is written once, and retirement keeps it.
An observation saves the runner state, its GitHub ID, and that first time in
one transaction. A missing runner or command record rolls back both writes.
An applied command result stays a separate value.
A retirement saves the trusted GitHub ID, `Stopping`, and its stop command
in one transaction.

## Quick Start

```rust
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let factory: &dyn StoreFactory = &PgStoreFactory;
    let store = factory.connect(&database_url)?;
    let _ = store;
    Ok(())
}
```

Keep the database URL in runtime configuration.
Use the returned store through `Arc<dyn Store>`.
Keep a returned scale set lock handle alive until the listener stops.

## Development

Install libpq development files.
Start PostgreSQL 15 with the commands in
[AGENTS.md](../../AGENTS.md#commands).
Set DATABASE_URL before store tests or the complete repository gate:

```sh
export DATABASE_URL=postgresql://ciproxy:ciproxy@127.0.0.1:5432/ciproxy_test
cargo build --package ciproxy-store-pg
cargo test --package ciproxy-store-pg --test store_tests
cargo fmt --all -- --check
cargo clippy --package ciproxy-store-pg --all-targets -- -D warnings
```

Tests fail when DATABASE_URL is missing.
Each test creates and removes a unique database on the same server.
The database user needs CREATEDB or superuser rights.
The provided database serves as the maintenance connection.
The fixture removes only the database that it created.
Separate databases isolate sequence keys and advisory locks.
Lock tests terminate only their own lock session.
Metering concurrency tests coordinate tagged sessions through exact database
lock dependencies. They check production cursor writes and report snapshots.
CI starts the database for the existing workspace test phase.

### Key Code

- [Installation queries](src/installation_links/README.md) retain exact claims,
  pending authorization results, and local execution admission.
- [src/dashboard](src/dashboard) owns dashboard identity and access queries.
- [src/own_runners.rs](src/own_runners.rs) stores exact execution intents.
- [src/own_observation.rs](src/own_observation.rs) saves one scoped runner
  observation and its first confirmation.
- [src/routing_snapshot.rs](src/routing_snapshot.rs) reads coherent routing.

- [src/billing/session.rs](src/billing/session.rs) commits billing state.
- [src/billing/renewals.rs](src/billing/renewals.rs) stores owned renewal
  outcomes.
- [src/factory.rs](src/factory.rs) opens stores and applies migrations.
- [src/connection.rs](src/connection.rs) owns the private query connection.
- [src/workspaces.rs](src/workspaces.rs) creates shared customer tenants.
- [src/organizations.rs](src/organizations.rs) updates App installation state.
- [src/error.rs](src/error.rs) maps database failures to interface errors.
- [src/scale_sets.rs](src/scale_sets.rs) owns advisory lock sessions.
- [src/job_views.rs](src/job_views.rs) reads tier-visible dashboard jobs,
  preserves trusted alias metadata and retained requests, and keeps actual
  runner dimensions.
- [src/scale_set_tracking.rs](src/scale_set_tracking.rs) owns retired binding
  context, listener discovery, and exact remote cleanup confirmation.
- [src/boat.rs](src/boat.rs) rotates and erases encrypted keys.
- [src/boat_runtime.rs](src/boat_runtime.rs) stores Boat registration
  identities.
- [src/overflow](src/overflow) owns durable Boat execution transactions.
- [src/overflow/authority.rs](src/overflow/authority.rs) reads current
  startup authority for one saved delivery.
- [src/overflow/connections.rs](src/overflow/connections.rs) issues pending
  connection grants under the workspace removal lock.
- [src/github](src/github) owns atomic installation setup, repository
  snapshots, durable job revisions, and demand retention.
- [src/machine_removal.rs](src/machine_removal.rs) preserves removal times.
- [src/metering](src/metering) stores cost histories and accounting deltas.
- [src/metering/snapshot.rs](src/metering/snapshot.rs) configures shared
  metering read transactions.
- [src/gateway.rs](src/gateway.rs) owns atomic gateway persistence.
- [src/machine_reads.rs](src/machine_reads.rs) loads coherent machine views.
- [src/machine_observation.rs](src/machine_observation.rs) limits contact
  writes to observed columns.
- [tests/store](tests/store) tests the public API against PostgreSQL.

### Related Docs

- [Store interface](../ciproxy-store-interface/README.md) owns the API.
- [Schema](../ciproxy-schema/README.md) owns migrations and table definitions.
- [Architecture](../../docs/architecture.md) gives the hosted boundaries.
- [Security](../../docs/protocol/security.md) defines isolation and custody.
- [Cache](../../docs/protocol/cache.md#cache-tiers) defines handling and counts.
- [App removal](../../docs/protocol/app-removal.md) defines removal.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Metering](../ciproxy-metering/README.md) documents trusted producer calls.
