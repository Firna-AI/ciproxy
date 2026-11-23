# Hosted rejection logs can quote request values

- **Severity:** Medium. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1: a problem that
  the check of finding 101 found outside the scope.
- **Location:** `crates/ciproxy-gateway/src/logging.rs:13`.
- **Reason:** The log guard is on `main`, and it covers every hosted route,
  not only the routes of the change that found the problem.

- **Terms:**
  - **Hosted log guard:** the tracing layer in
    `crates/ciproxy-gateway/src/logging.rs` that each hosted service
    installs. It drops the events of some log targets at every verbosity.
  - **Rejection event:** the `axum::rejection` trace event that axum writes
    when a `Json`, `Query`, or `Path` extractor refuses a request.
- **Background:**
  - Commit `12994b8` added the guard, so that the WebSocket frame logs of
    `tungstenite` and `tokio_tungstenite` never reach hosted output, at
    any verbosity. The function `permitted` lists only those two targets.
  - An operator can set `RUST_LOG=trace` on a hosted service to see more
    detail.
- **Problem:** With `RUST_LOG=trace`, each rejection event writes serde's
  error message to hosted output, and that message can quote the refused
  value. A review saw this on the Boat exit route: a malformed report made
  the hosted log print the refused value. Job code controls that body, and
  an agent or a browser controls the bodies of other routes.
- **Impact if no action:** A hosted log at trace level can keep request
  values that never passed a check, such as job data or a field that a
  client sent by mistake.
- **Root question:** How can hosted output stay free of request values at
  every verbosity, also for a request that an extractor refuses?
- **Options:**
  - **A.** Add `axum::rejection` to the targets that the guard blocks, with
    a trace test like the test of `12994b8`. Cost: an operator loses the
    trace event that names the cause of a refusal. The answer code stays.
    Effort: Small.
  - **B.** Give the hosted routes shared extractor wrappers whose rejection
    logs only the kind of error, without the message of serde, and block
    `axum::rejection` as in A. The cause stays visible without the value.
    Effort: Medium.
- **Left out:** A rule in each route, because a new route would not get
  it.
- **Wider fix:** A. The guard holds for every route, also a later one.
- **Previous examples:** `12994b8` blocked the WebSocket frame logs for the
  same reason. `plans/follow-ups/boat-exit-report-response-log.md` keeps
  the same rule for the diagnostics of the Boat callback.
- **Recommendation:** A. It closes the class with the smallest change,
  and the answer code still tells the operator that a request was
  refused.
- **Action:** ask the user, because A removes a diagnostic that an
  operator can turn on today on `main`.
