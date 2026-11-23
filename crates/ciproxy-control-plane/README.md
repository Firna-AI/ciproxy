# ciproxy-control-plane

The hosted ciproxy runtime connects GitHub scale sets to customer machines.
It also starts runners on configured overflow providers.
Its executable serves the dashboard API, the machine gateway, GitHub
webhooks, and sandbox exit reports on one HTTP host. It also runs the scaler
and the GitHub workers. This crate has no open licence.

## Responsibilities

Own the executable and the single HTTP host.
Keep GitHub App credentials in the hosted process.
Persist runner reservations before one-use registrations leave the process.
Preserve running jobs across busy races, offline machines, and restarts.
Keep slow GitHub and provider requests from delaying other workspaces.
Compose the provider registry and keep accepted overflow supply across
restarts.

## What This Crate Does

### Commands and startup

With no command, or with `serve`, the executable runs the hosted service.
The container image starts it with no arguments.
`diagnose` logs the protocol version and the image root bytes of the shared
root sizing, then exits. It reads no runtime setting or secret file.
`diagnose --image-root-bytes` or `CIPROXY_IMAGE_ROOT_BYTES` changes that
diagnostic value. Only `diagnose` accepts the flag.
Serving ignores `CIPROXY_IMAGE_ROOT_BYTES`.

The [startup guide](startup.md) gives the startup order, the hosted services
that startup builds, with billing and the Stripe return URLs, and the image
facts of the process.

### Routes

| Route                         | Use                                                       |
| ----------------------------- | --------------------------------------------------------- |
| `/agent/`                     | The agent WebSocket.                                      |
| `/agent/cache-metadata`       | Cache entry metadata for a job on the calling machine.    |
| `/hooks/github`               | GitHub App webhooks, up to GitHub's 25 MiB payload limit. |
| `/hooks/stripe`               | Raw signed Stripe billing webhooks.                       |
| `/hooks/overflow/runner-exit` | Sandbox exit reports while overflow is configured.        |
| `/api`                        | The dashboard API.                                        |
| `/api/auth/github/callback`   | The GitHub sign-in callback.                              |
| `/api/github/setup`           | The GitHub App setup URL.                                 |
| `/api/health/live`            | HTTP 200 while the process serves.                        |
| `/api/health/ready`           | HTTP 200 after the first reconciliation pass, else 503.   |

Register `PUBLIC_URL/hooks/github` as the App webhook URL.
[The API guide](../ciproxy-api/README.md#quick-start) gives the other App
registration settings. The [exit callback](overflow.md#exit-callback)
also serves deliveries that saved the previous path.
The API owns sign-in, membership, setup, and workspace management.
It also serves Jobs, Savings, Plan and usage, Plans, Checkout, and the shared
page banner. Stripe deliveries use the host root and need no browser cookie.
An Admin can add an organization to an existing workspace. The original
requester proves installation access and chooses an eligible workspace.
Local billing or Admin holds keep that organization inactive.

SIGTERM or Ctrl-C during startup stops the process with status 0 before it
serves. After startup, the signal withdraws readiness and stops new
connections at once. A new readiness probe is then normally refused. A probe
on a connection that is already open can see HTTP 503.
HTTP requests, listeners, and maintenance then get up to 240 seconds to
finish. Other runtime work then gets up to five more seconds.

### Routing and runners

The [runner guide](runners.md) gives routing, reservations, the agent event
queue, job ID binding and preparation, remote requests, report confirmation,
stop replay, retirement authority, and the removal and take-out policies.
Take-out checks saved retirement for both own-machine and overflow reservations
before the API removes GitHub access.
Routing reads `MachineSnapshot::state` from the store interface. Only `Ready`
sets a machine's scaler connection flag. Resource and drain checks still
control admission after that flag.

### Overflow providers

The optional `BOAT_SHARED_SNAPSHOT` setting adds Boat to the registry.
Without it, own machines still run jobs, and Boat stays off.
The runtime, the dashboard API, and the exit callback share one overflow
service. The [overflow guide](overflow.md) gives the configuration, the
runtime guarantees, and the exit callback. The
[live smoke guide](overflow-smoke.md) gives the live smoke checks.
After an owner removes the App, launched sandboxes run until their runner
exits or their saved lifetime ends. The runtime holds each provider key for
its sandbox cleanup, also after the last organization leaves.

### GitHub workers

The [GitHub worker guide](github-workers.md) gives the scale-set listeners,
webhook redelivery, the accounting worker, and the log guard.

## Quick Start

Run commands from the repository root:

```sh
cargo run --package ciproxy-control-plane -- --help
cargo run --package ciproxy-control-plane -- diagnose
cargo run --package ciproxy-control-plane -- diagnose --image-root-bytes 63000000000
```

`--help` prints:

```text
Usage: ciproxy-control-plane [COMMAND]

Commands:
  serve     Serve the dashboard API, agent gateway, and hosted workers
  diagnose  Report diagnostic image sizing without starting the service
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

`cargo run --package ciproxy-control-plane -- serve` needs the deployment
settings below. Without them, it exits with status 1 and logs
`missing setting DATABASE_URL`.

The chart sets `HTTP_BIND`, `DATABASE_URL`, `PUBLIC_URL`,
`IMAGE_URL_PREFIX`, `OVERFLOW_KMS_KEY`, `CDN_SIGNING_KEY_NAME`, and
`SECRETS_DIRECTORY`.
Deploy fills `controlPlane.env` with `GITHUB_APP_ID`,
`GITHUB_OAUTH_CLIENT_ID`, `GITHUB_APP_SLUG`, and the immutable
`JOB_IMAGE_MANIFEST_PATH`. It reads them from the GitHub environment
variables `CIPROXY_APP_ID`, `CIPROXY_APP_CLIENT_ID`, `CIPROXY_APP_SLUG`,
and `JOB_IMAGE_MANIFEST_PATH`.
`DOWNLOAD_URL` selects the environment's release host.
`BUILD_ID` identifies the built commit.
`BOAT_SHARED_SNAPSHOT` is optional. The
[overflow guide](overflow.md#configuration) gives its rules. Deploy reads it
from a Secret Manager secret, as
[Boat setup](../../deploy/runtime.md#boat-shared-snapshot) says.
The optional settings `MACOS_IMAGE_URL_PREFIX`, `MACOS_IMAGE_MANIFEST_PATH`,
and `R2_IMAGES_KEY` turn on the macOS image.
[macOS image setup](../../deploy/runtime.md#macos-images) gives where each
one comes from, when a new key takes effect, and when the runtime offers
the image.
The runtime uses mounted private key, webhook, OAuth, session, and CDN keys.
Billing reads `stripe-secret-key` and `stripe-webhook-secret` from the same
secret directory. Startup constructs the blocking Stripe client on a blocking
worker.
The trusted configuration supplies Linux x64 comparison price lists dated
7 November 2026. Blacksmith rates come from its
[pricing page](https://www.blacksmith.sh/pricing), with dimensions from its
[instance guide](https://docs.blacksmith.sh/blacksmith-runners/overview).
GitHub rates come from its [runner prices][github-prices].
Its [larger runner guide][github-sizes] gives the dimensions.

[github-prices]: https://docs.github.com/en/billing/reference/actions-runner-pricing
[github-sizes]: https://docs.github.com/en/actions/reference/runners/larger-runners
Linux artifact delivery gets a fresh Cloud CDN signature each time.
macOS artifact delivery gets a fresh presigned R2 URL each time.
`IMAGE_URL_PREFIX` selects the existing CDN /images origin.
Signed image clients use HTTP/1 explicitly. This keeps their transport
stable under Cargo feature unification and matches the real TRACE proof.
The HTTP client keeps other diagnostics and the mandatory frame guard.
Keep credentials in mounted private files.
See [runtime deployment](../../deploy/runtime.md) for secret file names.
See [container packaging](container.md) for build and non-root execution.

Publish a checked image release with an explicit credential file:

```sh
export CIPROXY_GCLOUD_CREDENTIALS_FILE=/PRIVATE/google-credentials.json
python3 crates/ciproxy-control-plane/scripts/publish-job-image.py \
  .context/ciproxy-image-build/artifacts IMAGE_BUCKET RELEASE
```

This calls the image publisher with `gs://IMAGE_BUCKET/images/RELEASE`.
Object generation preconditions prevent it from replacing existing artifacts.
Use a fresh release directory after a partial or failed upload.
The command prints no manifest selector when an upload fails.
The edge keeps the `/images/` prefix when it reads bucket objects.
Set `JOB_IMAGE_MANIFEST_PATH` to the printed relative manifest path.
This command does not deploy the control plane.

## Development

```sh
export DATABASE_URL=postgresql://ciproxy@127.0.0.1:5432/ciproxy_test
cargo test --package ciproxy-control-plane
cargo fmt --all -- --check
cargo clippy --package ciproxy-control-plane --all-targets -- -D warnings
```

Runtime integration tests create and remove isolated PostgreSQL databases.
They mock GitHub and gateway side effects. They await
`MachineRuntime::quiesce`, or `quiesce_lifecycle` for a lifecycle event,
before they check the results of queued and remote work.
The PostgreSQL role needs CREATEDB rights.
Executable signal tests need Linux procfs and the `openssl` command.
The [live smoke checks](overflow-smoke.md) need live Boat,
GitHub, and database access, so they stay operator work.

### Key Code

- [src/cli.rs](src/cli.rs) selects `serve` or `diagnose`.
- [src/application.rs](src/application.rs) runs startup, the HTTP server,
  maintenance, and bounded shutdown.
- [src/startup_api.rs](src/startup_api.rs) builds the access service, the
  hosted services that use it, and then the dashboard API and maintenance.
- [src/startup_billing.rs](src/startup_billing.rs) loads billing secrets,
  constructs the Stripe client outside the async runtime worker, and supplies
  `HostedBillingAlerts`.
- [src/pages.rs](src/pages.rs) shares billing, paid previews, metering,
  overflow, and image sizing for the current dashboard API.
- [src/billing.rs](src/billing.rs) retries durable App-removal cancellations
  and pending payment work.
- [src/comparison_prices.rs](src/comparison_prices.rs) holds the dated USD
  comparison catalogs supplied by trusted process configuration.
- [src/capacity.rs](src/capacity.rs) gives the API each machine's capacity.
- [src/composition.rs](src/composition.rs) constructs shared hosted adapters.
- [src/config.rs](src/config.rs) resolves runtime and image configuration.
- [src/image_release.rs](src/image_release.rs) fetches the pinned release
  of one platform with signed reads.
- [src/macos_release.rs](src/macos_release.rs) reads the optional macOS
  settings and release, and turns macOS images off when they are missing.
- [src/image_signing.rs](src/image_signing.rs) signs Cloud CDN URLs and
  gives each platform's release its signer.
- [src/presigned_signing.rs](src/presigned_signing.rs) signs S3 presigned
  GET URLs with AWS Signature Version 4.
- [Gateway logging](../ciproxy-gateway/src/logging.rs) keeps frame guards
  mandatory while it preserves unrelated selected diagnostics.
- [src/listeners.rs](src/listeners.rs) supervises reconciliation, listener
  discovery, readiness, and the stop of runtime tasks.
- [src/runtime.rs](src/runtime.rs) defines `OwnMachines` and `MachineRuntime`.
- [src/runtime_entrypoints.rs](src/runtime_entrypoints.rs) runs the tick, job,
  agent, and lifecycle entry points as short saved-state steps.
- [src/runtime_reconciliation.rs](src/runtime_reconciliation.rs) runs the
  short leased workspace pass and the pass after a provider refresh.
- [src/runtime_inbox.rs](src/runtime_inbox.rs) converts agent observations
  to compact values and handles them with machine removals in arrival order.
- [src/runtime_payloads.rs](src/runtime_payloads.rs) retains exact Worker
  IDs under the shared byte budget, including events still being handled.
- [src/runtime_tasks.rs](src/runtime_tasks.rs) owns and joins one GitHub or
  overflow task for each key. `OwnMachines::shutdown` stops them.
- [src/remote_results.rs](src/remote_results.rs) saves remote results through
  the guarded store writes.
- [src/hosted_events.rs](src/hosted_events.rs) defines `Relay`. Its gateway
  callbacks queue events and return at once.
- [src/actions.rs](src/actions.rs) routes scaler actions and saves runner
  states.
- [src/own_start.rs](src/own_start.rs) reserves an own-machine start before
  its GitHub registration task.
- [src/reconciliation.rs](src/reconciliation.rs) applies full reports to
  reservations, with the confirmation and absence rules.
- [src/retirement.rs](src/retirement.rs) saves stops and delivers them, live
  and in recovery.
- [src/lifecycle.rs](src/lifecycle.rs) applies the retirement policy of each
  App removal cause.
- [src/takeout.rs](src/takeout.rs) accepts a take-out only when each
  reservation of its organization has an answer.
- [src/lifecycle_completion.rs](src/lifecycle_completion.rs) defines
  `takeout_accepted`, which the callback and `quiesce_lifecycle` share.
- [src/reservations.rs](src/reservations.rs) gives one saved retirement
  result for machine commands, overflow registrations, and deliveries.
- [src/overflow_retirement.rs](src/overflow_retirement.rs) shares overflow
  registration cleanup between take-out and ordinary recovery.
- [src/accounting.rs](src/accounting.rs) runs coalesced accounting passes
  outside runner operations.
- [src/events.rs](src/events.rs) binds authenticated observations to runners.
- [src/assignment.rs](src/assignment.rs) defines `Assignment`, which job
  routing, job preparation, and the cache metadata route use to find a job's
  exact runner. `Assignment::job_ids` gives the pair for job preparation.
- [src/job_binding.rs](src/job_binding.rs) saves the Worker job ID that an
  agent `JobStarted` event or a full report gives for its exact runner.
- [src/preparation.rs](src/preparation.rs) starts facts requests with one
  job, its saved guard, Worker and scale-set IDs, machine identities, and the
  computed Docker budget.
- [src/cache_metadata/](src/cache_metadata/mod.rs) checks the machine
  credential, the workspace, and the saved Worker job binding, then lists
  the job's GitHub cache entries.
- [src/snapshot.rs](src/snapshot.rs) builds the pure scaler's inputs,
  including shared machine state, saved overflow supply, and each label's
  order of places.
- [src/docker_budget.rs](src/docker_budget.rs) budgets for labels that list
  own machines and for runner disk allocations.
- [src/warm.rs](src/warm.rs) allocates warm runners from recent jobs.
- [src/slots.rs](src/slots.rs) selects exact logical slot sets.
- [src/http.rs](src/http.rs) mounts shared HTTP routes and health checks.
- [src/overflow_composition.rs](src/overflow_composition.rs) builds the
  provider registry, shared overflow service, and registration authority.
- [src/overflow_runtime.rs](src/overflow_runtime.rs) dispatches cleanup,
  refresh, and recovery for overflow.
- [src/overflow_start.rs](src/overflow_start.rs) runs a provider start after
  GitHub registration.
- [src/overflow_snapshot.rs](src/overflow_snapshot.rs) projects provider
  accounts and saved sandbox supply into the scaler snapshot.
- [src/overflow_authority.rs](src/overflow_authority.rs) defines
  `OverflowRunnerAuthority`. It retires registrations GitHub-first for the
  overflow service and for runtime recovery. It asks GitHub again about a
  saved `Busy` runner without a started job. It preserves the launched
  sandboxes of an organization whose owner removed the App.
- [src/overflow_jobs.rs](src/overflow_jobs.rs) binds trusted GitHub job facts to
  overflow runners.
- [src/overflow_callback.rs](src/overflow_callback.rs) mounts the sandbox exit
  callback.
- [scripts/overflow-smoke.py](scripts/overflow-smoke.py) runs the live Boat
  checks.
- [tests/runtime](tests/runtime) checks durable runtime behavior.
- [tests/dashboard](tests/dashboard) checks the production page composition
  through a bound HTTP server and real PostgreSQL, with a local Stripe peer.

### Related Docs

- [Architecture](../../docs/architecture.md) describes own-machine and
  overflow flows.
- [Overflow](../../docs/protocol/overflow.md) defines provider accounts and
  overflow runners.
- [ciproxy-overflow](../ciproxy-overflow/README.md) executes provider deliveries.
- [Agent](../../docs/protocol/agent.md) defines machine and VM lifecycle.
- [Routing](../../docs/protocol/job-routing.md) defines fit and warm runners.
- [Cache](../../docs/protocol/cache.md) defines cache trust and restore order.
- [Docker disks](../../docs/protocol/docker-disks.md) defines Docker disk use.
- [Dashboard API](../ciproxy-api/README.md) defines the `/api` routes.
- [GitHub integration](../ciproxy-github/README.md) defines App boundaries.
- [Runtime setup](../../deploy/runtime.md) defines the deployed interface.
- [Agent images](../../docs/protocol/agent-images.md#the-macos-image)
  defines how ciproxy serves the macOS image.
