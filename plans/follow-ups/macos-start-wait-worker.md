# A slow macOS VM start holds the agent worker

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch
  `calummoore/macos-s-vm-start-host-stop`, review finding 1.
- **Location:** `crates/ciproxy-agent/src/platform/macos/vms.rs`,
  `MacVms::start` and `MacVms::stop`;
  `crates/ciproxy-agent/src/macos/guests.rs`, `wait_for_helper` and
  `wait_for_helper_end`; `crates/ciproxy-agent/src/runtime_worker.rs`.
- **Reason:** The hold happens only when a VM helper does not answer, and
  the fix changes how the one agent worker schedules its work.

- **Terms:**
  - **VM helper:** the launchd job `ciproxy-agent vm-run LOCAL_ID` that
    runs one macOS job VM and answers on `area/vms/LOCAL_ID/control.sock`.
  - **Agent worker:** the task in `runtime_worker.rs` that runs control
    commands, such as `StartRunner`, `StopRunner`, and `Remove`, and guest
    messages, one at a time.
- **Background:**
  - `MacVms::start` waits up to 60 s until a new helper answers on its
    control socket, and only then attaches the guest bridge. Without the
    wait, the bridge read the missing socket as a gone VM and ended it.
  - `MacVms::stop` waits up to 40 s after `launchctl bootout` until the
    helper no longer answers, so no new VM takes the seat, with the same
    MAC address and machine identifier, while the old VM runs.
  - A helper usually answers within about 1 s, and ends within a few
    seconds of a stop.
  - The worker awaits each command. A Mac runs up to 2 VMs.
- **Problem:** While one helper does not answer, the worker waits up to
  60 s on a start or 40 s on a stop. A `Remove` command or a guest
  message of the other VM, such as `JobStarting`, waits behind it.
  `docs/protocol/agent.md` says that a machine removal stops its warm
  runners at once.
- **Impact if no action:** In the rare case of a silent helper, a removal
  or a job step of the other VM waits up to about a minute.
- **Root question:** How does a Mac serve its other VM while one VM
  helper starts or stops?
- **Options:**
  - **A.** Move the start wait into the guest bridge task, and report a
    helper that never answers through the runner exit path. The start
    command then answers before the helper runs. Effort: Medium.
  - **B.** Run the helper waits in a supervised task. The worker goes on
    with other work, and keeps the operations of one VM in order. Test a
    removal while another helper does not answer. Effort: Medium.
  - **C.** Give each VM its own work queue, and coordinate machine-wide
    mode changes across the queues. This also changes the Linux worker.
    Effort: Large.
- **Left out:** A shorter wait: it still holds the worker, and a slow but
  healthy start would fail.
- **Wider fix:** C. It also covers other long operations of one VM.
- **Previous examples:** Commit `24ab86a5` moved report delivery out of
  the command worker, so a held operation no longer delays reports.
- **Related:** `plans/follow-ups/macos-start-order-test.md`: a test of the
  start order must follow the same start path after this change.
- **Recommendation:** B. It keeps the start result and the order of one
  VM, and frees the worker for the other VM.
- **Action:** fix after merge, apply B, because the hold needs a silent
  helper, and the change reaches the shared worker.
