# A macOS stop trusts the helper socket as exit proof

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from PR #121, branch
  `calummoore/macos-s-vm-start-host-stop`, review findings 6 and 9.
- **Location:** `crates/ciproxy-agent/src/macos/guests.rs`,
  `wait_for_helper_end`; `crates/ciproxy-agent/src/platform/macos/vm_helper.rs`,
  `run`; `crates/ciproxy-agent/src/platform/macos/vms.rs`, `MacVms::stop`;
  `crates/ciproxy-agent/src/platform/macos/seat_states.rs`, `clean_build`.
- **Reason:** The gap lasts only milliseconds after a failed VM stop, and
  a real exit proof needs a new check of the helper process.

- **Terms:**
  - **VM helper:** the launchd job `ciproxy-agent vm-run LOCAL_ID`. It runs
    one macOS VM and answers on `area/vms/LOCAL_ID/control.sock`.
  - **Seat:** one of the 2 macOS VM places of a Mac, with a fixed MAC
    address and machine identifier.
- **Background:**
  - `launchctl bootout` can return before the helper process ends. So
    `MacVms::stop` calls `wait_for_helper_end`: it polls the control
    socket every 100 ms, for up to 40 s, until the socket is gone or
    refuses connections. Only then does it free the record and the seat.
  - On SIGTERM, `vm_helper::run` calls `vm.stop()`. When that call fails,
    it logs a warning, writes `exit.json`, waits 1 s, removes the socket,
    and returns. The process then exits, and the VM ends with it.
  - Each state request of `VmControlClient::phase` can wait 10 s. The
    stop loop checks its 40 s deadline only after each request, so the
    wait can last up to about 50 s.
  - `SeatStates::clean_build` boots out a saved-state build helper and
    removes its folder with no exit wait.
- **Problem:** After a failed `vm.stop()`, the socket goes a moment before
  the process and its VM end. The stop can then free the seat while the
  old VM still runs. The stop wait can also run past its 40 s limit, and
  the build cleanup does not wait at all.
- **Impact if no action:** In a rare case, a new VM can start on the seat
  with the same identity while the old VM ends, or fail on Apple's limit
  of 2 VMs. A silent helper can hold the agent worker for up to 50 s.
- **Root question:** What proves that the agent can reuse a seat after a
  stop?
- **Options:**
  - **A.** Keep the socket after a failed VM stop until the process ends,
    put one deadline on the whole `wait_for_helper_end`, and use the same
    wait in `clean_build`. Effort: Small.
  - **B.** Record the helper process ID at start, and wait in one stop
    operation until the operating system no longer has that process. Use
    it for job VMs and builds. Test that a missing socket alone keeps the
    seat. Effort: Medium.
  - **C.** Hold a file lock per seat for the whole VM life, and make each
    start and cleanup respect it. Effort: Large.
- **Left out:** Another check of the launchd label: launchd can drop it
  before the process ends.
- **Wider fix:** B. One exit proof for every stop caller.
- **Previous examples:** Commit `007b6d43` added a wait for the label to
  go, to prevent stale jobs. Commit `04bb8d92` kept Linux network indices
  reserved until cleanup was proven.
- **Recommendation:** B. It proves the exit of the process that owns the
  VM, not of its socket.
- **Action:** fix after merge, apply B, because the gap needs a failed VM
  stop and lasts milliseconds.
