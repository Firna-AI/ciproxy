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

Defines object-safe traits for organization data and each store area.
Resource queries require the owning organization identifier.
Token authentication finds that identifier from a precomputed hash.

TokenHash and EncryptedBoatKey accept bytes from trusted callers.
Their Debug output hides the bytes.
Account metadata excludes encrypted keys.
Token metadata excludes hashes.
KMS encryption and token hashing belong to callers.

The store preserves runner placements and actual VM sizes for job history.
It accepts optional ref metadata without choosing its provenance.
The later job-facts work must choose a trusted source.
Removal and retention queries change stored records only.
They do not stop machines, call GitHub, or delete Boat sandboxes.

## Quick Start

Use the same interface for PostgreSQL and service test doubles:

```rust
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

fn organization_labels(
    store: &dyn Store,
    organization_id: Uuid,
) -> Result<Vec<Label>> {
    store.labels(organization_id)
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

- [src/store.rs](src/store.rs) defines Store and its connection factory.
- [src/error.rs](src/error.rs) owns typed failures and safe diagnostics.
- [src/secrets.rs](src/secrets.rs) owns opaque hashes and ciphertext.
- [src/scale_sets.rs](src/scale_sets.rs) defines session lock ownership.
- [src/jobs.rs](src/jobs.rs) defines metadata-only job records.

### Related Docs

- [Architecture](../../docs/architecture.md) gives the service boundaries.
- [Security](../../docs/protocol/security.md) defines isolation and custody.
- [Agent](../../docs/protocol/agent.md) defines credentials and removal.
- [Overflow](../../docs/protocol/overflow.md) defines Boat key rotation.
