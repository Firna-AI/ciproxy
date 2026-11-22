# A revocation during a macOS start can fail removal

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from PR #121, branch
  `calummoore/macos-s-vm-start-host-stop`, review finding 8.
- **Location:** `crates/ciproxy-agent/src/platform/macos/vms.rs`,
  `MacVms::start`; `crates/ciproxy-agent/src/vsock.rs`, `GuestChannels::send`;
  `crates/ciproxy-agent/src/retirement.rs`; `crates/ciproxy-agent/src/runtime.rs`.
- **Reason:** It needs a credential revocation within about 1 s of a
  start, and the same window existed before, during prepare and
  bootstrap.

- **Terms:**
  - **Credential rejection:** the gateway event that tells a connected
    agent that its credential is revoked. The agent then removes itself.
  - **Delivery claim:** the agent's record that one operation delivers a
    runner's configuration.
- **Background:** During a start, the delivery claim is active from before
  `prepare` until the guest bridge exists. `MacVms::start` now also waits
  for the helper (normally about 1 s) before `Guests::attach`. A removal
  during a delivery claim sends `Shutdown` through `GuestChannels::send`,
  which fails with `Disconnected` when no channel exists yet. The removal
  passes the error up, and the daemon stops its command worker.
- **Problem:** A revocation in that window ends the daemon before its
  removal is done.
- **Impact if no action:** The helper can keep running until the next
  agent start recovers it.
- **Root question:** How does a machine honour a revocation while a VM is
  starting?
- **Options:**
  - **A.** Register the guest channel before the helper wait, so it keeps
    a shutdown request until the bridge runs. Effort: Medium.
  - **B.** Let each delivery claim record a shutdown before the bridge
    exists, and carry it into the bridge before the first configuration.
    Test a revocation with the helper answer held. Effort: Medium.
  - **C.** A start phase that a removal can cancel before delivery.
    Effort: Large.
- **Left out:** Ignore each `Disconnected` shutdown: a VM could then end
  with neither a sent nor a pending shutdown.
- **Wider fix:** B. It covers every delay before the bridge exists.
- **Previous examples:** Commit `6b6162a2` made a concurrent removal use a
  guest shutdown during delivery.
- **Recommendation:** B.
- **Action:** fix after merge, apply B, because the window is short and
  existed before.
