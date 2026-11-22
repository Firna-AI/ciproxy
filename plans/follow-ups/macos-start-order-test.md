# No test checks the macOS start order

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch
  `calummoore/macos-s-vm-start-host-stop`, review finding 5.
- **Location:** `crates/ciproxy-agent/src/platform/macos/vms.rs`,
  `MacVms::start`; `crates/ciproxy-agent/src/macos/_tests_/guests_tests.rs`;
  `.github/workflows/ci.yml`, the `MACOS_BUILD_ONLY_PACKAGES` list.
- **Reason:** A test of the order needs the macOS start to move into code
  that Linux tests reach, with injected host operations.

- **Terms:**
  - **VM helper:** the launchd job that runs one macOS job VM and answers
    on its control socket.
  - **Guest bridge:** the agent task that `Guests::attach` starts. It
    checks the VM liveness at once, and it reads a missing helper socket
    with no `exit.json` as a gone VM.
- **Background:**
  - `MacVms::start` runs `launchctl bootstrap`, then `wait_for_helper`,
    then `Guests::attach`. Without the wait in that order, each VM ended
    about 200 ms after its start, and the control plane started a new
    runner every 10 s.
  - The Linux tests call `wait_for_helper` directly. `MacVms` builds only
    on macOS. The CI job "macOS crates" builds the agent on macOS but does
    not run its tests. `scripts/macos-helper-smoke.sh` starts a helper
    without `MacVms`.
- **Problem:** A change that moves `wait_for_helper` after `attach`, or
  drops it, passes every test.
- **Impact if no action:** The early VM end can come back with green CI.
- **Root question:** How does the normal gate catch a guest bridge that
  attaches before the VM helper answers?
- **Options:**
  - **A.** A macOS test of `MacVms::start` with a fake helper reply. The
    CI job would need to run agent tests on macOS. Effort: Medium.
  - **B.** Move the start steps into one function in `src/macos/` that
    takes the launchd, helper, and guest operations as traits. A Linux
    test holds the helper reply and checks that `attach` comes only after
    it. Effort: Medium.
  - **C.** A Mac smoke test that sends `StartRunner` to a signed agent and
    waits for the runner configuration. It needs a Mac and a test image.
    Effort: Large.
- **Left out:** Another test of `wait_for_helper` alone: it misses a
  change of the call order.
- **Wider fix:** B. The order lives in one function that the gate tests.
- **Previous examples:** Commit `56f5b414` moved the VM settings builder
  into code that the Linux tests reach with the real check.
- **Related:** `plans/follow-ups/macos-start-wait-worker.md` changes how
  the start waits. Test the start path after that change.
- **Recommendation:** B. The normal gate then catches the original fault.
- **Action:** fix after merge, apply B, because it needs a new seam for
  the macOS start.
