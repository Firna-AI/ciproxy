# Rerouting after a runner exit restores Busy

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-06, from `conductor/runner-observation-confirmation`,
  the post-push review of commit `fdc52ca`, which listed it outside the
  scope. The same flow exists on the primary branch at `5bfb46f`.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/events.rs:27`, `:87-106`,
  `:118-127`, and `:191-201`, `crates/ciproxy-control-plane/src/slots.rs:19`,
  and `crates/ciproxy-control-plane/runners.md:71-72`.
- **Reason:** The flow is older than the change that found it, and that
  change had no approval to fix it.

- **Terms:**
  - **Routing snapshot:** The `RoutingSnapshot` that `MachineRuntime::load`
    reads in one transaction. It holds the runners, jobs, machines, and
    settings of one workspace.
  - **Agent event:** A `RunnerReady`, `JobStarted`, or `RunnerExited`
    message from the agent about one runner. `MachineRuntime::agent_event`
    in `events.rs` handles it.
  - **Observation:** The store write `observe_own_runner`, which saves the
    runner state, its GitHub ID, and its first confirmation together.
  - **Stored `Started` job:** A durable GitHub job record in the `Started`
    phase that names the GitHub ID of the runner.
- **Background:**
  - `agent_event` loads one routing snapshot before it handles the event
    (`events.rs:27`).
  - It saves the event as an observation (`events.rs:87-100`). For
    `RunnerExited`, the saved state is `Stopped`.
  - Then it calls `route_job` for every stored job, with that same snapshot
    (`events.rs:104-106`).
  - `route_job` finds the runner by GitHub ID, organization, and scale set.
    It skips the runner when the snapshot shows it `Stopped`
    (`events.rs:118-127`). For a `Started` job, it saves `Busy` unless the
    runner is `Stopping` (`events.rs:191-201`).
  - Each runner that is not `Stopped` holds its slots and disk
    (`slots.rs:19`). The runner guide says that the capacity stays held
    until a later full report omits the VM, or the runner exits
    (`runners.md:71-72`).
- **Problem:** After `RunnerExited`, the snapshot still holds the old state
  of the runner, such as `Busy`. When GitHub has not completed the job yet,
  its record is still `Started`. `route_job` then saves `Busy` over the
  `Stopped` that the event saved just before.
- **Impact if no action:** The stopped VM shows as `Busy` and holds its
  slots and disk until the next full report omits it. That report releases
  it, because the runner is confirmed. No running VM or job result is lost.
  Another start can wait for one report interval.
- **Root question:** How does the rerouting after an agent event see the
  state that the event just saved?
- **Options:**
  - **A.** Skip the rerouting after `RunnerExited`. A stopped runner takes
    no job. It fixes only this event. Effort: Small.
  - **B.** After an accepted observation, load one fresh routing snapshot,
    and reroute the stored jobs of the event with it. `route_job` then sees
    `Stopped` and returns early. It adds one snapshot read for each event
    that reroutes. Effort: Small.
  - **C.** Let the store write return the updated runner or a new snapshot,
    and replace the old runner in memory before the rerouting. It changes
    the store interface for one caller. Effort: Medium.
- **Left out:** A store rule that never moves a `Stopped` runner back to
  `Busy`. It puts runtime lifecycle policy into the store, which keeps its
  queries apart from business decisions. No change, because the capacity
  then waits for the next report.
- **Wider fix:** B. Every rerouting after an observation then reads the
  saved state, also for later agent events.
- **Previous examples:** None found.
- **Recommendation:** B. One fresh snapshot keeps this caller from acting on
  a state that the event replaced. A is smaller, but it fixes one event and
  keeps the stale input for the others.
- **Action:** fix after merge, apply B, because the flow is older than the
  change that found it and needs its own branch and regression.
- **Steps for B:**
  1. Add the regression first, in the PostgreSQL runtime tests under
     `crates/ciproxy-control-plane/tests/runtime/`. Use a real own runner
     and a durable `Started` job for its GitHub ID. Send `RunnerExited`
     before GitHub completes the job. Assert `Stopped`, and assert that the
     job did not restore `Busy`. See it fail on the current code on that
     assertion.
  2. In `agent_event`, load the fresh snapshot after the observation, and
     pass it to `route_job`.
  3. Keep the protected `RunnerReady` return, the machine and runner scope
     checks, the `Stopping` rule, and saved stop replay.
  4. Run the new test 10 times, and record the pass count.
