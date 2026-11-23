# The macOS start wait accepts a stopped helper

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from PR #121, branch
  `calummoore/macos-s-vm-start-host-stop`, review finding 7.
- **Location:** `crates/ciproxy-agent/src/macos/guests.rs`,
  `wait_for_helper` and `HelperLiveness`.
- **Reason:** The guest bridge already reports such a VM end within
  seconds, so only the start result is late.

- **Terms:**
  - **VM helper:** the launchd job that runs one macOS VM and answers on
    its control socket.
  - **`exit.json`:** the file that a helper writes with the reason that
    its VM ended. It then serves its socket for 1 more second.
- **Background:** `MacVms::start` waits with `wait_for_helper` until the
  helper answers a state request, and then attaches the guest bridge. The
  wait accepts every answer, also `stopped`. It reads `exit.json` only
  after a missing socket or a timeout.
- **Problem:** A VM that ends right after its start can pass the wait.
  The start command then answers success, and the bridge reports the end
  a moment later.
- **Impact if no action:** The control plane first sees a started runner,
  then its exit, not a failed start.
- **Root question:** How does a start report a VM that ends before it is
  available?
- **Options:**
  - **A.** Read `exit.json` before each request, and treat `stopped` as a
    failed start. Test it with a helper that still serves. Effort: Small.
  - **B.** Share one helper state check between `wait_for_helper` and
    `HelperLiveness`, where `exit.json` and `stopped` win over a working
    socket. Keep the retry for a socket that is not there yet.
    Effort: Small.
  - **C.** A separate start-result request in the helper protocol.
    Effort: Medium.
- **Left out:** A shorter wait: the first answer can already be
  `stopped`.
- **Wider fix:** B. The start and the bridge then read a stopped VM the
  same way.
- **Previous examples:** Commit `05d013d4` added the 1 s in which a
  stopped helper still answers.
- **Recommendation:** B.
- **Action:** fix after merge, apply B, because the bridge already reports
  the end.
