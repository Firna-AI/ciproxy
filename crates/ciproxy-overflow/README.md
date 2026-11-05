# ciproxy-overflow

Execute Boat overflow actions for the hosted ciproxy control plane.
Use this crate after the scaler selects a Boat runner start.
The crate owns sandbox delivery, encrypted keys, cost facts, and cleanup.
It has no open licence.

## Responsibilities

The scaler decides where a runner is needed and which size it uses.
This crate supplies the account catalog and applies accepted Boat actions.
GitHub still assigns each job to a runner.
A workspace can have several GitHub organizations.
Each delivery keeps its organization, label, and scale-set identity.

The control plane owns Admin authorization, GitHub registrations, the queue,
and the HTTPS callback route.
It supplies `RunnerAuthority` for idle registration removal.
That operation must confirm that no job or waiting demand needs the runner.
The control plane must keep own-machine routing available when Boat fails.

## What This Crate Does

The dependency revision is
`965836198ba65947108cbe5e669524d2249d851a` of `futex-ai/sandbox`.
`BoatFactory` constructs the pinned lifecycle, capacity, and file capabilities.
All capabilities use the same explicit personal or organization wallet.
Account reads supply the verified name, trial state, and next start price.
The service refreshes eligible sizes every five minutes.
The scaler uses their dimensions and exact hourly prices to choose the
cheapest fitting size or the label's explicit largest-size fallback.

Each workspace has one PostgreSQL session lock.
Synchronous queries run on blocking workers.
The service saves the delivery, callback digest, and encrypted input before
the first provider mutation.
The delivery ID becomes Boat's idempotency key.
An uncertain create uses recovery reads forever, including after Boat's
24-hour idempotency window.
It never repeats the create mutation for that delivery.
A definitive refusal ends that delivery and preserves its account restriction.
The next scaler attempt needs a new delivery ID.
Relative and absolute retry times use the later supplied time.
No supplied retry time means no invented retry delay.
Plan refusals exclude that size until a successful catalog refresh.
Billing refusals remain until an account check confirms recovery.
`canStart` does not decide whether create will succeed.

Every sandbox restores the shared snapshot with ciproxy marks.
The pinned adapter sets `noEnv`, disables monitoring and snapshots, and sets
a six-hour lifetime.
A free trial gets two hours.
The runner input contains `AgentToGuest::ConfigureRunner`, the callback URL,
and a sandbox-only callback token.
It contains no Boat key and no machine-cache connection.
The first empty file write initializes the shared adapter's restricted user.
The trusted launcher then creates a private directory and a mode-0600 target.
The shared file capability transfers the registration into that target.

The image needs root for Docker, mounts, and its private namespaces.
The shared process capability deliberately runs a restricted user.
The narrow launcher therefore uses the pinned adapter's trusted controller
API with one fixed image command.
It runs `/opt/ciproxy/rootfs/opt/ciproxy-image/boat-run.sh`.
Secrets travel in the private file, never in that command.
A root-owned per-delivery marker permits at most one guest launch.
Launch recovery reuses that marker and does not rewrite consumed input.
The service records `Writing` before any file transfer.
An unconfirmed or interrupted write enters deletion before any replacement.

An end report derives workspace authority from its token digest.
The store accepts it once for its one live runner.
There is no independent token timeout before the sandbox ends.
The service persists deletion before calling Boat.
Cleanup retries failed deletion and inspects known IDs omitted from inventory.
It preserves running jobs and needed idle runners.
It only deletes identities proven by its durable create records.
Unrecognized inventory entries stay unchanged.

Spend includes all ciproxy sandboxes across the workspace's organizations
and retained key versions.
It excludes other use of the Boat account.
The service saves the documented start-price quote before dispatch.
It reads each sandbox's runtime for explicit UTC calendar-month windows.
It uses the pinned hourly price for the known running interval between the
last observation and a confirmed normal deletion.
It counts a start fee only in its start month.
At the monthly limit, new starts wait until the next UTC month or a limit
increase.
Running jobs and deletion continue.

`Spend` exposes unknown accepted start fees and unmeasured runtime intervals.
For example, a lost delete response can leave the final interval unknown.
`scaler_account` and new starts return `Error::AccountingIncomplete` while
the current month's required cost facts remain incomplete.
They never convert missing costs to zero.
Ordinary measured completion permits another start under the limit.
The store also protects metering from incomplete price intervals.
It corrects trusted provider start times and preserves the last trusted row
when a final interval is unknown.
Boat accounting and relevant savings reads then return a typed incomplete
error, without changing execution or cleanup.

Cloud KMS encrypts account keys and temporary runner input.
Authenticated data binds the workspace, secret identity, and purpose.
The REST cipher verifies CRC32C checksums and the selected key version.
GKE uses `MetadataAccessTokens` with its runtime identity.
The store receives ciphertext and token hashes only.
Key replacement and disconnect retain old ciphertext while a delivery is
unfinished, including uncertain creates.
Successful guest launch clears its sealed registration.
Finished delivery metadata stays for 90 days.
Diagnostics and live commands omit credentials, registrations, and bodies.

## Quick Start

Run from the repository root:

```sh
cargo build --package ciproxy-overflow
cargo test --package ciproxy-overflow
```

Construct `OverflowService` with `OverflowConfig` and
`OverflowDependencies` at the control-plane composition root.
Inject the store as `Arc<dyn OverflowStore>`.
Inject `CloudKmsCipher` over `CloudKmsRest` and `MetadataAccessTokens`.
Inject `BoatFactory`, `SystemClock`, `RandomTokens`, and the caller's
`RunnerAuthority`.
Consume the result as `Arc<dyn Overflow>`.

Call the public boundary in this order:

1. Call `connect` with a key, its new version ID, the wallet, and a positive
   spend limit.
2. Call `view` for the current catalog and spend.
3. Call `OverflowView::scaler_account` with the snapshot's UTC clock origin.
4. Build a consistent scaler snapshot and run its placement decision.
5. Allocate a stable runner ID and a stable delivery ID.
6. Obtain the GitHub registration for the action's organization and scale set.
7. Call `start` with the action and those identities.
8. Include accepted pending deliveries in the next scaler snapshot.
9. Call `resume` after a restart or an uncertain result.
10. Pass authenticated guest exit reports to `report_end`.
11. Call `cleanup` during regular reconciliation and after an outage.

The callback route must pass the Bearer token and the message's runner ID.
It must accept only `GuestToAgent::RunnerExited` messages.
The token grants no other operation.
Map typed Boat failures without stopping jobs on own machines.
Keep the same delivery ID until a result is known.

## Development

Targeted checks need no Boat or GCP credential:

```sh
cargo test --package ciproxy-overflow
DATABASE_URL=postgresql://ciproxy:ciproxy@127.0.0.1:5432/ciproxy_test \
  cargo test --package ciproxy-store-pg --test store_tests overflow::
cargo clippy --package ciproxy-overflow --all-targets -- -D warnings
```

Unit tests use `sandbox-interface` mocks.
Integration tests use the production pinned adapter against a local TLS
server with certificate and host verification enabled.
The store tests use isolated PostgreSQL databases.

### Live commands

These commands contact Boat or Cloud KMS and can incur charges.
Credential arguments are file paths.
Create those files outside Git with mode 0600.
The scripts read paths from environment variables and never print contents.
`--help` runs without credentials or network access.

```sh
export BOAT_API_KEY_FILE=/private/boat-key
export GCP_ACCESS_TOKEN_FILE=/private/gcp-oauth-token
export KMS_KEY_RESOURCE=projects/P/locations/L/keyRings/R/cryptoKeys/K

crates/ciproxy-overflow/scripts/kms-roundtrip.sh
crates/ciproxy-overflow/scripts/boat-live.sh account

mkdir -p .context/ciproxy-mvp/milestone-12/live
crates/ciproxy-overflow/scripts/boat-live.sh create \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json \
  --snapshot SHARED_SNAPSHOT_OR_SHARE_CODE --size large
crates/ciproxy-overflow/scripts/boat-live.sh recover \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
crates/ciproxy-overflow/scripts/boat-live.sh inspect \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
crates/ciproxy-overflow/scripts/boat-live.sh runner \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json \
  --registration-file /private/boat-registration.json
crates/ciproxy-overflow/scripts/boat-live.sh usage \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
crates/ciproxy-overflow/scripts/boat-live.sh delete \
  --state-file .context/ciproxy-mvp/milestone-12/live/boat.json
```

Use `--trial` with `create` on a free trial.
The private registration file uses the
[image's Boat format](../../image/README.md#boat-small-and-default-smoke).
It needs a live JIT registration and an HTTPS callback URL and token.
The runner probe adjusts the internal runner ID to the recorded delivery.
The live state file contains identities and phases, without credentials.
An existing create state always selects recovery reads.
Run `delete` even when a later probe fails.
Use a separate state file for each sandbox.
These probes do not supply the control-plane callback server or a GitHub
registration issuer.
The control-plane runtime owns those integrations.

### Key Code

- [src/api.rs](src/api.rs) defines `Overflow` and its caller contracts.
- [src/service](src/service) owns durable execution and cleanup.
- [src/view.rs](src/view.rs) supplies safe scaler and cost facts.
- [src/boat](src/boat) composes the pinned adapter and fixed image launcher.
- [src/kms](src/kms) implements real Cloud KMS encryption and runtime OAuth.
- [tests/boat](tests/boat) tests the production HTTPS boundaries.
- [examples/boat_live.rs](examples/boat_live.rs) provides live Boat probes.
- [examples/kms_roundtrip.rs](examples/kms_roundtrip.rs) checks real KMS access.

### Related Docs

- [Overflow](../../docs/protocol/overflow.md) defines the required behavior.
- [Labels](../../docs/protocol/labels.md) defines size policy.
- [Job routing](../../docs/protocol/job-routing.md) defines runner placement.
- [Security](../../docs/protocol/security.md#key-custody) defines key custody.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Scaler](../ciproxy-scaler/README.md) owns pure routing decisions.
- [Store interface](../ciproxy-store-interface/README.md) owns persistence.
- [PostgreSQL store](../ciproxy-store-pg/README.md) implements queries.
