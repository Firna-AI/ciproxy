# Liveness does not detect stalled runtime work

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of
  `calummoore/ci-proxy-load-balancer`, finding 1002, option E.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/listeners.rs`,
  `crates/ciproxy-control-plane/src/http.rs`, and
  `deploy/helm/ciproxy/templates/control-plane.yaml`.
- **Reason:** Progress checks change the health contract outside the store
  connection fix and the allowed source paths of that change.

- **Terms:**
  - **Tick:** One reconciliation pass of the control-plane runtime.
  - **Liveness:** The Kubernetes health check that restarts an unhealthy
    control-plane process.
  - **Local proxy:** The Cloud SQL Auth Proxy sidecar that accepts local
    database connections and forwards them to Cloud SQL.
- **Background:** The hosted control plane runs periodic reconciliation.
  Its liveness route always returns HTTP 200 while HTTP serving continues.
  Readiness becomes true after the first completed reconciliation pass.
  Store connections have connection, statement, lock, and idle transaction
  limits. These limits do not detect every stalled runtime operation.
- **Problem:** A lost connection between the local proxy and Cloud SQL can
  leave the local socket connected. Client keepalives then see a healthy
  local peer. A database timeout cannot deliver its error across the lost
  connection. A stalled lease check can retain the runtime operation lock.
  Neither health route reports that runtime work has stopped.
- **Impact if no action:** Dashboard requests and execution work can wait
  indefinitely while Kubernetes reports a healthy pod.
- **Root question:** How does the hosted service recover when a runtime
  operation stops making progress?
- **Options:**
  - **E.** Record the end of each completed tick. Make liveness fail when
    that time exceeds a configured limit. Kubernetes then restarts the
    stalled pod. Keep the limit above the longest valid tick, including
    startup and work across all workspaces. A restart interrupts work in
    progress, and requests can wait until the restart. Effort: Medium.
  - **F.** Give each asynchronous store call a deadline. A timed-out caller
    releases the runtime operation lock. This reaches the GitHub query
    helper, lease checks, and API and control-plane store handles. Blocking
    threads can remain stuck after the caller returns. Effort: Large.
- **Left out:** Database limits alone cannot detect a lost remote path
  behind a live local proxy. A connection pool alone cannot detect a
  stalled operation outside the database.
- **Wider fix:** E. Progress checks cover database and other runtime
  stalls without changing every store call boundary.
- **Previous examples:** Runtime entry points share one operation lock.
  A stalled store call can retain that lock while HTTP serving continues.
  The liveness route has no completed-work check.
- **Related:**
  [The asynchronous store boundary item](control-plane-async-store-boundary.md)
  owns the shared store handle needed by option F. This item owns process
  recovery, including stalls outside that handle.
- **Recommendation:** E. It detects stalled runtime progress across all
  causes while the connector limits ordinary database waits.
- **Action:** fix after merge, apply E. The user selected this follow-up on
  2026-11-17 while approving bounded store connections as the direct fix.
