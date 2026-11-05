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
Account reads supply the verified name, trial state, and start-fee bound.
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
Create execution and cleanup use one persistent adoption operation.
It saves the proven sandbox identity, phase, and original lifetime bound
before a demand check or another provider call.
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
An absent or archived sandbox gets one final usage read before lifecycle end.
It preserves running jobs and needed idle runners.
It only deletes identities proven by its durable create records.
Unrecognized inventory entries stay unchanged.
Listed or recovered sandboxes with a proven ended state enter cleanup
before a stale demand hint can retain them.
A known sandbox has a saved latest end from a trusted creation time or a
later existence observation and its unchanged provider lifetime.
One `KnownSandbox` value contains both the identity and its bound.
After that bound, failed credentials cannot keep its retired key forever.
Cleanup can finish lifecycle retention while the final bill stays unknown.
An ambiguous create without a known sandbox gets no inferred end.

Spend includes all ciproxy sandboxes across the workspace's organizations
and retained key versions.
It excludes other use of the Boat account.
The service saves the highest published start fee before dispatch.
The existing admission read selects the maximum
`overagePrices.startBrackets[].dollars` from `GET /limits`.
Missing or empty brackets fall back to `nextStartDollars`.
`start_fee_bound_usd` stores this one selected amount.
There is no second quote read or later fee reconciliation.
It reads each sandbox's runtime for explicit UTC calendar-month windows.
It uses the pinned hourly price for the known running interval between the
last observation and a confirmed normal deletion.
It counts a start fee only in its start month.
Measured dollars and the pinned rate define the continuous billing interval.
Its end is separate from the later lifecycle reconciliation time.
An impossible interval clamps to its observation bounds and logs a warning.
Every overflow-to-metering write uses a finite measured interval.
Its constructor clamps the end to the latest provider observation.
Later metering refreshes cannot add runtime beyond that prefix.
`Spend` separates confirmed costs, pending bounds, and expired settled bounds.
The scaler and start admission count all three against the monthly limit.
Each bound uses the requested size's pinned hourly price for the complete
six-hour lifetime, or two hours on a trial, plus the saved start-fee bound.
Measured runtime reduces the remaining runtime bound without counting twice.
A final runtime reading removes the remaining runtime bound.
The saved fee stays counted in its UTC start month after runtime completion
and lifetime expiry, including a free start with zero measured runtime.
Boat reports no actual start fee.
The saved bound never becomes a provider-confirmed start charge.
The existing measured-cost path still requires independent confirmed facts.

A lifetime expires for budget purposes from the saved dispatch time until a
trusted creation reading supplies the real start time.
Later running evidence prevents stale budget settlement, including when a
monthly cost read fails. A total runtime reading survives that failure.
A total within one UTC month supplies exact confirmed spend for that month.
An unallocated total across months supplies a conservative budget bound,
after subtraction of measured costs from other months.
That bound supplies no monthly measured charge or metering interval.
Each bound splits runtime at UTC month boundaries
and counts the fee only in its start month.
At budget expiry, the remaining bound moves to settled spend.
That transition changes no provider identity, lifecycle end, or key custody.
An uncertain create still requires recovery and retains its old key.
Later running evidence that contradicts a known lifetime prevents cleanup
from treating that old deadline as proof that the sandbox ended.
A later real cost replaces the estimate, including costs above its bound.

New starts continue while the limit covers existing costs and the proposed
start's bound in each UTC month its lifetime touches.
A missing safe fee bound returns `Error::AccountingIncomplete`.
`Spend::unbounded_starts` identifies the affected deliveries.
A new start with neither fee source waits before provider dispatch.
At the monthly limit, new starts wait until there is room.
Running jobs and deletion continue.

Missing or failed usage reads invalidate stale runtime completeness.
Known costs and saved account metadata remain readable after key rejection,
including records that still reference an old key.
Ordinary measured completion permits another start under the limit.
The store also protects metering from incomplete price intervals.
It publishes a bounded known billing prefix when a final interval is unknown.
The metering refresh can then account known costs and purge expired totals.
Relevant savings reads still return a typed incomplete error.
Ranges after a confirmed lifecycle end do not include that missing interval.

Cloud KMS encrypts account keys and temporary runner input.
Authenticated data binds the workspace, secret identity, and purpose.
The REST cipher verifies CRC32C checksums and the selected key version.
GKE uses `MetadataAccessTokens` with its runtime identity.
The store receives ciphertext and token hashes only.
Key replacement and disconnect retain old ciphertext while a delivery is
unfinished, including uncertain creates.
Retained keys expose only inspection, recovery reads, usage reads, and deletion.
Registration delivery and guest launch require the active accepted key.
They also require overflow to remain enabled in the workspace.
The service checks these facts after key decryption at each operation.
A retired delivery uses a read-only launch-marker check before cleanup.
A confirmed launch keeps its runner, including after a lost launch response.
An unavailable check keeps the delivery and its recovery key for another pass.
Successful guest launch clears its sealed registration.
Finished delivery metadata stays for 90 days.
Diagnostics and live commands omit credentials, registrations, and bodies.

## Quick Start

Run from the repository root:

```sh
cargo build --package ciproxy-overflow
export DATABASE_URL=postgresql://ciproxy:ciproxy@127.0.0.1:5432/ciproxy_test
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
export DATABASE_URL=postgresql://ciproxy:ciproxy@127.0.0.1:5432/ciproxy_test
cargo test --package ciproxy-overflow --test service_tests
cargo test --package ciproxy-overflow --test boat_tests
cargo test --package ciproxy-store-pg --test store_tests overflow::
cargo clippy --package ciproxy-overflow --all-targets -- -D warnings
```

Service integration tests require PostgreSQL 15 and `DATABASE_URL`.
Each test creates and removes its own database through `PgStoreFactory`.
The database user needs CREATEDB or superuser rights.
The supplied database serves only as the maintenance connection.
Provider, clock, and cipher mocks supply external facts.
A forwarding recorder captures successful store writes without supplying answers.
Callback tests use real token lookup and one-use claims across two workspaces.
Boat boundary tests use the production adapter against a local TLS server.
They keep certificate and host verification enabled.
The cipher unit tests use a mocked KMS transport.

### Live commands

The [live-probe guide](LIVE.md) gives credential setup and probe commands.
These commands contact Boat or Cloud KMS and can incur charges.

### Key Code

- [src/api.rs](src/api.rs) defines `Overflow` and its caller contracts.
- [src/service](src/service) owns durable execution and cleanup.
- [src/maintenance.rs](src/maintenance.rs) restricts retained-key capabilities.
- [src/view.rs](src/view.rs) supplies safe scaler and cost facts.
- [src/boat](src/boat) composes the pinned adapter and fixed image launcher.
- [src/kms](src/kms) implements real Cloud KMS encryption and runtime OAuth.
- [tests/boat](tests/boat) tests the production HTTPS boundaries.
- [tests/service](tests/service) tests public service behavior over real PgStore.
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
