# Events of a replaced connection can still apply

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 903 found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/runtime.rs`,
  `OwnMachines::agent`,
  `crates/ciproxy-control-plane/src/runtime_entrypoints.rs`, `agent`,
  `crates/ciproxy-control-plane/src/runtime_inbox.rs`, `consume`,
  `crates/ciproxy-control-plane/tests/runtime/settled.rs`,
  `settled_agent`,
  `crates/ciproxy-control-plane/tests/runtime/stale_remote_tests.rs`, and
  `crates/ciproxy-control-plane/runners.md`, the paragraph on the event
  queue.
- **Reason:** Commit `512224f` fixed the queued path, and today only tests
  call the agent entry that still skips the check.

- **Terms:**
  - **Event queue:** the one bounded queue in `runtime_inbox.rs`. It holds
    the agent events of the gateway callbacks until the runtime handles
    them.
  - **Session:** one WebSocket connection of a machine, with its own ID. A
    reconnect replaces the current session of the machine.
  - **Operation lock:** the process-wide lock that each runtime entry point
    takes: the tick, a job update, an agent event, and a lifecycle event.
  - **Agent entry:** `agent` of the public trait `OwnMachines` in
    `crates/ciproxy-control-plane/src/runtime.rs`. It handles one agent
    message of a workspace and a machine, and it takes no session.
- **Background:**
  - `crates/ciproxy-control-plane/runners.md` says that the runtime task,
    after it takes the operation lock, skips an agent event whose
    connection is no longer the current one of its machine.
  - At commit `9b094e1`, `consume` in `runtime_inbox.rs` read the current
    session of the machine with `machine_session`. When it matched the
    session of the event, it called `agent` of `runtime_entrypoints.rs`.
  - `agent` then waited for the operation lock, took the lease of the
    workspace, and handled the event. It did not read the session again.
    At that commit, `consume` and `agent` were the only path from the
    queue to the handler.
  - The wait for the lock lasts as long as the work that holds it, such as
    the reconciliation pass of another workspace.
  - A machine can reconnect during that wait. Its new session then
    replaces the old one.
  - On 17 November 2026 the final review chose a queue test that sends a
    valid event of a replaced session and expects no change. In that test,
    at commit `512224f` the session was already replaced when the event
    entered the queue. It did not cover a reconnect during the lock wait.
    The test is
    `gateway_queue_ignores_runner_exit_from_replaced_session` in
    `crates/ciproxy-control-plane/tests/runtime/queued_event_tests.rs`.
  - On the same date the final review chose a compact queued event of a
    fixed size. A fix of this item keeps that event type.
  - On 18 November 2026, commit `512224f` fixed the queued path. `consume`
    now takes the operation lock first. Then it reads the session, and it
    calls `agent_event` in `crates/ciproxy-control-plane/src/events.rs`
    only when the session is current. A reconnect during the lock wait no
    longer lets a queued event apply.
  - `consume` no longer calls `agent`, and it makes no check before the
    lock. The check sits in `consume`, not in one entry that every caller
    uses. So commit `512224f` does not implement option B.
  - The agent entry still takes the operation lock and calls `agent_event`
    with no session check. Its implementation is `agent` in
    `runtime_entrypoints.rs`.
  - No production code calls the agent entry today. The test helper
    `settled_agent` in
    `crates/ciproxy-control-plane/tests/runtime/settled.rs` calls it, and
    `stale_remote_tests.rs` in the same folder calls it directly.
    `OwnMachines` is public, so a later caller can reach it too.
  - Commit `512224f` added no test of replacement during the lock wait.
    This change extends that same test with lock-wait and command-delivery
    signals. It replaces the connection during the wait, then checks the
    saved busy state and the current command acknowledgement. The old
    lock order fails that saved-state assertion.
  - The remaining part of B is one entry, with the session check under the
    lock, for every caller that handles an agent event. The agent entry
    then takes the session of the event, and `settled_agent` and its
    callers change.
- **Problem:** The agent entry handles an agent event under the operation
  lock with no session, so it never checks that the connection is current.
  Only tests call it today, but each later caller skips the check that the
  queued path makes.
- **Impact if no action:** A later caller of the agent entry can apply an
  event of a replaced connection to the saved runner state. `runners.md`
  forbids this.
- **Root question:** How does the runtime handle an agent event only while
  its connection is the current one of its machine?
- **Options:**
  - **A.** Check the session again after `agent` takes the operation lock,
    and before it handles the event. Add a regression test that replaces
    the session while the event waits for the lock. Effort: Small.
  - **B.** Move the session check into `agent`, under the lock, and remove
    the check before the lock. One check then holds for every caller of
    `agent`. Keep the regression test of A. One method takes the lock and
    checks the session. Only the value that it returns lets the event
    handler run. Its type holds the lock guard and the checked event, so
    callers cannot skip that method. The regression still checks the order
    inside the method. Cost: `agent` needs the session of the event, so
    `settled_agent` and its callers change. Effort: Medium.
  - **C.** Save the session ID with each runner state write, and refuse a
    write from an older session in the store. Cost: a schema change and a
    store rule for each write. Effort: Medium.
- **Left out:** A shorter lock wait: it fails the goal, because any wait
  leaves the window open.
- **Wider fix:** B. The check runs under the lock, at the one entry point
  that handles agent events, so no caller can skip it.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/runtime-queue-fairness.md`: the same code,
  `consume` in `runtime_inbox.rs`. The options do not conflict, and neither
  item waits for the other.
- **Recommendation:** B. One check under the lock covers every caller of
  `agent`, with no second copy. A is the smallest direct fix.
- **Action:** fix after merge, apply B, because commit `512224f` fixed the
  queued path, and today only tests call the agent entry that B changes.
