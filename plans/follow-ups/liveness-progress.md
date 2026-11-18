# Health checks pass while the runtime stalls

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`: the wider fix E of finding
  1002, split off on 17 November 2026.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/http.rs`, the route
  `/api/health/live`, `crates/ciproxy-control-plane/src/listeners.rs`,
  `supervise` and its ready flag,
  `crates/ciproxy-control-plane/src/runtime_entrypoints.rs`, the operation
  lock of each entry point, and
  `deploy/helm/ciproxy/templates/control-plane.yaml`, the `livenessProbe`
  of the control plane.
- **Reason:** The connection limits of the branch already end the database
  waits that a server setting can see. This item covers the rarer stalls
  that no connection limit sees.

- **Terms:**
  - **Supervisor:** `supervise` in
    `crates/ciproxy-control-plane/src/listeners.rs`. It runs one
    reconciliation tick every 10 seconds and starts the GitHub listeners.
  - **Operation lock:** the process-wide lock that each runtime entry point
    takes: the tick, a job update, an agent event, and a lifecycle event.
  - **Lease:** a dedicated database connection that holds the advisory lock
    of one workspace.
  - **Liveness probe:** the Kubernetes check that restarts a container when
    its route fails.
  - **Cloud SQL Auth Proxy:** the sidecar container that the control plane
    reaches on 127.0.0.1. It forwards each connection to Cloud SQL.
- **Background:**
  - `/api/health/live` in `crates/ciproxy-control-plane/src/http.rs` always
    returns 200.
  - `/api/health/ready` returns 200 after the first tick of the supervisor
    sets the ready flag. The flag stays true after that.
  - The Helm chart points the startup and readiness probes at
    `/api/health/ready`, and the liveness probe at `/api/health/live`. The
    control plane runs as one replica.
  - So no stall makes Kubernetes restart the control plane.
  - Each runtime entry point takes the operation lock. So one stalled store
    call that holds the lock stops every runtime entry point.
  - The ordinary store calls of a process share one database connection. A
    lease opens a dedicated connection for each workspace.
  - The control plane reaches the Cloud SQL Auth Proxy on 127.0.0.1. That
    local peer stays alive when the path between the proxy and Cloud SQL
    is lost.
  - So client keepalive and `tcp_user_timeout` cannot see a lost path, and
    a server `statement_timeout` cannot send its error across it.
    `connect_timeout` still works, because libpq applies it to the full
    startup exchange.
  - On 17 November 2026 the final review chose one connector for every
    production connection. It sets connection, statement, and lock limits,
    and `idle_in_transaction_session_timeout` for a stalled lock holder.
    `PgStore` keeps only that connector.
  - Those limits cannot see a lost path behind the proxy, a stuck lease
    connection that holds the operation lock, or a stall outside the
    database.
  - The tick reconciles each workspace in turn. So the time of a valid tick
    grows with the number of workspaces.
- **Problem:** A stall that no connection limit sees stops every runtime
  entry point. Both health routes still answer 200, so Kubernetes does not
  restart the pod.
- **Impact if no action:** Routing, job updates, and agent events stop for
  every workspace until a person restarts the control plane.
- **Root question:** How does the hosted service get back to work when a
  stall that no connection limit sees stops the runtime?
- **Options:** Only two approaches give the work back without a person, as
  **Left out** says.
  - **A.** Make `/api/health/live` report progress.
    - `supervise` records the end of each completed tick. The route fails
      when the last completed tick is older than a limit. Kubernetes then
      restarts the stalled pod.
    - A ends every kind of stall. Examples are a lost path behind the proxy,
      a stuck lease connection that holds the operation lock, and a stall
      outside the database.
    - A drops the assumption that only the waiting caller can see a stall.
      The source of that assumption is the code: the health routes report
      no progress.
    - Cost: dashboard calls still wait until the restart, and the restart
      stops work in progress. The limit must stay above the longest valid
      tick, and that time grows with the number of workspaces.
    - Effort: Medium.
  - **B.** Put an application deadline on each store call at the async
    boundary.
    - The places are `ciproxy_github::persistence::query`, the lease check
      `Lease::check` of `Handle` in
      `crates/ciproxy-github/src/persistence/lease.rs`, and one async store
      handle for the control plane and the API. That handle is option F of
      `plans/follow-ups/control-plane-async-store-boundary.md`.
    - At the deadline, the caller gets an error and releases the operation
      lock. A stuck lease connection then holds only its thread and the
      lease of its own workspace.
    - A stuck call on the shared connection keeps the connection lock, so
      later calls also fail. They fail until `supervise` stops on the tick
      error and the pod restarts.
    - Cost: about 75 direct `spawn_blocking` calls in four crates move
      behind the handle. The abandoned threads stay until their queries
      end.
    - Effort: Large.
- **Left out:**
  - No change beyond the connection limits: it fails the goal. Those limits
    cannot see a lost path behind the proxy or a stall outside the
    database.
  - A timed wait for the shared connection alone, inside
    `PgStore::connection()`: it fails the goal for a stuck lease
    connection. The lease check holds the operation lock with no limit.
  - An alert on an old tick, with no restart: it fails the goal, because a
    person must still restart the pod.
  - A bounded connection pool, as option G of
    `plans/follow-ups/control-plane-async-store-boundary.md`: it fails the
    goal alone, because nothing restarts a stalled process.
- **Wider fix:** A. It covers every kind of stall, inside and outside the
  database. B covers only the store calls.
- **Previous examples:** Round 1 of the same review found a hang in which
  the gateway events filled the Tokio blocking pool. The lock holder then
  could not run its next store call, and both health routes still reported
  healthy. The queue of agent events removed that cause, but not the blind
  spot of the health routes.
- **Related:**
  - `plans/follow-ups/control-plane-async-store-boundary.md`: B uses its
    option F as the async store handle. Neither item waits for the other.
  - `plans/follow-ups/own-lease-connections.md`: the same code. Each tick
    opens one lease connection for each workspace under the operation lock.
    The limit of A must stay above the time of such a tick. A fix of that
    item shortens the tick.
- **Recommendation:** A. It restarts a stalled pod whatever the cause, at
  Medium cost, and it keeps the store design. B moves about 75 call sites
  and still needs a restart for a stuck shared connection.
- **Action:** fix after merge, apply A, because the stalls that it covers
  are rare. The connection limits of the branch already end the common
  database waits.
