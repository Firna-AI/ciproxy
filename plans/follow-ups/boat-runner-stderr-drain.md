# Boat runner entry can end before its stderr drains

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-runner-start`, review round
  1, finding 2. Its check confirmed it, narrowed its trigger, and added
  option D.
- **Location:** `image/boat-runner-entry.sh:4-12`, `forward` and `main` in
  `image/boat-runner-stderr.py`, `crates/ciproxy-guest/src/service.rs:156-158`,
  and `image/ciproxy-boat-guest.service:12`.
- **Reason:** Only a guest that ends before the sandbox delete loses a
  pending helper line. That happens after a failed exit report, or after a
  saved report that returns 204 before a deferred delete. Otherwise a
  working callback deletes the sandbox first.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Guest:** the ciproxy program in the image that starts the runner of
    one job and reports the end of that runner.
  - **Runner entry:** the installed command that loads the shell settings
    of the `runner` account and then starts the GitHub runner's listener.
  - **Journal helper:** `image/boat-runner-stderr.py`, which keeps a
    bounded, redacted copy of the runner's stderr in the sandbox journal.
  - **Control group:** the set of processes of one systemd unit, which
    systemd stops together.
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
  - The guest starts `runuser` in a new process group, with stderr to
    `/dev/null`, at `crates/ciproxy-guest/src/process.rs:63-87`. It waits
    only for that child, at lines 127-131. At
    `crates/ciproxy-guest/src/service.rs:156-158`, it sends the exit report
    and returns. The Boat shutdown at
    `crates/ciproxy-guest/src/power.rs:43-50` does nothing.
  - `image/ciproxy-boat-guest.service:12` sets `KillMode=control-group`.
    When the guest ends, systemd sends SIGTERM to each remaining process of
    the unit. The `runuser` PAM service has no `pam_systemd`, so the helper
    stays in the control group of the guest unit.
  - With a working callback, the control plane first saves the report as a
    one-use claim. It then waits up to 5 seconds for the workspace lock and
    deletes the sandbox inside that request, while the guest waits in its
    exit report. The code is at
    `crates/ciproxy-control-plane/src/boat_callback.rs:41`, then
    `crates/ciproxy-overflow/src/service/cleanup.rs:58` and `:77`. On
    2026-11-21, two sandboxes ended 7 and 6 seconds after their reports.
    The deletion, not the guest stop, then ends the journal.
  - When that lock wait ends first, or the delete fails, the control plane
    still answers 204, because the report is saved. The guest then ends
    before the delete, and a later cleanup pass deletes the sandbox.
    "The exit report" in `docs/protocol/overflow-sandbox-end.md` states
    this rule.
  - The operator guide covers the reads that come before a deletion. The
    restore smoke waits up to 15 seconds for a fresh journal record. The
    job smoke copies the journal while the sandbox exists. See
    `image/boat-smoke.md`.
- **Problem:** The end of the Boat runner entry does not show that its
  stderr is in the journal. After a runner start failure, an exit report
  can fail fast, for example on an unreachable control plane or a fast 401
  or 503. A saved report can also return 204 before its delete, when the
  workspace lock stays busy for 5 seconds or the delete fails. In each case
  the guest ends at once, and systemd stops the helper. A line that the
  helper has not yet written is then lost.
- **Impact if no action:** After a report that fails before its claim, the
  sandbox stays until its lifetime ends, 6 hours or 2 hours on a Boat free
  trial. Its journal is the only record, and it holds no runner error.
  After a saved report with a deferred delete, a later cleanup deletes the
  sandbox, and no journal copy holds the last line. In both cases, an
  operator must reproduce the failure on a new paid sandbox.
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
  - **D.** Add a bounded `ExecStop=` wait to
    `image/ciproxy-boat-guest.service`. systemd runs it after the guest
    ends on its own, before it sends SIGTERM to the remaining processes.
    The command waits until the helper has ended, with a short limit. For
    example, the launcher takes a lock before the common entry starts, and
    the helper holds it until it ends. The `exec` chain and the
    signal handling of the runner stay. D covers only a guest that ends
    before the sandbox delete. That guest ends after a failed exit report,
    or after a saved report that returns 204 before a deferred delete. It
    does not order the line before the exit report, the restore smoke read,
    or the local check. A local test needs systemd, so its proof is a live
    restore with a forced report failure. Effort: Medium.
- **Left out:**
  - A fixed delay after the runner exits, because it does not show that
    the line reached the journal.
  - A new process group for the helper, because the helper stays in the
    same control group.
  - An eager journal writer with a ready signal at the start, because it
    only makes the window shorter. It does not order the write before the
    end of the entry.
  - A direct journal write from the error hook, because it orders only the
    hook's own records. The Bash missing-registration message and the
    listener's stderr stay unordered.
  - `KillMode=process` in the guest unit, because it overrides the cleanup
    of leftover runner processes. D keeps that cleanup with a short wait.
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
  a larger cost. D is the narrow fallback when the launcher must keep
  `exec`.
- **Action:** fix after merge, apply A, because only a guest that ends
  before the sandbox delete loses the pending line. After a report that
  fails before its claim, the lifetime stays the backstop. After a saved
  report that returns 204 first, a later cleanup deletes the sandbox.
