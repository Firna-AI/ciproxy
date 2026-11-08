# A take-out runtime test flakes under load

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from the complete gate of the pull request #39
  merge.
- **Source:** Implementation.
- **Category:** Test.
- **Location:** `crates/ciproxy-control-plane/tests/runtime/overflow_takeout_tests.rs:107`,
  the test `takeout_waits_for_boat_retirement_and_saves_idle_and_busy_answers`.
- **Reason:** The merge did not change this test or the runtime code that
  it calls, so the fix belongs in a separate branch.
- **Terms:**
  - **Take-out:** the App removal flow that retires the runners of an
    organization, also its Boat runners, before it releases the
    organization.
- **Background:**
  - The test runs the take-out lifecycle three times. Each run answers
    differently for an idle Boat runner and a busy one. The third run must
    succeed.
  - In the complete gate, the third run returned `Err(StateChanged)`. The
    panic was `called Result::unwrap() on an Err value: StateChanged` at
    line 107.
  - The test then passed 5 of 5 runs alone, and the whole `runtime_tests`
    binary passed 6 of 6 runs. Other sessions used the same machine and
    PostgreSQL server during the failed gate.
- **Problem:** The test depends on an order or a timing that load can
  change. The source is not proven. Async ordering between the lifecycle
  call and a runtime tick, or shared fixture state, is the likely source.
- **Impact if no action:** The gate can fail without a code defect, and its
  Rust phase then skips the later test binaries.
- **Root question:** How does the take-out test control the order of the
  events that it checks?
- **Options:**
  - **A.** Reproduce the flake under load, name its source, and make the
    test deterministic with explicit event order or isolated state. Keep
    every assertion, and add no retry, sleep, or longer limit. Effort:
    Medium.
- **Left out:** A retry or a longer time limit hides the cause. Removing the
  test loses the take-out regression.
- **Wider fix:** None. The deterministic-test rules of the `testing` skill
  already cover the class.
- **Previous examples:** None found for this test.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, because the flake is unrelated to
  the branch that found it.
