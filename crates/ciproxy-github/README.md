# ciproxy-github

The hosted GitHub App and runner scale-set integration for ciproxy.
The control plane uses this crate for installations, repository policies,
durable job demand, and one-use runner registrations.
This crate has no open licence.

## Responsibilities

Keep App keys and installation tokens in the control plane.
Maintain one private runner group for each organization.
Maintain one scale set for each organization and workspace label.
Hold a database session lock while each GitHub message session runs.
Save job updates before acknowledging their messages.

## What This Crate Does

The App uses these permissions only:

- Organization self-hosted runners: write.
- Repository Metadata: read.
- Repository Actions: read.

The REST adapter reads repository metadata and workflow-run metadata.
It never reads code, logs, or artifact contents.
RSA App JWTs allow one minute of clock drift and expire eight minutes
after the supplied clock time.
Installation tokens refresh before expiry and after a rejected token.
Their error and Debug output hides secret values.
The caller supplies an HTTP client that disables redirects.

### Installations and workspace ownership

An authenticated installation event uses `requester` for an approved
member request. It uses `sender` for a direct install with no requester.
The role service implements `InstallAuthority::installation_decision` for the
exact requester, account, and installation. An unambiguous direct install keeps
normal first-install and retained-reinstall behavior. An unresolved Add saves
`AwaitingAssignment` before any workspace or runner preparation.
Signed Direct evidence retries after preparation failure without Add proof.
Callback-only Direct evidence still waits for its signed creation event.
For Add, the original person must prove installed association and choose a
workspace.
App access, missing state, or the newest attempt cannot select that workspace.
The service asks for no GitHub runner right.

Each organization mutation waits for a PostgreSQL advisory lease before
reading policy, choosing a local registration, or listing repositories.
The lease uses the full GitHub organization ID in the two-key namespace.
It stays separate from listener locks. Its dedicated connection leaves
ordinary queries available while another process waits.

App-JWT lookups verify current installation identity and suspension inside
that lease. Synchronization atomically saves both the verified login and
App state. Later contexts use that saved record. A rename also restarts
live listeners so new SDK routes use the current login. A delayed event
cannot replace another current installation.
When an event still names the previous login, an InstallationPending
lookup falls back to its installation ID under the same lease. It verifies
the account ID before using that current identity. An unknown or deleted
installation stays retryable and changes nothing. Verified
Deleted events apply immediately and make no GitHub lookup. The signed
requester still selects ownership and the first Admin.
When current GitHub installation B supersedes stored installation A,
reconcile first applies A's removal bookkeeping and cleanup output.
It uses the trusted observation time. B then resolves retained ownership
or an explicit target. A delayed removal of A cannot remove active B.

Lifecycle state and immutable notifications commit in one transaction.
Each organization has one increasing delivery order. A failed callback
keeps its snapshot pending, even when a retry sees matching saved state.
Cleanup of an old installation stays ahead of replacement activation.
Snapshots keep their old workspace and last-organization result when a row
is reused or purged. Only consumer acceptance permits deletion. A crash
after acceptance can deliver the same snapshot again, so the consumer must
be idempotent. Accepted snapshots leave the outbox at once.

A first install atomically saves the workspace, its settings, its three
default labels, its organization, prepared bindings, its lifecycle event,
and a pending first-Admin assignment.
The Admin callback is idempotent. The store confirms it after delivery.
Remote scale-set preparation succeeds before an activation transaction
commits. Preparation failure commits no new activation or event. A replay
preserves label edits and does not recreate deleted labels.
A retained reinstall reuses its workspace for 30 days, unless an Admin
explicitly chose another workspace. It restores no revoked credential.
A personal install returns organizations-only setup data and a removal
link. It creates no workspace or runner resource.

`resolve_installation_link` retries one exact signed Direct claim or verified
Add choice.
`reconcile_organization_activation` retries pending admission in a workspace.
Local billing/Admin holds are separate from GitHub App state. `Inactive`
lifecycle output blocks new work for only that organization. Original-requester
Admin authority is checked at link, then preserved until a current Admin clears
an explicit hold. Already admitted organizations stay admitted on downgrade.
New workspace registration alone delivers a first-Admin grant.

Consume `Store::organization_runnable` for actual execution authority. Listener
context exposes that authority, and JIT creation requires it. Completion and
runner removal retain exact binding context. The maintenance caller handles
pending choices and activation without a new constructor or startup dependency.

### Repository access and scale sets

The [scale-set guide](scale-sets.md) gives repository refresh, the
managed runner groups, scale-set creation and reconcile, and label
replacement.

### Durable demand and registrations

Scale-set `jobId` is a UUID. It is separate from the numeric Actions job
identity used by the existing history store. `runnerRequestId` can be zero.
The durable record keeps both fields and an optional Actions job identity.
It does not invent a numeric mapping.

One atomic write saves each job update and a pending notification revision.
Exact replay does not duplicate demand. Stale replay cannot move a job
backwards. A later assignment can restart that same job UUID.
The assignment order preserves all nanoseconds. It does not depend on
PostgreSQL timestamp rounding.
The queue wait starts at `scaleSetAssignTime`. Zero times stay absent.
`queueTime` does not replace the assignment time.

Raw polling uses the SDK's public session data and the spike's HTTP shape.
The SDK still manages session creation, token refresh, acquisition,
acknowledgement, and closure. Its default decoder discards malformed
entries. This adapter instead keeps an unreadable known batch
unacknowledged. Extra JSON fields remain compatible.
Direct `JobAssigned` messages need no acquire call. An available job with
a usable request identity can use the SDK acquire operation.

The listener first replays pending notifications. It confirms each
accepted revision before acknowledging its batch. The output must be
idempotent by organization and job UUID.
The listener checks its database lease before and after each poll.
Cancellation and errors close the GitHub session before releasing the lock.
The runtime discovers listeners with `Store::listener_scale_sets` for each
active organization. The query includes retired bindings with unfinished jobs
or pending notifications. It excludes bindings with confirmed remote cleanup.
Exact binding lookup remains available through the last delivery and ACK.
A retired listener advertises zero capacity and acquires no new work.
New runner registration requires an active binding. Runner removal can still
use its retired binding. The API cleanup waits for the listener's session lease.
A saved remote-ID or login change ends a session with ScaleSetRestart.
The runtime restarts it from the current internal row. On ScaleSetMissing,
The control-plane runtime first calls refresh_repositories to reconcile.
It then retries with the
saved binding. It must not repeatedly open the obsolete remote ID.
Actual Actions-resource lookups accept GitHub's observed 204 absence
and 404. SDK errors from token or organization-auth routes remain errors.
They cannot claim runner deletion or trigger scale-set replacement.
The runtime also retries failed webhook deliveries.

`RunnerRegistrations` creates one secret JIT configuration for one runner.
Send it only to the selected runner. Use the runner's environment variable
`ACTIONS_RUNNER_INPUT_JITCONFIG`, not a command argument.
Removal accepts an already absent registration. A busy result tells the
runtime to keep the VM and let its job finish.
`retire` is the one retirement operation. It returns `Removed` with the ID,
`Absent` after a live lookup finds no runner, or `Busy` with the ID.
Missing access, token, and provider failures stay errors.
The local `Removed` App state can mean a pending take-out, so it proves no
retirement.

Default labels request 20, 20, and 60 SI GB of free job disk.
The scaler adds the image and ext4 allowance before runner allocation.

### Service integration boundaries

See [Service integration boundaries](INTEGRATION.md#service-integration-boundaries).

### Trusted run metadata

The [run metadata guide](run-metadata.md) gives the trusted run facts,
the assignment facts for the agent, and webhook delivery recovery.

## Quick Start

Run commands from the repository root:

```sh
cargo build --package ciproxy-github
cargo test --package ciproxy-github --lib
```

Set up PostgreSQL with [AGENTS.md](../../AGENTS.md#commands), then run the
fake GitHub HTTP listener through the real store:

```sh
cargo test --package ciproxy-github --test integration_tests
cargo test --package ciproxy-store-pg --test store_tests store::github
```

The tests need no GitHub credential or Boat account.
Each database test creates and removes its own database.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --package ciproxy-github --all-targets -- -D warnings
```

Unit tests use `unimock` at narrow service boundaries.
HTTP integration tests start an isolated fake GitHub service.
They use the real SDK, REST adapter, listener, and PostgreSQL queries.
Live probes require a real test organization and App installation.
They belong to the operator workflow under `scripts/`.

### Live probes

The [live-probe guide](scripts/README.md) gives credentials, prerequisites,
commands, fixture contracts, and cleanup for the live probes.

### Key Code

- [src/redelivery.rs](src/redelivery.rs) recovers failed App webhooks.

- [src/app/mod.rs](src/app/mod.rs) owns `GitHubApp` and its role and
  lifecycle callbacks.
- [src/hooks.rs](src/hooks.rs) verifies and dispatches webhook deliveries.
- [src/auth](src/auth) signs App JWTs and rotates installation tokens.
- [src/installation_state.rs](src/installation_state.rs) reads authoritative
  current App identity and state with the App JWT.
- [src/rest](src/rest) owns repository and workflow-run REST operations.
- [src/listener](src/listener) holds session locks and delivers durable
  demand before acknowledgement.
- [src/scale_sets](src/scale_sets) adapts the SDK and raw message polling.
- [src/registrations.rs](src/registrations.rs) creates and removes JIT
  registrations with explicit busy outcomes.
- [src/assignments.rs](src/assignments.rs) combines saved API metadata,
  scale-message identity, and JIT identity into the agent assignment.
- [src/persistence](src/persistence) keeps blocking store calls outside the
  asynchronous services. Its lifecycle seam is the only App state writer.
- [src/app/delivery.rs](src/app/delivery.rs) replays immutable lifecycle
  notifications and confirms them after consumer acceptance.

### Related Docs

- [Architecture](../../docs/architecture.md) defines hosted service owners.
- [Store interface](../ciproxy-store-interface/README.md) owns persistence.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Security](../../docs/protocol/security.md) defines permissions and access.
- [Labels](../../docs/protocol/labels.md) defines defaults and scale sets.
- [Job routing](../../docs/protocol/job-routing.md) defines queue waits.
- [App removal](../../docs/protocol/app-removal.md) defines lifecycle work.
