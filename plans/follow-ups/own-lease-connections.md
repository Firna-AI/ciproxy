# Each tick opens a connection per workspace

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 1002 found.
- **Source:** Review.
- **Category:** Performance.
- **Location:** `crates/ciproxy-store-pg/src/own_lease.rs`, `acquire`,
  `crates/ciproxy-store-pg/src/own_runners.rs`, `routing_workspaces`,
  `crates/ciproxy-control-plane/src/runtime_entrypoints.rs`, `tick` and
  `agent`, and `crates/ciproxy-control-plane/src/runtime_reconciliation.rs`,
  `reconcile_workspace_inner`.
- **Reason:** The cost grows with the number of workspaces and machines,
  and ciproxy has no production workspaces yet.

- **Terms:**
  - **Lease:** a dedicated database connection that holds the advisory lock
    of one workspace. It makes the execution decisions of the workspace
    one at a time, also across pods.
  - **Operation lock:** the process-wide lock that each runtime entry point
    takes: the tick, a job update, an agent event, and a lifecycle event.
  - **Cloud SQL Auth Proxy:** the sidecar container that the control plane
    reaches on 127.0.0.1. It forwards each connection to Cloud SQL.
- **Background:**
  - `tick` in `runtime_entrypoints.rs` runs every 10 seconds. It reads the
    workspaces with `routing_workspaces`, which returns every workspace,
    also a workspace with no machines.
  - For each workspace in turn, `tick` takes the operation lock and runs
    the reconciliation. `reconcile_workspace_inner` takes the lease of the
    workspace first.
  - `own_lease::acquire` opens a new PostgreSQL connection for each lease,
    through the proxy. PostgreSQL starts a new server process for each
    connection. The connection closes when the lease ends.
  - Each queued agent event takes the lease again in `agent`, with one more
    new connection.
  - Each reconciliation sends a `DockerBudget` command to each machine, in
    `reconcile_workspace_inner`. Each machine answers with a
    `CommandResult`, which is one more queued agent event.
  - All these connections open one at a time, while the operation lock is
    held. So the setup time of each connection adds to the time that every
    other runtime entry point waits.
  - On 17 November 2026 the final review chose one connector, with time
    limits, for every production connection. The leases then still open one
    new connection each time, through that connector.
- **Problem:** Each 10-second tick opens one new database connection for
  every workspace, also for a workspace with no machines. Each agent event
  opens one more, and each pass makes each machine send one such event.
- **Impact if no action:** The time of each tick and the hold time of the
  operation lock grow with the number of workspaces and machines. Cloud SQL
  starts and stops one server process for each of these connections.
- **Root question:** How does the runtime take the lease of a workspace for
  each pass and each event without a new database connection each time?
- **Options:**
  - **A.** Skip each workspace that has nothing to route: no machine, no
    open job, and no Boat record. Cost: each other workspace still opens a
    connection for each pass and each event. Effort: Small.
  - **B.** Reuse the lease connections. Keep the idle lease connections in
    a small bounded pool from the store connector. Release the advisory
    lock with `pg_advisory_unlock` before a connection goes back to the
    pool. Cost: a pool with a size limit and a test that a returned
    connection holds no lock. Effort: Medium.
  - **C.** Hold the lease of each active workspace on one open connection
    between passes, and give it up only at shutdown or after an error.
    Cost: a second pod cannot take a held lease during a rolling update
    until the first pod stops. One connection stays open for each active
    workspace. Effort: Medium.
- **Left out:** A transaction-level advisory lock on the shared store
  connection: not a real choice. One execution section spans many store
  calls and remote waits, not one transaction.
- **Wider fix:** B. It removes the connection setup from each pass and each
  event, for every workspace. A removes only the empty workspaces.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/liveness-progress.md`: the same code. A progress
    limit for the liveness route must stay above the time of a valid tick.
    A fix of this item shortens the tick.
  - `plans/follow-ups/runtime-queue-fairness.md`: the same cause of slow
    event handling. A fix of this item drains the event queue faster.
- **Recommendation:** B. It keeps the lease rule of each workspace and
  removes the repeated connection setup. A is a small first step that B
  does not need.
- **Action:** fix after merge, apply B, because the cost grows only with the
  number of workspaces and machines, and no production workspace exists
  yet.
