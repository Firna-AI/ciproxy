# ciproxy-control-plane

The hosted ciproxy runtime connects GitHub scale sets to customer machines.
When those machines are busy, it starts runners on Boat.
Its executable serves the dashboard API, the machine gateway, GitHub
webhooks, and Boat exit reports on one HTTP host. It also runs the scaler
and the GitHub workers. This crate has no open licence.

## Responsibilities

Own the executable and the single HTTP host.
Keep GitHub App credentials in the hosted process.
Persist runner reservations before one-use registrations leave the process.
Preserve running jobs across busy races, offline machines, and restarts.
Compose the Boat overflow service and keep accepted Boat supply across
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

Serving reads its settings from the environment and its secrets from files.
A missing setting stops the process with exit status 1 and names the setting.
The process then signs and fetches the pinned published manifest.
It reads `downloads.json` from the same release directory.
These files supply the exact root bytes, image ID, archive hash, and size.
The process then connects to PostgreSQL and applies the embedded migrations.
It binds `HTTP_BIND` only after these steps succeed.

Startup builds the dashboard access service first. It owns sign-in,
workspace roles, and the App install authority. The machine gateway and the
GitHub App service use it. The dashboard API gets the same gateway, App
service, scale-set client, and App signer.
`MachineRuntime::new` requires an `InstallationApi`. Composition passes the
App service's App-JWT installation client, which take-out acceptance uses.
The API reads machine capacity from the last report of each machine: its
slots, its job disk minus active Docker leases, and the root bytes of its
selected image.
A machine without a report has zero slots and unknown disk capacity.
API maintenance runs at startup and then every minute. A failed run logs a
warning and runs again.

This one release supplies the image facts of the process. The agents'
image download, the API's image disk size, the scaler fallback, and Boat
sizes come from it. Each machine's selected local image still sizes the new
roots of that machine.
[Runtime setup](../../deploy/runtime.md#turn-on-the-runtime) says which
release file gives each field.

### Routes

| Route                       | Use                                                       |
| --------------------------- | --------------------------------------------------------- |
| `/agent/`                   | The agent WebSocket.                                      |
| `/hooks/github`             | GitHub App webhooks, up to GitHub's 25 MiB payload limit. |
| `/hooks/boat/runner-exit`   | Boat sandbox exit reports, while Boat is configured.      |
| `/api`                      | The dashboard API.                                        |
| `/api/auth/github/callback` | The GitHub sign-in callback.                              |
| `/api/github/setup`         | The GitHub App setup URL.                                 |
| `/api/health/live`          | HTTP 200 while the process serves.                        |
| `/api/health/ready`         | HTTP 200 after the first reconciliation pass, else 503.   |

Register `PUBLIC_URL/hooks/github` as the App webhook URL.
[The API guide](../ciproxy-api/README.md#quick-start) gives the other App
registration settings. No route has an alias.
The API owns sign-in, membership, setup, and workspace management.
An Admin can add an organization to an existing workspace. The original
requester proves installation access and chooses an eligible workspace.
Local billing or Admin holds keep that organization inactive.

SIGTERM or Ctrl-C during startup stops the process with status 0 before it
serves. After startup, the signal makes readiness return HTTP 503 at once.
HTTP requests, listeners, and maintenance then get up to 240 seconds to
finish. Other runtime work then gets up to five more seconds.

### Routing and runners

The [runner guide](runners.md) gives routing, reservations, report
confirmation, stop replay, retirement authority, and the removal and
take-out policies.
Take-out checks saved retirement for both own-machine and Boat reservations
before the API removes GitHub access.

### Boat overflow

The optional `BOAT_SHARED_SNAPSHOT` setting turns on the Boat service.
Without it, own machines still run jobs, and Boat stays off.
The runtime, the dashboard API, and the exit callback share one overflow
service. The [overflow guide](overflow.md) gives the configuration, the
runtime guarantees, the exit callback, and the live smoke checks.
After an owner removes the App, launched sandboxes run until their runner
exits or their saved lifetime ends. The runtime holds the Boat key for
their cleanup, also after the last organization leaves.

### GitHub workers

Each reconciliation pass also starts a scale-set listener for each label of
an admitted organization with an active App installation. It stops the
listeners of
bindings that are gone. A deleted label keeps its listener while it has
unfinished jobs or a pending delivery. That listener acquires no new jobs.
An inactive organization gets no new runner or warm capacity. Its idle runners
use GitHub-first retirement. Received jobs keep their trusted preparation and
completion listeners through the final acknowledgement. Active sibling
organizations keep their runners and demand.

GitHub does not automatically retry failed webhooks.
A separate worker scans all pages of retained App delivery history each
minute. It groups retries by delivery GUID and uses the latest attempt.
It requests redelivery only for relevant events whose latest attempt failed.
The App's durable lifecycle outbox handles failures after local persistence.
A separate startup and five-minute poll refreshes current installation
authority and repository snapshots. It needs no optional event subscriptions.

One supervised accounting worker does job timing repair, the metering
refresh, and the raw job purge, in that order.
The ten-second tick and listener job delivery only signal it.
Pending signals merge into one pass, and passes never overlap.
Readiness, runner operations, and workspace leases never wait for it.
An error or a panic in one workspace does not stop the next workspace.
The supervisor restarts the worker when it ends. Shutdown cancels it with
the other short-grace runtime work.

The runtime installs the mandatory gateway log guard.
`RUST_LOG` changes other verbosity without exposing serialized frames.

The [runner guide](runners.md#removal-and-take-out) gives the removal and
take-out policies.
The user selected spare-disk reservations for sticky Docker attachments.
Active reservations count against root admission until those jobs finish.
Live KVM and authenticated GitHub checks remain operator work.

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
`IMAGE_URL_PREFIX`, `BOAT_KMS_KEY`, `CDN_SIGNING_KEY_NAME`, and
`SECRETS_DIRECTORY`.
Deploy fills `controlPlane.env` with `GITHUB_APP_ID`,
`GITHUB_OAUTH_CLIENT_ID`, `GITHUB_APP_SLUG`, and the immutable
`JOB_IMAGE_MANIFEST_PATH`. It reads them from the GitHub environment
variables `CIPROXY_APP_ID`, `CIPROXY_APP_CLIENT_ID`, `CIPROXY_APP_SLUG`,
and `JOB_IMAGE_MANIFEST_PATH`.
`DOWNLOAD_URL` selects the environment's release host.
`BUILD_ID` identifies the built commit.
`BOAT_SHARED_SNAPSHOT` is optional. The
[overflow guide](overflow.md#configuration) gives its rules.
The runtime uses mounted private key, webhook, OAuth, session, and CDN keys.
Artifact delivery gets a fresh Cloud CDN signature each time.
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
They mock GitHub and gateway side effects.
The PostgreSQL role needs CREATEDB rights.
The [live smoke checks](overflow.md#live-smoke-checks) need live Boat,
GitHub, and database access, so they stay operator work.

### Key Code

- [src/cli.rs](src/cli.rs) selects `serve` or `diagnose`.
- [src/application.rs](src/application.rs) runs startup, the HTTP server,
  maintenance, and bounded shutdown.
- [src/startup_api.rs](src/startup_api.rs) builds the access service, the
  hosted services that use it, and then the dashboard API and maintenance.
- [src/capacity.rs](src/capacity.rs) gives the API each machine's capacity.
- [src/composition.rs](src/composition.rs) constructs shared hosted adapters.
- [src/config.rs](src/config.rs) resolves runtime and image configuration.
- [src/image_release.rs](src/image_release.rs) fetches the pinned release.
- [Gateway logging](../ciproxy-gateway/src/logging.rs) keeps frame guards
  mandatory while it preserves unrelated selected diagnostics.
- [src/listeners.rs](src/listeners.rs) supervises reconciliation, listener
  discovery, and readiness.
- [src/runtime.rs](src/runtime.rs) owns workspace execution reconciliation.
- [src/actions.rs](src/actions.rs) reserves starts and saves runner states.
- [src/reconciliation.rs](src/reconciliation.rs) applies full reports to
  reservations, with the confirmation and absence rules.
- [src/retirement.rs](src/retirement.rs) saves stops and delivers them, live
  and in recovery.
- [src/lifecycle.rs](src/lifecycle.rs) applies the retirement policy of each
  App removal cause.
- [src/takeout.rs](src/takeout.rs) accepts a take-out only when each
  reservation of its organization has an answer.
- [src/reservations.rs](src/reservations.rs) gives one saved retirement
  result for machine commands, Boat registrations, deliveries, and runners.
- [src/boat_retirement.rs](src/boat_retirement.rs) shares durable Boat
  registration cleanup between take-out and ordinary recovery.
- [src/accounting.rs](src/accounting.rs) runs coalesced accounting passes
  outside runner operations.
- [src/events.rs](src/events.rs) binds authenticated observations to runners.
- [src/snapshot.rs](src/snapshot.rs) supplies the pure scaler's inputs.
- [src/warm.rs](src/warm.rs) allocates warm runners from recent jobs.
- [src/slots.rs](src/slots.rs) selects exact logical slot sets.
- [src/http.rs](src/http.rs) mounts shared HTTP routes and health checks.
- [src/overflow_composition.rs](src/overflow_composition.rs) builds the
  Boat service and its registration authority when Boat is configured.
- [src/boat_runtime.rs](src/boat_runtime.rs) starts and recovers Boat
  deliveries and their GitHub registrations.
- [src/boat_snapshot.rs](src/boat_snapshot.rs) adds accepted Boat supply to
  the scaler snapshot.
- [src/boat_authority.rs](src/boat_authority.rs) defines
  `BoatRunnerAuthority`. It retires Boat registrations GitHub-first for the
  overflow service and for runtime recovery. It preserves the launched
  sandboxes of an organization whose owner removed the App.
- [src/boat_jobs.rs](src/boat_jobs.rs) binds trusted GitHub job facts to
  Boat runners.
- [src/boat_callback.rs](src/boat_callback.rs) mounts the sandbox exit
  callback.
- [scripts/overflow-smoke.py](scripts/overflow-smoke.py) runs the live Boat
  checks.
- [tests/runtime](tests/runtime) checks durable runtime behavior.

### Related Docs

- [Architecture](../../docs/architecture.md) describes the own-machine and
  Boat flows.
- [Overflow](../../docs/protocol/overflow.md) defines Boat runners, cleanup,
  and the spend limit.
- [ciproxy-overflow](../ciproxy-overflow/README.md) executes Boat deliveries.
- [Agent](../../docs/protocol/agent.md) defines machine and VM lifecycle.
- [Routing](../../docs/protocol/job-routing.md) defines fit and warm runners.
- [Cache](../../docs/protocol/cache.md) defines trusted cache and Docker use.
- [Dashboard API](../ciproxy-api/README.md) defines the `/api` routes.
- [GitHub integration](../ciproxy-github/README.md) defines App boundaries.
- [Runtime setup](../../deploy/runtime.md) defines the deployed interface.
