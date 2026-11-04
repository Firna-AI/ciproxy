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

The gateway queries atomically register a machine and its hashed credential.
GatewayStore reuses TokenStore authority lookup to derive the workspace from
the install hash. Registration rechecks the token under its transaction
locks before it inserts the machine.
They retain the full typed protocol report with a connection identifier.
Report writes preserve desired drain and warm settings.
They reject replaced sessions and removed registrations.
Gateway token issuance and registration serialize with workspace removal.

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

- [src/store.rs](src/store.rs) defines Store and its connection factory.
- [src/workspaces.rs](src/workspaces.rs) defines shared customer ownership.
- [src/organizations.rs](src/organizations.rs) defines GitHub App records.
- [src/error.rs](src/error.rs) owns typed failures and safe diagnostics.
- [src/secrets.rs](src/secrets.rs) owns opaque hashes and ciphertext.
- [src/scale_sets.rs](src/scale_sets.rs) defines session lock ownership.
- [src/jobs.rs](src/jobs.rs) defines metadata-only job records.
- [src/gateway.rs](src/gateway.rs) defines atomic registration and reports.

### Related Docs

- [Architecture](../../docs/architecture.md) gives the service boundaries.
- [Security](../../docs/protocol/security.md) defines isolation and custody.
- [Cache](../../docs/protocol/cache.md#cache-tiers) defines handling and counts.
- [Agent](../../docs/protocol/agent.md) defines credentials and removal.
- [Overflow](../../docs/protocol/overflow.md) defines Boat key rotation.
- [App removal](../../docs/protocol/app-removal.md) defines retention.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
