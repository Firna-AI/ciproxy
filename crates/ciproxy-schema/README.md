# ciproxy-schema

The PostgreSQL schema and migrations of the hosted ciproxy control plane.
Store implementations depend on this crate.
Service code uses ciproxy-store-interface instead.
This crate has no open licence.

## Responsibilities

Own table definitions and schema history.
Keep one Diesel schema module for each area.
Keep database rows inside the store implementation.

## What This Crate Does

Embeds ordered forward and reverse SQL migrations.
Defines workspaces as the customer tenant.
Workspaces own labels, machines, tokens, settings, and Boat key versions.
Each GitHub organization has at most one current workspace registration.
Removed registrations keep their old workspace history during retention.
It owns its installation, repository access, scale sets, runners, and jobs.
Composite foreign keys enforce both ownership boundaries.
One workspace machine can serve all organizations in that workspace.
The machine area also stores the last report of each disk.
The gateway report table retains the full latest protocol report as JSONB.
Its connection identifier fences reports from replaced agent connections.

The schema stores token hashes and encrypted Boat keys.
It stores job metadata and references to immutable runner placements.
Cache measurements and restored content origins remain independent.
The schema accepts measured GitHub-only fetches and zero-entry restores.
It has no job-content, job-log, plaintext-token, or plaintext-key field.
Metering tables hold trusted times, costs, and billing-period boundaries.
Anonymous aggregates hold daily VM-size totals and Boat costs.
Anonymous timestamp counts preserve exact period starts.
Job and runner accounting cursors cascade with their source records.
Anonymous aggregates belong only to the workspace.
They survive organization deletion until their own retention expires.

Billing accounts keep confirmed typed state and durable command inputs.
Their creation time supplies the initial Free period anchor.
Each has a distinct negative advisory-lock key.
Billing events keep committed receipt identifiers.
Billing renewals retain invoice outcomes and verified automatic collection
stops.
One forward migration preserves legacy pending stops as eligible per-event
decisions and retains the latest collection-change time.
Their workspace ownership and subscription index keep queries scoped.
The last organization removal keeps a durable cancellation request.
Later milestones own members and roles.
Workspace deletion cascades through its retained records after 30 days.

The GitHub area adds visible repository snapshots and pending initial
Admin delivery. UUID scale-set demand is separate from numeric Actions
history. Each demand record keeps a monotonic assignment and phase, its
trusted raw run facts, and durable notification revisions.
The assignment order keeps nanoseconds independently of timestamp storage.
The first trusted completion observation bounds retention when GitHub
omitted its finish time. It does not replace that absent GitHub field.
These records hold metadata only. They hold no runner registration secret.
A forward M6 index supports scoped trusted run-facts lookup.
Organization advisory leases need no persistent schema key rows.
The lifecycle outbox stores ordered, immutable organization metadata.
It survives organization/workspace purge until the consumer accepts it.
It contains no key, token, JIT configuration, job content, or job log.
Confirmed events are deleted instead of retained indefinitely.

Overflow tables hold credential-free account observations and durable
sandbox deliveries across every organization of a workspace.
One pending connection grant per workspace holds a fresh operation identity
and the intended key-version identity.
It holds no plaintext key or ciphertext.
Last-App removal deletes the grant, and successful activation consumes it.
Each delivery holds a callback digest and temporary encrypted runner input.
Provider identity, phase, start-price quote, and per-month runtime facts
survive a control-plane restart.
Unfinished deliveries keep their Boat key reference during rotation.
The overflow advisory-key sequence has its own reserved range.
Delivery metadata has no cascading organization reference, so retention
cannot discard live cleanup authority when one organization leaves.

## Quick Start

Use the migration constant through Diesel's migration harness:

```rust
use ciproxy_schema::migrations::MIGRATIONS;
use diesel::PgConnection;
use diesel_migrations::MigrationHarness;

fn migrate(
    connection: &mut PgConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
```

The PostgreSQL store factory applies pending migrations when it connects.
The factory serializes migration runs with a database session lock.

## Development

Run commands from the repository root:

```sh
cargo build --package ciproxy-schema
cargo fmt --all -- --check
cargo clippy --package ciproxy-schema --all-targets -- -D warnings
```

The [store tests](../ciproxy-store-pg/README.md#development) use a real
PostgreSQL database.
New migration names use a full date, hour, and minute prefix.
Each migration has an up.sql and a down.sql.

### Key Code

- [src/billing.rs](src/billing.rs) defines billing accounts and event receipts.

- [build.rs](build.rs) tracks migration directory changes for Cargo.
- [src/migrations.rs](src/migrations.rs) exports the embedded migrations.
- [migrations](migrations) holds the forward and reverse schema history.
- [src/workspaces.rs](src/workspaces.rs) defines the shared customer tenant.
- [src/organizations.rs](src/organizations.rs) defines GitHub installations.
- [src/runners.rs](src/runners.rs) defines stored runner placements.
- [src/github.rs](src/github.rs) defines repository snapshots and durable
  UUID job notifications.
- [src/metering.rs](src/metering.rs) defines histories and aggregates.
- [src/overflow.rs](src/overflow.rs) defines durable sandbox execution state.

### Related Docs

- [Architecture](../../docs/architecture.md) gives the hosted boundaries.
- [Security](../../docs/protocol/security.md) defines isolation and custody.
- [Cache](../../docs/protocol/cache.md#cache-tiers) defines handling and counts.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Labels](../../docs/protocol/labels.md) defines labels.
- [App removal](../../docs/protocol/app-removal.md) defines retention.
- [Metering](../../docs/protocol/metering.md) defines counts and reports.
