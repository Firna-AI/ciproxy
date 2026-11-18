# Events of a replaced connection can still apply

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 903 found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/runtime_inbox.rs`,
  `consume`, `crates/ciproxy-control-plane/src/runtime_entrypoints.rs`,
  `agent`, and `crates/ciproxy-control-plane/runners.md`, the paragraph on
  the event queue.
- **Reason:** The window needs a reconnect of the machine while its event
  waits for the operation lock.

- **Terms:**
  - **Event queue:** the one bounded queue in `runtime_inbox.rs`. It holds
    the agent events of the gateway callbacks until the runtime handles
    them.
  - **Session:** one WebSocket connection of a machine, with its own ID. A
    reconnect replaces the current session of the machine.
  - **Operation lock:** the process-wide lock that each runtime entry point
    takes: the tick, a job update, an agent event, and a lifecycle event.
- **Background:**
  - `crates/ciproxy-control-plane/runners.md` says that the runtime task
    skips an agent event whose connection is no longer the current one of
    its machine.
  - `consume` in `runtime_inbox.rs` reads the current session of the
    machine with `machine_session`. When it matches the session of the
    event, it calls `agent` of `runtime_entrypoints.rs`.
  - `agent` then waits for the operation lock, takes the lease of the
    workspace, and handles the event. It does not read the session again.
  - The wait for the lock lasts as long as the work that holds it, such as
    the reconciliation pass of another workspace.
  - A machine can reconnect during that wait. Its new session then
    replaces the old one.
  - On 17 November 2026 the final review chose a queue test that sends a
    valid event of a replaced session and expects no change. In that test,
    the session is already replaced when the event enters the queue. So it
    does not cover a reconnect during the lock wait.
  - On the same date the final review chose a compact queued event of a
    fixed size. A fix of this item keeps that event type.
  - The functions `consume` and `agent` are the only path from the queue
    to the handler. Tests also call `agent` directly through the helper
    `settled_agent` in `crates/ciproxy-control-plane/tests/runtime/`.
- **Problem:** `consume` checks the session before the event waits for the
  operation lock, and not again after it. With rare timing, the runtime
  handles an event whose connection was replaced during that wait.
- **Impact if no action:** An event of the old connection can change the
  saved runner state after the machine reconnected. `runners.md` forbids
  this.
- **Root question:** How does the runtime handle an agent event only while
  its connection is the current one of its machine?
- **Options:**
  - **A.** Check the session again after `agent` takes the operation lock,
    and before it handles the event. Add a regression test that replaces
    the session while the event waits for the lock. Effort: Small.
  - **B.** Move the session check into `agent`, under the lock, and remove
    the check before the lock. One check then holds for every caller of
    `agent`. Add the regression test of A. Cost: `agent` needs the session
    of the event, so `settled_agent` and its callers change. Effort: Small.
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
- **Action:** fix after merge, apply B, because the window needs a
  reconnect of the machine during the lock wait.
