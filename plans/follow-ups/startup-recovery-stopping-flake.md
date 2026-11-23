# Startup recovery test saw Starting once

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Low.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, the control-plane work.
- **Location:** `crates/ciproxy-control-plane/tests/runtime/startup_recovery_tests.rs:105`.
- **Reason:** The test failed once in about 25 runs, and the cause is not
  known yet.

- **Source:** Implementation.
- **Category:** Test.
- **Terms:**
  - **Startup recovery:** the control-plane step at startup that finds
    runners whose configuration was lost and retires them on GitHub first.
- **Background:**
  - The test
    `runtime::startup_recovery_tests::confirmed_configuration_loss_retires_github_first_with_exact_resources_and_preserves_busy`
    checks that a runner moves to `Stopping` after a confirmed loss.
  - On 23 November 2026, one full `cargo test --workspace` run failed it
    with this text: `assertion 'left == right' failed`, `left: Starting`,
    `right: Stopping`, at line 105.
  - The same test then passed in two full runs and in 21 runs on its own.
  - That run was during the change that renamed the control-plane
    overflow modules and moved provider starts to a registry. The change
    did not touch this test file.
- **Problem:** The test can read the runner state before recovery has moved
  it to `Stopping`. The suspected source is a wait on time, or an order of
  background tasks, that a loaded machine can break.
- **Impact if no action:** The gate can fail now and then with no product
  fault, and a real fault in startup recovery can hide behind it.
- **Root question:** How does the test wait until startup recovery has made
  its decision?
- **Options:**
  - **A.** Make the test wait for the recovery event or the store change,
    not for a time. Effort: Medium.
  - **B.** Run the test 200 times under load first, to find the order that
    fails, then apply A. Effort: Medium.
- **Left out:** A longer timeout or a retry, because the `testing` skill
  forbids both.
- **Wider fix:** A. A wait on an event removes this class in the test.
- **Previous examples:** None found.
- **Recommendation:** B. The cause is not proven, so first find the order
  that fails.
- **Action:** fix after merge, apply B, because the failure is rare and the
  cause needs its own investigation.
