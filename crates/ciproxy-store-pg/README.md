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

Scopes resource reads and writes to an organization.
Stores only caller-supplied token hashes and Boat key ciphertext.
Keeps retired Boat key versions while their sandbox runners are live.
App removal revokes credentials and erases Boat keys atomically.
Retention queries delete finished jobs after 90 days and removed
organization data after 30 days.

PgStoreFactory connects and applies pending migrations.
One connection serializes ordinary queries across clones of the store.
Each scale set lock owns a separate PostgreSQL connection.
The database releases its advisory lock when that connection closes.
The handle checks the connection so a listener can detect a lost session.
Each scale set has a distinct sequence key, without hashing its identifier.

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
Each test creates and removes a unique schema.
The database user must own the database or have CREATE permission.
Lock tests terminate only their own lock session.
CI starts the database for the existing workspace test phase.

### Key Code

- [src/factory.rs](src/factory.rs) opens stores and applies migrations.
- [src/connection.rs](src/connection.rs) owns the private query connection.
- [src/error.rs](src/error.rs) maps database failures to interface errors.
- [src/scale_sets.rs](src/scale_sets.rs) owns advisory lock sessions.
- [src/boat.rs](src/boat.rs) rotates and erases encrypted keys.
- [tests/store](tests/store) tests the public API against PostgreSQL.

### Related Docs

- [Store interface](../ciproxy-store-interface/README.md) owns the API.
- [Schema](../ciproxy-schema/README.md) owns migrations and table definitions.
- [Architecture](../../docs/architecture.md) gives the hosted boundaries.
- [Security](../../docs/protocol/security.md) defines isolation and custody.
- [App removal](../../docs/protocol/app-removal.md) defines removal.
