# No test refuses another sandbox's token at exit

- **Severity:** Medium. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1: a problem that
  the check of finding 103 found outside the scope.
- **Location:** `crates/ciproxy-store-pg/src/overflow/completion.rs:26` and
  `crates/ciproxy-overflow/src/service/cleanup.rs:30`.
- **Reason:** The exit claim is on `main`, and the change that found the
  gap did not change it.

- **Terms:**
  - **Sandbox token:** the token that a Boat sandbox holds. Job code in the
    sandbox can read it. It may only end the runner of its own sandbox and
    send figures for that job, as `docs/protocol/security.md` says.
  - **Exit claim:** the store write that records the exit report of a Boat
    runner, before the control plane deletes its sandbox.
- **Background:**
  - The exit report names a runner and carries the sandbox token. The
    service first finds the workspace of the token
    (`crates/ciproxy-overflow/src/service/cleanup.rs:30`). The claim then
    matches the token hash and the runner
    (`crates/ciproxy-store-pg/src/overflow/completion.rs:26`).
  - The exit tests send a token of another workspace, and the workspace
    step refuses it before the claim.
  - The branch that added Boat sample reports added the matching case for
    them: a second live sandbox in the same workspace sends its real token
    with the runner of the first sandbox, and gets HTTP 401.
- **Problem:** No exit test sends the real token of another sandbox in the
  same workspace. A change that drops the token filter of the claim passes
  every test. The lint `cargo clippy -- -D warnings` flags only an unused
  `hash`, and a rename to `_hash` passes it.
- **Impact if no action:** After such a change, job code that knows the
  runner ID of another sandbox in its workspace could end that sandbox,
  and the tests would not show it.
- **Root question:** How can a test prove that the exit claim refuses a
  token that does not own the runner it names?
- **Options:**
  - **A.** Add an exit test with two live Boat sandboxes in one workspace.
    Queue two jobs before the first tick, as
    `crates/ciproxy-control-plane/tests/runtime/overflow_registration_retention_tests.rs:111-114`
    does. Send the exit report with the real token of the second sandbox
    and the runner of the first. Assert the refusal, no saved claim, and
    no delete of the first sandbox. Effort: Small.
  - **B.** Add a store test of the claim with two deliveries and different
    token hashes. It does not fail when the service stops using this
    claim. Effort: Small.
- **Left out:** The lint alone, because a rename passes it.
- **Wider fix:** None. One verifier for both report paths is the item
  `plans/follow-ups/boat-report-token-verifier.md`.
- **Previous examples:** The sample report case above.
- **Related:** `plans/follow-ups/boat-report-token-verifier.md`: a shared
  verifier would need this test too.
- **Recommendation:** A. It tests the threat at the route, with the real
  service and store, and it fails also for a lookup that checks only the
  workspace.
- **Action:** fix after merge, apply A, because the exit claim is on
  `main` and outside the change that found the gap.
