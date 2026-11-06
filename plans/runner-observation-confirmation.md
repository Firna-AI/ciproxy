# Runner observation confirmation

Status: Active. On branch `conductor/runner-observation-confirmation`, no
pull request yet. The pull request that completes the work deletes this
plan, as the `plans` skill says.

This change applies findings 1 and 3 of the second owner review of pull
request #31. It starts from the primary branch
`calummoore/ci-proxy-load-balancer` at `5bfb46f`, after #31 merged. A lost
start result can leave an own-machine runner `Busy` with no first
confirmation, so no later full report releases its slots and disk. One typed
observation now saves the state, the trusted GitHub ID, and the first
confirmation in one transaction, for each positive observation. Listener
start also stops reading PostgreSQL on a Tokio worker.

The change adds no serving shutdown test, no async store handle, no repair
of older rows, and no protocol change. It does not edit `runtime.rs`,
`snapshot.rs`, `lifecycle.rs`, or `composition.rs` of the control plane,
because the organization-add branch of pull request #32 changes them.

User decisions, 6 November 2026: the user chose to merge pull request #31
first, and to fix findings 1 and 3 at once in this separate branch. The user
approved 1B and 3B now, and 2A after the merge. Fresh read-only checks then
confirmed the three findings and added options with new letters.

Design calls, my calls, 6 November 2026:

- **Split of finding 3:** apply B now, and record its wider fix F as a
  follow-up item. Reason: F reaches files of the organization-add branch.
- **Older rows:** no repair of unconfirmed `Busy` rows from before this
  change. Reason: ciproxy is not in production, and no such row is known.
  Option D of decision 1 is the answer if one appears.
- **Evidence time:** a full report confirms at its receipt time. Agent and
  GitHub observations confirm at the control-plane clock time, never at a
  GitHub job time. Reason: the agent builds each report just before it sends
  it, and the VM exists before GitHub can report its job.
- **Full-report writes:** a report that lists a runner writes only when the
  state changes or the first confirmation is missing. It confirms before the
  configuration-loss retirement and each early return. Reason: a pass does
  not rewrite each runner, and a runner that it retires keeps its evidence.

Contract owners:

- [Routing](../docs/protocol/job-routing.md#own-machine-offline): a machine
  shows its slots again when it is back. The fix restores this requirement.
- The [runner guide](../crates/ciproxy-control-plane/runners.md) and the
  [control-plane README](../crates/ciproxy-control-plane/README.md).
- The READMEs of the
  [store interface](../crates/ciproxy-store-interface/README.md), the
  [PostgreSQL store](../crates/ciproxy-store-pg/README.md), and the
  [schema](../crates/ciproxy-schema/README.md).

Replaced ways:

- Separate own-runner state and confirmation writes → one typed
  observation, `OwnRunnerStore::observe_own_runner` with an
  `OwnRunnerObservation` of state, GitHub ID, and `observed_at`. It filters
  only by workspace, organization, and runner, and it keeps an earlier
  `confirmed_at`. Consumers: the full-report presence path of
  `reconciliation.rs`; the agent ready, job start, and exit events, the
  GitHub job start, and the busy answer for a removed machine in
  `events.rs`; the busy answer in `retirement.rs`; the `confirm_own_runner`
  test in `own_routing_tests.rs` of `ciproxy-store-pg`; and the confirmation
  facts of the runner guide and the store READMEs. Moved in Milestone 2,
  with the guide and the schema README in Milestone 1. Deleted in Milestone
  2: `OwnRunnerStore::confirm_own_runner` with its query, and the `Ready`
  and `Busy` writes of the private `MachineRuntime::save_state`. It then
  takes only `UnobservedRunnerUpdate::Registered` or
  `UnobservedRunnerUpdate::Stopped`. The generic `RunnerStore::update_runner`
  stays for other store areas and test setup.
- The organization read at listener start → a private discovered target in
  `listeners.rs`. It holds the unchanged public `ListenerTarget` and the
  `installation_id` that discovery already reads on a blocking worker.
  Consumer: the listener start of `Hosted::supervise`. Moved and deleted in
  Milestone 2, with no adapter or wrapper.

## Decisions

Terms:

- **Reservation:** the slots and disk bytes that the store holds for one
  runner.
- **Full report:** the agent's `Hello` or `Report` message with every VM it
  owns. The store keeps the last one with its receipt time.
- **Confirmation:** the `confirmed_at` time of an own runner. A later full
  report that omits the runner releases it only when it arrived after it.

### 1. Lost start results can hold capacity forever

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
- **Source:** Review, finding 1 of the second owner review of #31.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/reconciliation.rs:114`,
  `events.rs:159`, and `retirement.rs:74`, at `5bfb46f`.
- **Background:** The control plane saves a reservation before it sends
  `StartRunner`. Only a current command result or a full report that lists
  the runner saves its confirmation.
- **Problem:** The agent can apply the start and lose its connection before
  its result arrives. GitHub then reports the runner busy, and the runtime
  saves `Busy` with no confirmation. If the job ends before the agent
  reconnects, no absence rule acts on that runner.
- **Impact if no action:** The reservation stays until someone edits the
  database. A machine with room for one job stops taking jobs.
- **Root question:** Which observations prove that a runner's VM existed,
  and how does each one save the time that lets a later full report release
  the reservation?
- **Options:**
  - **A.** Save the first confirmation before each trusted ready, started,
    or GitHub busy update, as a separate write. Effort: Small.
  - **B.** One typed observation saves the state, the trusted GitHub ID, and
    the first confirmation in one transaction. Every positive observation
    uses it. Effort: Medium.
  - **C.** The agent keeps lifecycle messages until the control plane
    acknowledges them, and replays them. It needs new protocol messages.
    Effort: Large.
  - **D.** Extend the 60-second GitHub-first repair to unconfirmed `Busy`
    runners that a later report omits. It adds GitHub calls and a stop round
    trip. Effort: Medium.
  - **E.** After a reconnect, the agent answers for each unconfirmed runner
    from a durable record of applied starts. It needs a protocol message and
    agent persistence. Effort: Large.
- **Left out:** Release on absence alone, because a report built before the
  VM existed can arrive after it. A resend of `StartRunner`, because the
  database never stores the plaintext JIT configuration. Exit records in
  reports, because they need the work of C and cover less.
- **Wider fix:** B, only when the private raw state write can no longer save
  `Ready` or `Busy`. Otherwise B is A behind a helper.
- **Previous examples:** Commit `0456ee0` added the confirmation fence for
  the first owner review. Its GitHub busy paths stayed outside it.
- **Recommendation:** B. State and confirmation then cannot be saved apart.
- **Action:** fix now, apply B. The user approved B on 6 November 2026.

### 2. Serving shutdown has no regression test

Its [follow-up item](follow-ups/control-plane-serving-shutdown-regression.md)
holds the full record and the work. This is the dated summary.

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Review, finding 2 of the second owner review of #31.
- **Category:** Test.
- **Location:** `crates/ciproxy-control-plane/src/application.rs:115`.
- **Problem:** No test checks shutdown after the service starts. A change
  that keeps the stop sender but does not send the stop passes every test.
- **Options:** **A.** One local test of the real executable, with events.
  **B.** An extracted serving lifecycle with injected work. **C.** A
  container lifecycle test. **D.** One shared shutdown type, tested with a
  paused clock.
- **Recommendation:** A. It checks the real signal path without a
  production refactor.
- **Action:** fix after merge, apply A, because it needs a new test harness
  and no current shutdown defect was confirmed. The user chose this on
  6 November 2026.

### 3. Listener start reads PostgreSQL on Tokio

- **Severity:** Low. **Effort:** Small.
- **Source:** Review, finding 3 of the second owner review of #31.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-control-plane/src/listeners.rs:134`, at
  `5bfb46f`.
- **Background:** PostgreSQL store methods are synchronous, so async code
  calls them on blocking workers. Listener discovery reads each
  organization on a blocking worker.
- **Problem:** Each listener start reads the organization again on a Tokio
  worker, only to keep its installation for the missing-scale-set repair.
- **Impact if no action:** The query holds a runtime thread while it waits,
  and a store error there ends serving.
- **Root question:** How does the listener supervisor get the installation
  to refresh after a missing scale set, without a synchronous store call on
  a Tokio worker?
- **Options:**
  - **A.** Move the read into `spawn_blocking`. Effort: Small.
  - **B.** Return the installation ID with each discovered target, and
    remove the read. Effort: Small.
  - **C.** Read the installation on a blocking worker only after the error.
    A read error can skip the repair. Effort: Small.
  - **D.** The GitHub listener returns the identity with the error, which
    changes a public API. Effort: Medium.
  - **E.** An App refresh by workspace and organization, which adds a
    public trait method. Effort: Medium.
  - **F.** A private async store handle for the control plane. Effort:
    Medium.
  - **G.** An async store client and async store traits in every crate.
    Effort: Large.
- **Left out:** `block_in_place`, because A is better on every point. More
  Tokio threads, because the call still blocks a thread.
- **Wider fix:** F for the control plane, and G for every crate.
- **Previous examples:** None found.
- **Recommendation:** B. It removes the query with no new store call and no
  public API change. F best protects the control plane.
- **Action:** fix now, apply B, and fix after merge, apply F, because F
  reaches files of the organization-add branch. The user approved B on
  6 November 2026. Its
  [follow-up item](follow-ups/control-plane-async-store-boundary.md) holds
  F.

## Milestone 1: Documentation

Depends on: none

This plan records the decisions. The runner guide names every source of
first confirmation. Two follow-up items keep the deferred work. No protocol
doc changes, because the routing requirement stays.

- [x] Write this plan from the user decisions and the merged checks of
      6 November 2026. The check reports are under
      `.context/reviews/conductor/runner-observation-confirmation/`.
- [x] Add the positive agent and GitHub observations to the confirmation
      sources of the runner guide, with one atomic write.
- [x] Update the `confirmed_at` sentence of the schema README.
- [x] Add the serving shutdown
      [follow-up item](follow-ups/control-plane-serving-shutdown-regression.md)
      for decision 2.
- [x] Add the async store
      [follow-up item](follow-ups/control-plane-async-store-boundary.md)
      for the wider fix F of decision 3.
- [x] Run the Markdown lint, the line-width check, and the local link check
      on the changed files. Output:
      `.context/runner-observation-confirmation/docs-checks/`.

## Milestone 2: Atomic observations and listener discovery

Depends on: 1

This milestone applies 1B and 3B. It keeps the absence fence, the 60-second
repair of unobserved starts, the `Busy` and `Stopping` protections, stop
authority and replay, take-out, and command results. It changes no schema,
protocol, HTTP route, setting, or public constructor.

- [x] Add the PostgreSQL runtime regression first. The start result is lost,
      GitHub reports the runner busy, and the job completes before the agent
      reconnects. A later full report omits the VM. Expect `Stopped` and
      released slots and disk. Show that `5bfb46f` fails on that assertion.
- [x] Cover two more cases in that regression: a report received before
      the busy observation keeps the runner, and registration alone leaves
      `confirmed_at` empty.
- [x] Add `observe_own_runner` to the store interface and PostgreSQL. Move
      each consumer under "Replaced ways" to it.
- [x] Narrow the private `save_state`. Delete `confirm_own_runner`, and move
      its store test to the new operation with every assertion.
- [x] Return the private discovered target from listener discovery, and
      delete the read at listener start. Keep the discovery filter, the
      listener keys, the stop branch, the drain, and the drop guards. Record
      its two small behaviour differences in the commit.
- [x] Run each new or changed asynchronous test ten times. Save the counts
      under `.context/runner-observation-confirmation/evidence/`.
- [x] After the source freeze, name the final API in the store interface
      and PostgreSQL store READMEs, and update each changed Key Code entry.

## Milestone 3: Verification, commit, review, and pull request

Depends on: 1, 2

- [x] Run the targeted tests of `ciproxy-store-pg` and
      `ciproxy-control-plane`.
- [ ] Run the complete gate `cargo xtask check` with `DATABASE_URL` set.
- [ ] Run the host HTTP smoke: the built executable with an isolated
      database, a local signed release, and a proxy that refuses outside
      calls. Expect readiness, liveness, the routes, and a clean SIGTERM
      exit.
- [ ] Run the actual container smoke,
      `crates/ciproxy-control-plane/scripts/container-smoke.sh`, with the
      same private inputs. Expect ready, live, exit 0, and no OOM kill.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against
      `origin/calummoore/ci-proxy-load-balancer` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record follow-up
      items under `plans/follow-ups/` for the rest, re-review once, and
      report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request into `calummoore/ci-proxy-load-balancer`. Its
      description gives the decisions and the design calls of this plan
      with their dates, a permalink to this plan, and the follow-up items
      that the change adds.
