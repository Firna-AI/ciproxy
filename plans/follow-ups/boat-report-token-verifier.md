# Boat report paths check the token in two ways

- **Severity:** Low. **Effort:** Large.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1: the blocked
  wider fix of finding 103.
- **Location:** `crates/ciproxy-overflow/src/service/reports.rs:51`,
  `crates/ciproxy-store-pg/src/job_usage/boat.rs:24` and `:51`, and
  `crates/ciproxy-store-pg/src/overflow/completion.rs:26`.
- **Reason:** The fix replaces the live exit lookup on `main`.

- **Terms:**
  - **Sandbox token:** the token that a Boat sandbox holds. It may only end
    the runner of its own sandbox and send figures for that job.
  - **Exit report** and **sample report:** the two reports that the guest
    in a Boat sandbox sends to the control plane: the exit of its runner,
    and the CPU and memory figures so far.
- **Background:**
  - The exit path finds the workspace of the token
    (`crates/ciproxy-overflow/src/service/cleanup.rs:30`), then claims the
    exit with the token hash and the runner
    (`crates/ciproxy-store-pg/src/overflow/completion.rs:26`).
  - The sample path reads the target with the token hash and the runner
    (`boat_usage_target` at `crates/ciproxy-store-pg/src/job_usage/boat.rs:24`).
    Its locked write then checks the delivery, the runner, and that the
    sandbox has no recorded end, but not the token (`boat.rs:51-61`).
  - Route tests now refuse a wrong token on the sample path, also the real
    token of another sandbox of the same workspace.
- **Problem:** The two paths check the same right with two different
  lookups, so a change can weaken one path and not the other.
- **Impact if no action:** Each change to the token rules must touch two
  lookups and their tests.
- **Root question:** How can every Boat report path check, in one way,
  that the sender owns the runner it names?
- **Options:**
  - **A.** Replace both lookups with one verifier that returns a delivery
    only after it matches both the token and the runner. Keep the exit
    claim atomic. Effort: Large.
  - **B.** Keep two lookups, but pass the token hash to `save_boat_sample`
    and add the token filter to its locked select, as the exit claim does.
    It never refuses a request alone today, because the token hash is
    unique and nothing changes it after the reservation. Effort: Medium.
- **Left out:** A lookup by the token alone with a runner compare in code,
  because the token filter stays the only guard.
- **Wider fix:** A. One verifier keeps the two paths from drifting apart.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/boat-exit-claim-token-test.md`: the exit
  path lacks the test that the sample path has, and A needs it.
- **Recommendation:** A. It removes the second lookup instead of guarding
  it.
- **Action:** ask the user, because A replaces the live exit lookup on
  `main`.
