# ciproxy-api

The hosted dashboard API for ciproxy. The control plane mounts its axum router
under `/api`. Depend on this crate to compose sign-in, workspace access, Setup,
Machines, Settings, Jobs, Savings, and billing pages. Settings includes
[Boat under Overflow](overflow.md). This crate has no open licence.

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
Token rotation saves its encrypted replacement pair before membership checks.
A failed check preserves the previous membership snapshot and freshness time.

Requests that change data require the configured Origin and the session's CSRF
token. Every response carries `x-ciproxy-build` and `Cache-Control: no-store`.
This includes framework extraction errors and unknown routes.

The API consumes actual prepared job-area capacity through an injected
`MachineCapacity` adapter. The hosted runtime reads the exact snapshot. A report
without available capacity produces an explicit unavailable value. Label fit
includes the trusted image disk size supplied by the caller. Machine byte counts
and other large counters use decimal strings. IDs are strings. Money uses
integer US cents as decimal strings. Times use UTC RFC 3339.

Intentional take-out saves its own durable removal marker before remote cleanup.
It preserves job history and sets no owner-removal retention period. Lifecycle
replay runs after the mutation lease ends. The API and maintenance start
cleanup only after the runtime accepts that replay. The runtime accepts it
only when each runner of the organization has a saved stop, a stopped
reservation, or a protected busy job. Cleanup deletes scale sets before it
uninstalls the exact App installation.

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

Register `PUBLIC_URL/api/auth/github/callback` as the sign-in callback. Register
`PUBLIC_URL/api/github/setup` as the App setup URL. PUBLIC_URL has no trailing
slash. Keep OAuth during installation OFF, expiring user tokens ON, and redirect
on update OFF. The separate PKCE flow requests `offline_access`. M13 owns
`/hooks/github` and periodic repository refresh. Stripe uses `/hooks/stripe`.
There is no `/hooks` alias.

The public setup route treats incoming ID/state as untrusted. It reads the
configured-App installation and starts fresh user authorization. Its sealed
one-use binding captures purpose and prior token-free metadata at the start. The
callback requires the same installation/account through the fresh user token and
requires organization membership or exact personal ownership. Only then can
sign-in persistence run. Verified Add claims can then choose their workspace.
GET `/session` returns typed setup and installation-link outcomes without
frontend installation-ID forwarding.
Failed proof preserves business records and the existing sign-in. A matching
identity keeps its original expiry.

The superseded `/api/auth/callback` route and session-query bridge are gone.

Warm runners and cost have independent typed writes:

- PUT `/workspaces/{workspace_id}/machines/{machine_id}/warm-runners` changes
  only `warm_runners`. An unknown cost stays unknown and no history is added.
- PUT `/workspaces/{workspace_id}/machines/{machine_id}/cost` changes only
  `monthly_cost_cents`. It preserves the warm target. Zero is a known cost.

The old generic machine PUT has no alias. Current-cost writes read the injected
clock after acquiring their store locks.
That time closes the prior interval and starts the replacement interval.
The API supplies no historical price. Earlier unknown registered dates keep the
savings figure unavailable. This API adds no effective-date policy.

Label deletion stops new acquisition and runner registration immediately.
Retired bindings keep their job results and completion sessions. Remote cleanup
waits for tracked jobs, pending delivery, and the listener's session lease.
A draining remote binding releases its label name before a same-name re-create.
Cleanup records completion for the exact old binding and leaves its history.

HTTP path workspace IDs are opaque. Only the access boundary returns a checked
UUID after the current workspace role check. Owner claim has its own
membership-authorized opener. Request parsing and rejection order stay
unchanged.

Supply `ApiState.capacity` with a runtime adapter that returns actual slots and
optional prepared job-area bytes from each machine snapshot. The API does not
reconstruct a disk budget from hardware totals.

Set `ApiConfig.image_disk_bytes` from the trusted current release's root disk
size. Run `Maintenance::reconcile` on startup and at least hourly. It refreshes
memberships, prunes expired authentication data, and retries unfinished
intentional removals and retired-label cleanup. Use the same API collaborators.

## Organization linking

POST `/workspaces/{workspace_id}/organizations/add` returns the configured
`/installations/new?state=...` URL. GitHub still chooses the organization.
The registered setup return starts the same browser-bound OAuth proof.
`setup_action=request` stays waiting and chooses no workspace.

GET `/installation-links` refreshes the original person's installed claims.
Add requires the exact signed requester and fresh own-user installation proof.
Signed Direct claims retry without Add proof, even when the person's
installation list omits them. A provisional Direct return still waits for the
signed event.
The store joins signed identity before the access service resolves admission.
Another person's provisional return supplies no flow, proof, or destination.
One current Admin workspace links Add automatically. Several require
PUT `/installation-links/{claim_id}/workspace` with an offered workspace ID.
No eligible workspace leaves the claim waiting. Exact offers survive the
choice-to-link role-loss window. Missing state never proves a Direct flow.

A linked organization has local activation separate from GitHub App state.
Billing and Admin reasons can coexist. Every linked inactive organization
counts against the tier, excluding Removed. Existing admitted organizations
stay admitted on downgrade. A current Admin can approve with POST
`/workspaces/{workspace_id}/organizations/{organization_id}/enable`.
Only a new workspace receives a first-Admin grant. An inactive link grants
no role. Retained reinstall preserves holds and completed history.

The API returns exact-workspace upgrade actions only to Billing and Admin.
Other people get the reason and an ask-Admin capability. No held organization
can acquire work, register a JIT runner, or reserve/start overflow.
Run the existing maintenance method to reconcile billing eligibility and
retry saved choices. It discovers durable pending work after restart.

Pass the existing GitHub service to `DashboardAccess::finish_login` for this
operation. Organization linking needs no other constructor or ApiState field.
Fresh proof resolves before the identity lease. No browser-only grant is
retained.

## Jobs, Savings, and billing composition

The [pages and billing guide](pages-and-billing.md) shows how to mount the page
router and the Stripe hook. It also gives the paid review and payment steps,
and the rules of Jobs, Savings, and the shared banner.

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
- [src/handlers/links.rs](src/handlers/links.rs) owns requester choice and
  enable.
- [src/handlers/overflow.rs](src/handlers/overflow.rs) serves the Boat Setup
  and Settings routes.
- [src/handlers/overflow_views.rs](src/handlers/overflow_views.rs) builds the
  safe Boat view and converts limits and display amounts.
- [src/wire/overflow.rs](src/wire/overflow.rs) defines the Boat wire types.
- [src/maintenance.rs](src/maintenance.rs) owns restart and hourly work.
- [src/wire](src/wire) owns the Rust dashboard boundary types.
- [src/openapi.rs](src/openapi.rs) builds the generated reference.
- [src/pages.rs](src/pages.rs) composes metering, billing, and Stripe previews.
- [src/handlers/checkout.rs](src/handlers/checkout.rs) serves reviews, quote
  payments, and Checkout.
- [tests/http](tests/http) covers the HTTP contract with PostgreSQL.

### Related Docs

- [Architecture](../../docs/architecture.md) gives the hosted boundaries.
- [Security](../../docs/protocol/security.md) defines sign-in and roles.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Agent](../../docs/protocol/agent.md) defines install and machine controls.
- [Labels](../../docs/protocol/labels.md) defines label policy.
- [Overflow](../../docs/protocol/overflow.md) defines the Boat account and
  the spend limit.
- [Cache](../../docs/protocol/cache.md) defines Clear cache.
- [App removal](../../docs/protocol/app-removal.md) defines owner removal.
