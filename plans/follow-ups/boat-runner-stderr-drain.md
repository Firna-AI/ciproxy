# Boat runner entry can end before its stderr drains

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-runner-start`, review round
  1, finding 2. Its check confirmed the original guest-stop race and
  added the former option D.
- **Location:** `image/boat-runner-entry.sh:4-12`, `forward` and `main` in
  `image/boat-runner-stderr.py`, `Platform::runner_command` in
  `crates/ciproxy-guest/src/platform.rs`, and `Supervisor::report_exit` in
  `crates/ciproxy-guest/src/service.rs`.
- **Reason:** The guest service no longer stops the helper when it ends.
  A sandbox delete can still end the helper before it writes its last line.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Guest:** the ciproxy program in the image that starts the runner of
    one job and reports the end of that runner.
  - **Runner entry:** the installed command that loads the shell settings
    of the `runner` account and then starts the GitHub runner's listener.
  - **Journal helper:** `image/boat-runner-stderr.py`, which keeps a
    bounded, redacted copy of the runner's stderr in the sandbox journal.
  - **Runner scope:** a systemd unit with a separate control group for
    the runner and its child processes.
- **Background:**
  - The orchestrator's call for this work: keep a bounded, redacted copy
    of runner stderr in the sandbox journal, with no protocol change.
    The task specifies no helper completion order at the guest exit.
    Before this change, the runner's stderr went to `/dev/null`.
  - `image/boat-runner-entry.sh:4-7` starts the journal helper as a Bash
    coprocess, and line 9 sends the entry's stderr to it. Line 12 replaces
    the entry with the common entry through `exec`, and
    `image/runner-entry.sh:20` replaces that with the listener. So
    `runuser` waits for the listener, and no process waits for the helper.
  - In the helper, `forward` starts `systemd-cat` only at the first
    message, and `main` reads stdin to its end. The helper sets no signal
    handler.
  - On 23 November 2026, the user chose option B of decision 2 in
    [CPU and memory of each job step](../job-step-cpu-and-memory.md):
    start the runner in its own scope with `OOMPolicy=continue`.
    The helper runs in the runner scope, not the guest service.
    `KillMode=control-group` on the guest service no longer stops it.
  - The guest waits for its `runuser` child, then sends the exit report.
    It does not wait for the helper. Boat mode does not shut down the
    sandbox when the guest ends.
  - With a working callback, the control plane first saves the report as a
    one-use claim. It then waits up to 5 seconds for the workspace lock and
    deletes the sandbox inside that request, while the guest waits in its
    exit report. The code is at
    `crates/ciproxy-control-plane/src/boat_callback.rs:41`, then
    `crates/ciproxy-overflow/src/service/cleanup.rs:58` and `:77`. On
    2026-11-21, two sandboxes ended 7 and 6 seconds after their reports.
  - When that lock wait ends first, or the delete fails, the control plane
    still answers 204, because the report is saved. The guest then ends
    before the delete, and a later cleanup pass deletes the sandbox.
    "The exit report" in `docs/protocol/overflow-sandbox-end.md` states
    this rule.
  - The operator guide covers the reads that come before a deletion. The
    restore smoke waits up to 15 seconds for a fresh journal record. The
    job smoke copies the journal while the sandbox exists. See
    `image/boat-smoke.md`.
- **Problem:** The runner entry can end before the helper writes its last
  line. The exit report can cause a sandbox delete while the helper still
  writes. A failed report or a deferred delete gives the helper more time,
  but does not make a later delete wait for the write.
- **Impact if no action:** A journal read can miss the last runner error.
  If the sandbox ends before the write, the operator can lose the error
  and must reproduce the failure on a new paid sandbox.
- **Root question:** How can the end of the Boat runner entry show that its
  stderr is in the journal, within a short limit, so that the steps after
  it do not lose or miss the last line?
- **Options:**
  - **A.** Keep the Boat launcher as the parent. On runner exit, close the
    pipe and allow a bounded drain before it returns the runner status.
    Preserve the signal handling, and use a held journal writer to check
    the order. The recording writer of the success case in
    `image/check-boat-runner.py`, held open, can serve as that order test.
    Effort: Medium.
  - **B.** Let the guest own the runner and the diagnostic reader in Boat
    mode, and give its exit path a bounded drain before the report. This
    needs a Boat-only change in guest Rust, so it waits for a later guest
    release. It does not cover the restore smoke, which runs the entry
    without the guest. Effort: Medium.
  - **C.** Give the helper a separate sandbox service, and let the
    launcher wait for an explicit completion signal from it, up to a limit.
    This needs a new service and local coordination. It also needs a new
    path for the registration value that the helper uses to redact,
    because a separate service does not inherit it. Effort: Large.
- **Left out:**
  - A fixed delay after the runner exits, because it does not show that
    the line reached the journal.
  - A new process group for the helper, because a group does not make the
    runner wait for the journal write.
  - An eager journal writer with a ready signal at the start, because it
    only makes the window shorter. It does not order the write before the
    end of the entry.
  - A direct journal write from the error hook, because it orders only the
    hook's own records. The Bash missing-registration message and the
    listener's stderr stay unordered.
  - A wait in `ExecStop=` of the guest service (former D), because the
    helper is in the runner scope and the wait starts after the exit
    report. Changing `KillMode=` in the guest service also cannot order the
    write before the report.
  - A direct journal stream as the runner's stderr, because it removes the
    filter and breaks the redaction requirement.
  - A copy outside the sandbox, because it changes a protocol message or
    adds a boundary, which the orchestrator's call excludes. Examples are
    a bounded tail in the exit report, a control-plane journal read before the
    deletion, and a kept failed sandbox.
- **Wider fix:** A. The same launcher owns both processes and their exit
  order on every Boat run.
- **Previous examples:** Commit `c9fcf480` holds a request across shutdown,
  to check that the control plane finishes pending work before it exits.
- **Recommendation:** A. It is the one option that makes the end of the
  entry the completion point. So the guest stop, the exit report, the
  restore smoke read, and the local check all come after the last line. It
  stays in the Boat image, with no protocol or guest change. Keep the limit
  to a few seconds: the drain normally takes milliseconds, and a longer
  wait delays the deletion and adds billed time. C gives the same order at
  a larger cost.
- **Action:** fix after merge, apply A, because the drain must end before
  the guest sends an exit report that can cause a sandbox delete.
