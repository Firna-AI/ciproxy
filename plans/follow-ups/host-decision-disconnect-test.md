# No test covers a start while disconnected

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 1103 found.
- **Source:** Review.
- **Category:** Test.
- **Location:** `crates/ciproxy-agent/src/observations.rs`,
  `Controller::observe`, the call of `release_pending_jobs` in its
  `GuestToAgent::JobStarting` arm,
  `crates/ciproxy-agent/src/job_decisions.rs`,
  `Controller::release_pending_jobs`, and
  `crates/ciproxy-agent/src/_tests_/host_decision_tests.rs`.
- **Reason:** The rule works today, and the gap only lets a later change
  break it unseen.

- **Terms:**
  - **Agent:** the open-source program on a customer machine that owns the
    job VMs. It lives in `crates/ciproxy-agent`.
  - **`JobStarting`:** the guest message that gives the agent the assigned
    job and its original credentials, before the first step of the job.
  - **Host decision:** the choice of the agent for a job: the machine cache
    with the original handoff, or the GitHub cache only. The steps of the
    job wait for it.
  - **Trusted facts:** the job facts that the control plane sends to the
    agent for a job, such as the event and the head repository.
  - **Control connection:** the WebSocket connection from the agent to the
    control plane.
- **Background:**
  - "The cache handoff" in `docs/protocol/cache.md` says that the steps of
    a job start only after the host decides. The runner sets no time limit
    of its own. GitHub's job time limit ends a job whose host never
    decides.
  - In `Controller::observe`, the `JobStarting` arm calls
    `release_pending_jobs` when the agent is not active or the control
    connection is down.
  - `release_pending_jobs` gives each running job without trusted facts the
    GitHub cache only, and sends that decision to its guest. So a job does
    not wait for facts that cannot arrive.
  - The module comment of `host_decision_tests.rs` states the rule: "A
    received job uses GitHub when trusted facts cannot arrive."
  - The existing tests disconnect only after the start, in
    `host_preserves_received_jobs_without_facts_through_disconnect_drain_remove_and_stop`.
    Or they have saved facts, in a test of
    `crates/ciproxy-agent/src/_tests_/cache_authority_tests.rs`.
  - The drain test in `crates/ciproxy-agent/src/_tests_/drain_tests.rs`
    uses a `Stopping` runner. That runner gets the GitHub-only decision
    from an earlier branch of `observe`, so it does not reach this call.
  - So a deletion of this `release_pending_jobs` call fails no test that
    the review found.
- **Problem:** No test sends a first `JobStarting` without saved facts
  while the control connection is down. The call that releases such a job
  has no test.
- **Impact if no action:** A later change can drop the call and pass every
  test. A job that starts while the control connection is down would then
  wait for a host decision until GitHub's job time limit ends it.
- **Root question:** Which test proves that a job that starts while the
  control connection is down gets the GitHub cache at once?
- **Options:**
  - **A.** Add one case to `host_decision_tests.rs`. Disconnect the control
    connection, then send a first `JobStarting` for a runner with no saved
    facts. Expect a GitHub-only `PrepareJob` to the guest, and a saved
    `GitHubOnly` decision. Effort: Small.
  - **B.** A, and the same case for an agent that is not active, such as a
    draining agent. Effort: Small.
- **Left out:** A test through the real executable with a guest: not a real
  choice. The unit level with the existing mocks reaches the call, and the
  `testing` skill puts each test at the lowest level that fails.
- **Wider fix:** None. The `testing` skill already asks for a test of each
  rule at its lowest level.
- **Previous examples:** None found.
- **Recommendation:** B. Both conditions of the call then have a test, for
  almost the same effort as A.
- **Action:** fix after merge, apply B, because the rule works today, and
  only a later change can break it.
