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
Later milestones own members, roles, billing, and metering.
Workspace deletion cascades through its retained records after 30 days.

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

- [build.rs](build.rs) tracks migration directory changes for Cargo.
- [src/migrations.rs](src/migrations.rs) exports the embedded migrations.
- [migrations](migrations) holds the forward and reverse schema history.
- [src/workspaces.rs](src/workspaces.rs) defines the shared customer tenant.
- [src/organizations.rs](src/organizations.rs) defines GitHub installations.
- [src/runners.rs](src/runners.rs) defines stored runner placements.

### Related Docs

- [Architecture](../../docs/architecture.md) gives the hosted boundaries.
- [Security](../../docs/protocol/security.md) defines isolation and custody.
- [Cache](../../docs/protocol/cache.md#cache-tiers) defines handling and counts.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Labels](../../docs/protocol/labels.md) defines labels.
- [App removal](../../docs/protocol/app-removal.md) defines retention.
