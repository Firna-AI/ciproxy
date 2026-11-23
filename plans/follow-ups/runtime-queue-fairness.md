# One machine can fill the shared event queue

- **Severity:** High. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 901 found.
- **Source:** Review.
- **Category:** Security.
- **Location:** `crates/ciproxy-control-plane/src/runtime_inbox.rs`,
  `RuntimeTasks::enqueue` and `consume`, and
  `crates/ciproxy-control-plane/runners.md`, the paragraph on the event
  queue.
- **Reason:** The fix of the final review bounds the memory of each queued
  event. A share of the queue for each workspace is a new queue design
  outside that fix.

- **Terms:**
  - **Event queue:** the one bounded queue in `runtime_inbox.rs`. It holds
    the agent events and the machine removals of the gateway callbacks
    until the runtime handles them.
  - **Operation lock:** the process-wide lock that each runtime entry point
    takes: the tick, a job update, an agent event, and a lifecycle event.
  - **Lease:** a dedicated database connection that holds the advisory lock
    of one workspace.
  - **Full report:** the complete state report that each machine sends to
    the control plane every 10 seconds.
- **Background:**
  - The gateway callbacks put each agent event and each machine removal on
    one queue, in arrival order, and return at once.
  - `RuntimeTasks::enqueue` creates one `mpsc::channel(1024)` for the whole
    process. All workspaces share it.
  - One supervised task handles the events one at a time. Each agent event
    waits for the operation lock in `agent` of
    `crates/ciproxy-control-plane/src/runtime_entrypoints.rs`. It also opens
    a new PostgreSQL connection for the lease of its workspace.
  - So a machine can always send valid events faster than the runtime
    handles them.
  - When the queue is full, `try_send` drops the new event and logs a
    warning. `crates/ciproxy-control-plane/runners.md` says that the next
    full report and the saved-stop resend then repair the saved state.
  - The gateway saves each full report itself, in `receive` of
    `crates/ciproxy-gateway/src/gateway.rs`. Full reports do not pass
    through the queue, so a full queue does not drop them.
  - On 17 November 2026 the final review chose to queue compact events of a
    fixed size. That bounds the memory of the queue. It does not change the
    share of each workspace.
  - "Tenant isolation" in `docs/protocol/security-isolation.md` says that
    tenant isolation holds between workspaces.
- **Problem:** One machine can keep the shared queue full with small valid
  events. While the queue stays full, `try_send` drops the events of every
  other workspace.
- **Impact if no action:** Other workspaces lose their runner and command
  events until their next full report repairs the state. One customer
  machine can slow the routing of every other customer.
- **Root question:** How does the runtime keep the events of one workspace
  from pushing out the events of the other workspaces?
- **Options:**
  - **A.** Give each workspace its own bounded queue. One consumer takes
    one event from each workspace in turn. A full queue then drops only the
    events of its own workspace. Cost: a map of queues, the turn-taking
    consumer, and tests for the order inside one workspace. Effort: Medium.
  - **B.** Keep one queue, but count the queued events of each workspace.
    Refuse a new event of a workspace above its share, such as 64 events.
    Cost: the share needs a number, and a large workspace with many
    machines can reach it in normal use. Effort: Small.
  - **C.** Limit the event rate of each machine connection in the gateway,
    before the queue. Cost: a limit for each connection, and a machine with
    many busy slots can reach it in normal use. Other machines of the same
    customer still add up. Effort: Small.
- **Left out:**
  - A larger queue: it fails the goal, because one machine can fill any
    fixed size.
  - Blocking sends in the gateway: the read loop of each machine would wait
    for the runtime again. The queue exists to remove that wait.
  - Faster event handling: not a real choice. Each event waits for the
    operation lock and opens a lease connection, so a machine can always
    send faster.
- **Wider fix:** A. Each workspace gets its own bound, so no workspace can
  push out another, also for a new kind of event.
- **Previous examples:** Round 1 of the same review found that slow provider
  work of one workspace stopped the machine events of every workspace. The
  fix put the gateway events on this queue and the provider work in tasks.
- **Related:**
  - `plans/follow-ups/inbox-session-recheck.md`: the same code, `consume`
    in `runtime_inbox.rs`. The options do not conflict, and neither item
    waits for the other.
  - `plans/follow-ups/own-lease-connections.md`: the same cause of slow
    handling. Each event opens a new lease connection. A fix there drains
    the queue faster, but it gives no workspace its own share.
- **Recommendation:** A. A queue for each workspace keeps the drops inside
  the workspace that causes them, with no shared number to tune.
- **Action:** fix after merge, apply A, because a full report repairs each
  dropped event within 10 seconds, and A is a new queue design.
