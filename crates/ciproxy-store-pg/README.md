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

Atomic GitHub registration creates the workspace, defaults, organization,
and pending initial Admin together. A failed registration rolls back all
of them. GitHub repository refresh atomically replaces one snapshot.
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

Queries are synchronous.
Async services must call them from blocking workers.
This crate contains no scheduler, listener, gateway, KMS client, Boat
client, or runtime composition.

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
CI starts the database for the existing workspace test phase.

### Key Code

- [src/factory.rs](src/factory.rs) opens stores and applies migrations.
- [src/connection.rs](src/connection.rs) owns the private query connection.
- [src/workspaces.rs](src/workspaces.rs) creates shared customer tenants.
- [src/organizations.rs](src/organizations.rs) updates App installation state.
- [src/error.rs](src/error.rs) maps database failures to interface errors.
- [src/scale_sets.rs](src/scale_sets.rs) owns advisory lock sessions.
- [src/boat.rs](src/boat.rs) rotates and erases encrypted keys.
- [src/github](src/github) owns atomic installation setup, repository
  snapshots, durable job revisions, and demand retention.

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
