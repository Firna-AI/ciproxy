# ciproxy-api

The hosted dashboard API for ciproxy. The control plane mounts its axum router
under `/api`. Depend on this crate to compose sign-in, workspace access, Setup,
Machines, and non-Boat Settings. This crate has no open licence.

## Responsibilities

Own the dashboard HTTP contract in Rust. Require the workspace role for all
private reads and changes. Use the existing GitHub App and machine gateway
services. Keep credentials out of response diagnostics.

## What This Crate Does

GitHub App OAuth uses one-use browser-bound state and PKCE S256. A sealed Secure
HttpOnly cookie keeps the fixed 30-day sign-in. It contains profile and session
metadata, with no GitHub token. The store keeps a person, session digest, and
encrypted OAuth grant only while a role or an open access request exists. A
deleted session leaves only the signed-in install or access page. It cannot
authorize a private workspace route.

The session response provides a same-origin reauthorization URL. The frontend
uses top-level navigation to get fresh GitHub proof for Check again or
promotion. A matching GitHub identity keeps the first sign-in expiry.
Reauthorization rotates session and CSRF secrets. Membership refresh uses the
latest stored grant under a database lease. It does not create another access
request after an Admin declines one.

Requests that change data require the configured Origin and the session's CSRF
token. Every response carries `x-ciproxy-build` and `Cache-Control: no-store`.
This includes framework extraction errors and unknown routes.

The API consumes actual prepared job-area capacity through an injected
`MachineCapacity` adapter. The hosted runtime reads the exact snapshot. A report
without available capacity produces an explicit unavailable value. Label fit includes
the trusted image disk size supplied by the caller. Machine byte counts and
other large counters use decimal strings. IDs are strings. Money uses integer US
cents as decimal strings. Times use UTC RFC 3339.

Intentional take-out saves its own durable removal marker before remote cleanup.
It preserves job history and sets no owner-removal retention period. Cleanup
deletes scale sets before it uninstalls the exact App installation. Lifecycle
replay runs after the mutation lease ends.

The generated [OpenAPI document](../../ts/api/generated/openapi.json) is the
route and schema reference. The TypeScript package
[@ciproxy/api](../../ts/api/README.md) generates the dashboard types.

## Quick Start

Mount the router from the hosted composition root:

```rust
use axum::Router;
use ciproxy_api::router::router;
use ciproxy_api::state::ApiState;

fn application(state: ApiState) -> Router {
    Router::new().nest("/api", router(state))
}
```

Construct `AccessService` first. Coerce cloned Arcs to `DashboardAccess`,
`ciproxy_gateway::boundary::Authorization`, and
`ciproxy_github::app::InstallAuthority`. Then construct the gateway and App
service. They do not form a construction cycle.

Supply a redirect-free reqwest client to `GithubOAuthClient` and
`GithubInstallationAdmin`. Load the OAuth client secret and the 32-byte
credential encryption key from the operator's secret store. Keep the key stable
across restarts and for all replicas that serve the same sign-ins. Keep each
machine's API callers and WebSocket on the same gateway owner.

Register `/api/auth/callback` as the GitHub sign-in callback on the dashboard
host. Set the App setup URL to that host's `/` page. Disable "Request user
authorization during installation". The dashboard forwards the setup
return's `installation_id` to `/api/session` for verification.

Supply `ApiState.capacity` with a runtime adapter that returns actual slots and
optional prepared job-area bytes from each machine snapshot. The API does not
reconstruct a disk budget from hardware totals.

Set `ApiConfig.image_disk_bytes` from the trusted current release's root disk
size. Run `Maintenance::reconcile` on startup and at least hourly. It refreshes
memberships, prunes expired authentication data, and retries unfinished
intentional removals and retired-label cleanup. Use the same API collaborators.

## Development

Run commands from the repository root. Set up PostgreSQL as
[AGENTS.md](../../AGENTS.md#commands) describes.

```sh
cargo build --package ciproxy-api
cargo test --package ciproxy-api --test http_tests
cargo clippy --package ciproxy-api --all-targets -- -D warnings
npm --prefix ts run api:generate
npm --prefix ts run api:generate:check
```

The HTTP tests use real PostgreSQL, the gateway, AppService, and advisory
leases. Controlled external services replace GitHub. The adapter tests run the
production OAuth client and scale-set SDK against a local HTTP peer. They
require no live credential.

[Live verification](scripts/README.md) uses an operator-owned credential file.
It never prints a cookie, token, install command, or response body.

### Key Code

- [src/state.rs](src/state.rs) defines the hosted composition boundary.
- [src/router.rs](src/router.rs) registers routes and browser protection.
- [src/access](src/access) owns sign-in and shared role adapters.
- [src/identity/http.rs](src/identity/http.rs) implements GitHub user OAuth.
- [src/installations.rs](src/installations.rs) implements App-JWT metadata reads
  and intentional uninstall.
- [src/maintenance.rs](src/maintenance.rs) owns restart and hourly work.
- [src/wire](src/wire) owns the Rust dashboard boundary types.
- [src/openapi.rs](src/openapi.rs) builds the generated reference.
- [tests/http](tests/http) covers the HTTP contract with PostgreSQL.

### Related Docs

- [Architecture](../../docs/architecture.md) gives the hosted boundaries.
- [Security](../../docs/protocol/security.md) defines sign-in and roles.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Agent](../../docs/protocol/agent.md) defines install and machine controls.
- [Labels](../../docs/protocol/labels.md) defines label policy.
- [Cache](../../docs/protocol/cache.md) defines Clear cache.
- [App removal](../../docs/protocol/app-removal.md) defines owner removal.
