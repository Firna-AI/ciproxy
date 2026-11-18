# First offline assignment lacks fallback coverage

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the check of final-review finding 1103.
- **Source:** Review.
- **Category:** Test.
- **Location:** `crates/ciproxy-agent/src/observations.rs:92` and
  `crates/ciproxy-agent/src/_tests_/host_decision_tests.rs`.
- **Reason:** The missing case predates the guest-message cleanup.

- **Terms:**
  - **JobStarting:** the guest message that gives the agent the original
    job assignment before action setup.
  - **GitHubOnly:** the host decision that keeps the original GitHub cache
    endpoints for the whole job.
- **Background:** A received job must use GitHub when the agent cannot
  obtain trusted job facts. The agent releases pending jobs when its
  control connection is down.
- **Problem:** No test sends the first `JobStarting` for a warm runner
  without saved facts while the control connection is already down.
  Removing the disconnected `release_pending_jobs` call from `observe`
  therefore leaves the existing tests passing.
- **Impact if no action:** A regression can leave that job waiting for a
  host decision that cannot arrive.
- **Root question:** How should tests protect the first offline job handoff?
- **Options:**
  - **A.** Add the case to the host-decision tests. Check the GitHub-only
    preparation, retained job, and refusal of later cache promotion.
    Effort: Small.
- **Left out:** Another test entry for the same requirement adds no
  distinct approach. The existing disconnect case starts the job before
  disconnecting, so it cannot cover this order.
- **Wider fix:** None. The testing skill already requires each missing
  boundary case to protect observable behaviour.
- **Previous examples:** The restored-cache test in
  `crates/ciproxy-agent/src/_tests_/cache_authority_tests.rs` has saved
  facts. The pending-removal drain test starts a stopping runner, which
  selects GitHub-only before the disconnected release call.
- **Recommendation:** A. It protects the missing order in the existing
  test group.
- **Action:** fix after merge, apply A, because this test gap is outside
  the guest-message cleanup.
