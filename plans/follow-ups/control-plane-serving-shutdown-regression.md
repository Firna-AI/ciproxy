# Serving shutdown has no regression test

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from `conductor/runner-observation-confirmation`,
  finding 2 of the second owner review of pull request #31.
- **Source:** Review.
- **Category:** Test.
- **Location:** `crates/ciproxy-control-plane/src/application.rs:73-129`,
  the stop branch and the listener drain of `Hosted::supervise` in
  `crates/ciproxy-control-plane/src/listeners.rs`,
  `crates/ciproxy-control-plane/tests/startup_shutdown_tests.rs:30-106`,
  `crates/ciproxy-control-plane/src/_tests_/http_tests.rs:17-49`, and
  `crates/ciproxy-control-plane/README.md:74-77`.
- **Reason:** On 2026-11-06, the user chose this test for after the merge,
  because it needs a new test harness and no current shutdown defect was
  confirmed.

- **Terms:**
  - **Readiness:** The route `/api/health/ready`, which tells the load
    balancer whether the process takes new work. It answers 200 after the
    first reconciliation pass, else 503.
  - **Stop channel:** The `tokio::sync::watch` channel that
    `Production::serve` in `application.rs` creates after startup. Its
    sender `shutdown` tells the serving tasks to stop.
  - **Core supervisor:** `Hosted::supervise` in `listeners.rs`. It runs
    reconciliation, the scale-set listeners, webhook recovery, repository
    refresh, and the accounting worker.
  - **Grace wait:** The 240-second `timeout` that waits for the core
    supervisor, the HTTP server, and maintenance after the stop.
  - **Deliberate break:** A temporary change to product code that shows that
    a new test fails on an assertion. The author reverts it after the run.
- **Background:**
  - `deploy/runtime.md:75` requires that the process "stop within 300
    seconds". The chart sets `terminationGracePeriodSeconds: 300` in
    `deploy/helm/ciproxy/templates/control-plane.yaml:31`.
  - `crates/ciproxy-control-plane/README.md:74-77` gives the serving
    contract: "SIGTERM or Ctrl-C during startup stops the process with
    status 0 before it serves. After startup, the signal makes readiness
    return HTTP 503 at once. HTTP requests, listeners, and maintenance then
    get up to 240 seconds to finish. Other runtime work then gets up to five
    more seconds."
  - `application.rs:73-107` creates the stop channel. Its receivers go to
    the core supervisor, to the graceful-shutdown future of the axum
    server, and to the maintenance loop. The core supervisor gives clones
    to the accounting worker and to the recovery and refresh tasks.
  - On SIGTERM or Ctrl-C, `application.rs:115` calls
    `shutdown.send_replace(true)`. Line 116 sets readiness to false. Lines
    117-128 are the grace wait. `OwnedTask` in `src/tasks.rs` aborts each
    task that still runs when it drops. Then `src/cli.rs:68` gives the
    Tokio runtime five more seconds.
  - On a `true` value, the core supervisor leaves its loop, stops each
    listener, and awaits it. The HTTP server, maintenance, and the other
    workers stop on any change of the channel.
  - axum 0.8.9 stops its accept loop and drops the listening socket when
    the graceful-shutdown future completes. A new connection is then
    normally refused. Only a request on a connection that is already open
    can see the 503 of line 116.
  - `tests/startup_shutdown_tests.rs:30-106` sends SIGTERM while the
    release manifest request is pending, before HTTP binds.
    `src/_tests_/http_tests.rs:17-49` sets the readiness flag directly on
    the router. `tests/cli_startup_tests.rs` and `src/_tests_/cli_tests.rs`
    check the configuration and the version. No test calls
    `Production::serve` after startup, or `Hosted::supervise`.
- **Problem:** No test checks shutdown after the service starts. A change
  that keeps the stop sender but does not send the stop at once passes the
  complete gate. Examples are a move of `shutdown.send_replace(true)` after
  the grace wait, a `_shutdown` binding with the call deleted, and a send of
  `false`. The literal deletion of line 115 fails the gate only on the
  unused-variable lint, because the `rust-clippy` phase runs with
  `-D warnings`.
- **Impact if no action:** After such a change, the process keeps accepting
  connections for 240 seconds after the signal. It then aborts in-flight
  requests and listener sessions. Each rollout takes about four more minutes
  for each pod. The exit status stays 0 inside 300 seconds, so the operator
  container smoke still passes.
- **Root question:** Which gate test shows that, after startup, a
  termination signal stops new connections at once, reaches the serving
  work, and lets in-flight work finish before the process exits?
- **Options:**
  - **A.** Add one local integration test of the real executable, with
    active work and a controlled shutdown. Events show the stop delivery,
    the readiness withdrawal, and the work completion. It sees the HTTP
    drain and the exit, not each background worker. Effort: Medium.
  - **B.** Extract the serving lifecycle from startup. Test it with
    injected work and shutdown futures. It gives evidence for each task, and
    a paused clock can check the 240-second limit. It needs a production
    extraction. Effort: Medium.
  - **C.** Add a container lifecycle integration test with isolated outside
    services. This adds container infrastructure to the test setup. Effort:
    Large.
  - **D.** Move the order of stop, readiness, grace wait, and abort into one
    shared shutdown type. `Production::serve` and the listener set of the
    core supervisor use it. Unit-test it with a paused Tokio clock. It
    changes two shutdown paths with no confirmed defect. It does not catch a
    task that ignores its stop receiver, so A or B is still needed. Effort:
    Medium.
- **Left out:**
  - An exit-status check alone. It cannot tell a graceful stop from task
    cancellation.
  - Wall-clock assertions. They depend on machine speed.
  - A GitHub API host setting or a test hook in product code, so that the
    test can see the listener stops. It adds boundary behaviour that no
    requirement covers, or test detection in product code.
  - Shutdown log events as the observable. They add operator logs only for a
    test, and A already sees the main break through HTTP.
  - The operator container smoke,
    `crates/ciproxy-control-plane/scripts/container-smoke.sh`, as the
    protection. It runs outside the gate with private inputs.
- **Wider fix:** None. The `testing` skill already requires tests of
  observable boundary behaviour. D covers only the stop order, and a task
  that ignores its stop receiver still escapes it.
- **Previous examples:** None found.
- **Related:** [The async store item](control-plane-async-store-boundary.md)
  changes store calls in `Hosted::supervise`, the same code. Until this item
  lands, no test protects the stop path of that function, so that work must
  keep it unchanged. Neither item waits for the other.
- **Recommendation:** A. It checks the real signal path end to end with no
  production refactor. B gives stronger evidence for each task, but it needs
  a production extraction.
- **Action:** fix after merge, apply A, because it needs a new test harness
  and no current shutdown defect was confirmed. The user chose this on
  2026-11-06.
- **Steps for A:**
  1. Add the case beside the signal test in
     `crates/ciproxy-control-plane/tests/startup_shutdown_tests.rs`. Move
     the shared fixture code into a helper module, so that each file stays
     under 300 lines.
  2. Give the child what startup needs. `README.md:29-33` of the crate
     gives the signed manifest and `downloads.json`, which a local fake CDN
     can serve. Give it an isolated PostgreSQL database through
     `DATABASE_URL`. Give it secret files that parse, such as an RSA key in
     `github-app-private-key` (`composition.rs:75-83`). Without
     `DATABASE_URL`, the test fails. It never passes without its checks.
  3. Keep the child away from GitHub. The webhook recovery task calls the
     fixed host `https://api.github.com/` (`composition.rs:90`) on its first
     tick. One way needs no product change: point `HTTPS_PROXY` at a closed
     local port. The reqwest clients of `application.rs:40-47` and
     `composition.rs:84-89` keep the proxy settings of the environment.
     Verify this before you rely on it.
  4. Handle port reuse. The executable logs no bound address
     (`application.rs:65`), so the test must choose or find the port of
     `HTTP_BIND`. Another process can take a port that the test picks and
     releases. The flaky-test rule of the `testing` skill names port reuse.
     Prevent it, and record the method in the test.
  5. Use events only, and assert no elapsed time. Poll readiness until HTTP
     200, with a deadline of at least 10 seconds. Before the signal, hold
     one request in flight under test control. For example, send a partial
     body to `/hooks/github`, which reads the full body before its handler
     runs (`http.rs:45-69`).
  6. Send SIGTERM. Poll until a new connection is refused while that request
     is still open. Do not expect HTTP 503. Then complete the request, and
     assert its exact status. Assert that the child exits with status 0.
     Keep a hang guard of at least three times the typical run. The guard
     stops a hung test. It does not decide a result.
  7. Do not claim that A proves the stop of listeners, maintenance, or
     accounting. A sees the HTTP drain and the exit only.
  8. Prove the test with deliberate breaks. Move
     `shutdown.send_replace(true)` after the grace wait: the refused
     connection assertion must fail. Set the grace to zero: the in-flight
     request must get no status. Do not use the literal deletion of the
     call, because it fails on the lint, not on the test.
  9. A send of `false` also passes the gate today. The HTTP server and the
     other workers still stop on it, and only the core supervisor reads the
     value. So A sees that break only as a late exit, which the hang guard
     must not decide. B can catch it with injected work.
  10. Run the new test 10 times, and record the pass count.
  11. Update the readiness sentence of
      `crates/ciproxy-control-plane/README.md:74-75`. It says that the
      signal makes readiness return HTTP 503 at once. The HTTP server also
      stops new connections at that moment, so a new probe is normally
      refused. Say both facts. The serving contract stays the same.
