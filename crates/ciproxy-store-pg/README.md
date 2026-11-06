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
The full last report uses JSONB and a connection identifier.
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
credential revocation and the last-organization result in the same commit.
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

Overflow uses one dedicated advisory-lock connection per workspace.
Its negative key range is separate from billing and scale-set locks.
A blocking callback lease waits on that dedicated connection.
Catalog observations and exact delivery records persist before Boat access.
Reservation rechecks the active key, workspace settings, organization,
label, and scale-set ownership under the workspace row lock.
Pending acceptance saves demand, sealed input, and the saved fee together.
It saves no fixed provider parameters or provider cost bound.
Pending connections receive fresh database-generated grant identities.
Activation consumes a matching live grant under the workspace row lock.
A later grant cannot restore an older operation, even with the same key ID.
Suspension and removal with another App present preserve shared grants.
One current startup query binds the saved delivery to its organization and
key in that workspace. It requires active App state, enabled overflow, and
the accepted non-retired key together.
The same authority SQL checks scale-set ownership, the label's Boat switch,
and exact original requested dimensions during reservation and later startup.
An undersized selected size also requires the label's current largest-size
fallback permission. Fitting requests remain permitted under Wait.
Each delivery retains its image root bytes and required total root bytes.
Authority compares catalog disk with those saved totals.
Even a largest-size fallback must hold the complete image.
The dispatch transaction accepts only the saved unsent delivery.
It rechecks authority and the current saved limit before saving `Creating`.
It compares the exact pending record, including its identity and saved fee.
It saves selected size, lifetime, and dispatch time in that same transaction.
The caller supplies those parameters with their required monthly amounts.
Current-label queries supply facts for service policy without selecting sizes.
Cancelled unsent records release sealed input without provider or cost facts.
Adoption saves the provider identity, immutable runner placement, and price
facts in one transaction.
Callback claims save token consumption and deletion intent together.
Refusal records and account restrictions also commit together.
Key erasure checks unfinished deliveries as well as live runners.
No query receives a plaintext key or runner registration.
Metering price facts require a trusted runtime observation and a counted
start fee. They select the confirmed fee, or else the saved fee bound.
That selection never confirms a provider charge.
Delivery JSON retains the admission bound and actual fee separately.
Their upserts correct provider start times and every price field.
Every overflow price row has a finite end from its measured interval.
Later metering refreshes cannot extend an old reading.
An unknown final interval publishes only the known measured billing prefix.
Boat accounting remains available so the metering refresh can reach purge.
Relevant savings reads return `AccountingIncomplete` while both fee sources
are missing or a required runtime interval remains unknown.
The savings range excludes a lifecycle end at its first instant.
Execution and cleanup do not use those reporting reads.

The dashboard area stores verified GitHub memberships, workspace roles,
access requests, opaque session digests, and encrypted OAuth grants.
A person record stays only with a role or an open request. Intentional
organization removal keeps a separate durable marker and no owner-removal
retention deadline. Retired labels preserve history while active names
remain unique.

Credential-only rotation writes preserve membership freshness. Current-cost
writes read the supplied clock after locking the workspace and machine rows.
They use that time for both price interval boundaries.

Retired scale-set bindings keep completion context and durable cleanup state.
Listener discovery includes unfinished work and pending notification revisions.
Confirmed remote removal excludes the old binding from later cleanup and
discovery without deleting job history.

Own-machine queries save runner and command reservations atomically.
A repeatable-read routing snapshot keeps reports, demand, and settings
consistent. A separate negative advisory-key range owns execution leases.
Mutable machine, organization, and label authority is checked again under
row locks before a new reservation commits.
The command records retain exact byte dimensions and logical slot sets.
No conversion through integer GB changes the held root allocation.

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

- [src/dashboard](src/dashboard) owns dashboard identity and access queries.
- [src/own_runners.rs](src/own_runners.rs) stores exact execution intents.
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
- [src/scale_set_tracking.rs](src/scale_set_tracking.rs) owns retired binding
  context, listener discovery, and exact remote cleanup confirmation.
- [src/boat.rs](src/boat.rs) rotates and erases encrypted keys.
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
