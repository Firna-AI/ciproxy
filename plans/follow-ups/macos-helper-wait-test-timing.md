# Two macOS helper wait tests use wall-clock time

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from PR #121, branch
  `calummoore/macos-s-vm-start-host-stop`, review finding 10.
- **Location:** `crates/ciproxy-agent/src/macos/_tests_/guests_tests.rs`,
  `a_new_helper_is_awaited_until_its_socket_answers` and
  `one_deadline_covers_a_request_in_flight`.
- **Reason:** Both tests passed 10 of 10 runs, and the fix only makes
  their proof stronger.

- **Terms:**
  - **Helper wait:** `wait_for_helper` in `macos/guests.rs`. It polls a VM
    helper's control socket until the helper answers, with one deadline.
- **Background:** The first test binds the socket after a 300 ms sleep
  and checks that at least 300 ms passed. The second holds a connection
  with no answer and checks that the wait fails in less than 5 s of wall
  time, with a 300 ms deadline.
- **Problem:** The first check follows from its own sleep and does not
  show that the wait polled while the socket was absent. The second
  depends on scheduling, and an immediate error before any request would
  also pass it.
- **Impact if no action:** A slow CI host can fail the second test, and a
  wrong early error can pass it.
- **Root question:** How do the tests prove when the helper wait must stay
  pending or end?
- **Options:**
  - **A.** Keep the real sockets. Signal from the fake helper when the
    first request arrives, and check the deadline with paused time.
    Effort: Small.
  - **B.** Inject the state probe for the timing tests, and keep one real
    socket test. Effort: Medium.
- **Left out:** Larger time bounds: they still depend on scheduling.
- **Wider fix:** None. The `testing` skill already asks for deterministic
  timing evidence.
- **Previous examples:** Commit `05f0e2d2` used paused time and socket
  events for the image download timeout test.
- **Recommendation:** A. Reproduce a failure first, and record the pass
  counts before and after.
- **Action:** fix after merge, apply A.
