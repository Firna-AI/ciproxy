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
The setup URL does not choose the workspace.
The role service implements `InstallAuthority`.
It resolves the persisted intent of the Admin who started adding an
organization. It checks the Admin role and the Free organization limit.
It does not ask for a GitHub runner right.

A first install atomically saves the workspace, its settings, its three
default labels, its organization, and a pending first-Admin assignment.
The Admin callback is idempotent. The store confirms it after delivery.
A replay preserves label edits and does not recreate deleted labels.
A retained reinstall reuses its workspace for 30 days, unless an Admin
explicitly chose another workspace. It restores no revoked credential.
A personal install returns organizations-only setup data and a removal
link. It creates no workspace or runner resource.

### Repository access and scale sets

Repository refresh reads every installation page and saves the default
branches. The group's selected repositories intersect the customer's
choice with visible private repositories. Public access stays false.
An all-private choice includes new visible private repositories on refresh.
Subscribe the App to repository metadata events. Dispatch `repository` and
`installation_repositories` through the authenticated webhook handler.
The control plane can also call `refresh_repositories` after a reconnect.

The group name includes the internal organization identity. This keeps
the App's policy separate from the organization's other runner groups.
GitHub supports self-hosted runner groups on its Free plan, as its
[Free runner-group announcement][free-groups] confirms.
The live selected-group probe still checks this on a real installation.

The adapter uses `scaleset` 0.2.0. The foundation live spike used 0.1.0.
The integration tests check the current version with the spike's wire
format and nanosecond timestamps.
Scale-set creation uses GitHub's label acceptance and disables runner
updates. The image release process controls runner versions.
The API also exposes scale-set deletion for the later label and take-out
services. It does not erase retained job history.

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
The runtime starts one listener for each active organization-label pair.
It restarts failed listeners and retries failed webhook deliveries.

`RunnerRegistrations` creates one secret JIT configuration for one runner.
Send it only to the selected runner. Use the runner's environment variable
`ACTIONS_RUNNER_INPUT_JITCONFIG`, not a command argument.
Removal accepts an already absent registration. A busy result tells the
runtime to keep the VM and let its job finish.

### Service integration boundaries

The crate consumes the hosted store through `dyn Store` and blocking
workers. The later control-plane runtime supplies these adapters:

- `InstallAuthority` resolves installation intents and grants the first
  Admin role. The sign-in service owns memberships and role storage.
- `LifecycleOutput` stops idle organization runners and their listeners.
  It preserves running jobs during removal and suspension.
- After the last organization leaves, the store revokes machine and
  install credentials. The output closes machine connections, requests
  idle sandbox cleanup, and schedules the subscription end. The overflow
  service keeps retired Boat keys until running sandbox cleanup finishes.
- `JobOutput` notifies the snapshot and scaler service of saved demand.
  That service maps GitHub runner IDs to its runner records.
  It supplies consistent snapshots and sends accepted actions to agents.
- The retention worker calls `purge_github_jobs` for completed demand after
  90 days, including records with pending notifications. Active work stays.
  When GitHub omits `finishTime`, expiry uses the first trusted completion
  observation from the store adapter's injected `Runtime`. Replay keeps
  that observation. The GitHub completion field stays absent.

No adapter in this crate calls Boat.
An owner removal cannot delete inaccessible GitHub resources.
Suspension preserves the workspace and its credentials.
Unsuspend reconciles the repositories and resumes lifecycle delivery.
The dashboard take-out service owns the intentional-removal distinction.

### Trusted run metadata

Run facts come from the Actions read API for the message's repository and
run ID. They preserve `event`, `head_branch`, `head_sha`, and nullable
`head_repository` from that API. A reusable workflow path stays separate.

The API does not supply a guaranteed full job ref. Its `head_branch` does
not distinguish a same-name branch and tag. Associated pull requests do
not identify the triggering pull request. A deleted head repository stays
absent. This crate makes no event-to-ref conversion and grants no cache
or Docker trust from these fields.
The Actions API remains the approved source today. A complete ref source
and an unknown-ref policy still need decisions.
Full-ref forwarding to the agent therefore remains incomplete.

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

Install Python 3, OpenSSL, and `gh` on Linux.
Keep the App JSON outside tracked files. It has `id` and `pem` fields.
The `id` accepts a number or a string. The `pem` holds the private key.
Authenticate `gh` separately as an operator who can create fixture refs
and workflows, cancel their runs, and read the organization plan.
The App keeps its three permissions. Tokens never go on command arguments.

```sh
export CIPROXY_GITHUB_APP_PATH=/absolute/path/to/app.json
export PYTHONPYCACHEPREFIX="$PWD/.context/ciproxy-mvp/milestone-6/pycache"
python3 crates/ciproxy-github/scripts/runner_group_probe.py --help
python3 crates/ciproxy-github/scripts/ref_metadata_probe.py --help
python3 crates/ciproxy-github/scripts/push_ref_probe.py --help
python3 crates/ciproxy-github/scripts/run_job_ref_probe.py --help
```

Each help page gives its prerequisites, resource scope, and cleanup steps.
Use two private disposable repositories with initial commits for the
selected-group probe. Install the same App on both repositories.
Provide an executable launcher for a clean, unconfigured Linux runner.
It receives JIT only in `ACTIONS_RUNNER_INPUT_JITCONFIG`.
It copies the runner into `CIPROXY_GITHUB_PROBE_WORK_DIR` and runs it there.
The script withholds operator credentials from the runner environment.

```sh
python3 crates/ciproxy-github/scripts/runner_group_probe.py \
  --allowed-repo Firna-AI/ciproxy-spike \
  --refused-repo Firna-AI/ciproxy-spike-refused \
  --runner-launcher /absolute/path/to/launcher
python3 crates/ciproxy-github/scripts/ref_metadata_probe.py \
  --repo Firna-AI/ciproxy-spike --create-temp
python3 crates/ciproxy-github/scripts/push_ref_probe.py \
  --repo Firna-AI/ciproxy-spike
python3 crates/ciproxy-github/scripts/run_job_ref_probe.py \
  --run "pull-request=Firna-AI/ciproxy-spike:$PR_RUN_ID"
```

Set `PR_RUN_ID` to an existing run. Repeat `--run` for other supplied cases.
The push probe creates both same-name branch and tag push runs itself.
It needs no existing workflow or run ID. It uses `ubuntu-latest` and one
`true` step. It needs no ciproxy runner or Boat account.
The probes read no job code, logs, artifacts, or cache contents.
They print safe JSON records and remove their unique resources.
Check every cleanup result. `free_proven` requires successful execution
on a GitHub organization whose reported plan is `free`.
No live success is claimed by the local HTTP tests.

### Key Code

- [src/app/mod.rs](src/app/mod.rs) owns `GitHubApp` and its role and
  lifecycle callbacks.
- [src/hooks.rs](src/hooks.rs) verifies and dispatches webhook deliveries.
- [src/auth](src/auth) signs App JWTs and rotates installation tokens.
- [src/rest](src/rest) owns repository and workflow-run REST operations.
- [src/listener](src/listener) holds session locks and delivers durable
  demand before acknowledgement.
- [src/scale_sets](src/scale_sets) adapts the SDK and raw message polling.
- [src/registrations.rs](src/registrations.rs) creates and removes JIT
  registrations with explicit busy outcomes.
- [src/persistence](src/persistence) keeps blocking store calls outside the
  asynchronous services.

### Related Docs

- [Architecture](../../docs/architecture.md) defines hosted service owners.
- [Store interface](../ciproxy-store-interface/README.md) owns persistence.
- [Workspaces](../../docs/protocol/workspaces.md) defines organization joins.
- [Security](../../docs/protocol/security.md) defines permissions and access.
- [Labels](../../docs/protocol/labels.md) defines defaults and scale sets.
- [Job routing](../../docs/protocol/job-routing.md) defines queue waits.
- [App removal](../../docs/protocol/app-removal.md) defines lifecycle work.

[free-groups]: https://github.blog/changelog/2024-10-17-actions-runner-groups-now-available-for-organizations-on-free-plan/
