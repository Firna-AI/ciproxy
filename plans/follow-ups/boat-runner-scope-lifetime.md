# Stopping the Boat guest leaves its runner alive

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, finding 303.
- **Location:** `crates/ciproxy-guest/src/platform.rs:101`, the
  `systemd-run` launcher in `Platform::runner_command`,
  `image/ciproxy-boat-guest.service:10` and `:12`,
  `image/ciproxy-guest.service:10`,
  `crates/ciproxy-guest/src/_tests_/platform_tests.rs:26`,
  `image/check-runner-environment.py:49-57`, and
  `crates/ciproxy-guest/README.md:45-51`.
- **Reason:** The fix needs the real systemd image checks before an image
  release, and the defect needs an operator stop or a guest crash.

- **Terms:**
  - **Guest:** the ciproxy program in each job VM and each Boat sandbox. It
    starts the GitHub runner, waits for it, and sends the exit report with
    the CPU and memory figures of the job.
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Guest unit:** the systemd service that runs the guest:
    `image/ciproxy-boat-guest.service` in a Boat sandbox, and
    `image/ciproxy-guest.service` in a Linux job VM of an own machine.
  - **Runner scope:** the systemd scope unit that the guest makes with
    `systemd-run --scope` for the runner and its child processes. It has
    its own control group.
  - **Control group:** the Linux group of processes that systemd manages
    and stops as one unit.
- **Background:**
  - Before this branch, the runner ran in the control group of the guest
    unit. The guest unit sets `KillMode=control-group`
    (`image/ciproxy-boat-guest.service:12`). So each end of the guest unit
    stopped the runner too.
  - On 23 November 2026, the user chose option B of the out-of-memory
    decision of this branch. B starts the runner in its own systemd scope
    with `OOMPolicy=continue`. So an out-of-memory kill of one job process
    no longer stops the guest unit.
  - The launcher runs `systemd-run --scope --quiet
    --property=OOMPolicy=continue`, then `runuser`
    (`crates/ciproxy-guest/src/platform.rs:101-115`). The runner stays a
    child process of the guest, so the guest still waits for it and gets
    its exit code. The runner also keeps its own process group
    (`crates/ciproxy-guest/src/process.rs:93`).
  - The scope has no lifetime link to the guest unit. So
    `KillMode=control-group` no longer reaches the runner.
  - On Boat, the stderr helper `image/boat-runner-stderr.py` also runs in
    the runner scope. It keeps a bounded, redacted copy of the stderr of
    the runner in the journal of the sandbox.
  - The guest has no SIGTERM handler. A search of `crates/ciproxy-guest`
    finds no signal handler.
  - The Boat guest unit has no restart and no reboot at exit, and the Boat
    power step does nothing (`crates/ciproxy-guest/src/power.rs:50-53`).
    The guest unit of an own machine reboots the job VM when it stops
    (`image/ciproxy-guest.service:11`), and the reboot ends the runner. So
    the defect is on Boat.
  - Three ends of the guest unit need different answers: an operator stop,
    a guest crash, and the normal end after the exit report. At the normal
    end, the runner has already exited. So a link at that end stops only
    the stderr helper of the runner and leftover processes.
  - These rules of `docs/protocol/job-usage.md` stay:
    - A failed sample or a failed sample report never stops the runner or
      its exit report (`:140-141`).
    - A lost or refused exit report loses its figures. On Boat, the newest
      sample report then gives the figures of the job (`:142-143`).
    - The Out of memory decision: no stop of the processes of the job ends
      the guest, and the job does not run inside the service of the guest
      (`:176-180`).
  - ciproxy stops and deletes each sandbox when its job ends
    (`docs/protocol/overflow-sandbox-end.md:28`). So a job that outlives
    its guest runs until the job-end delete, with no more figures.
  - The chroot check of the image replaces `systemd-run` with a stand-in
    that makes no scope (`image/check-runner-environment.py:49-57`). No
    check in this repository runs a real scope. A real check needs a Linux
    job VM with KVM and a Boat sandbox, which the workspace of this branch
    did not have.
- **Problem:** The runner scope outlives each end of the Boat guest unit.
  An operator stop, such as `systemctl stop ciproxy-boat-guest.service`,
  ends the guest but not the job. A guest crash also leaves the job
  running. The guest can then no longer send the final report.
- **Impact if no action:** A job can go on after an operator stops the
  guest. The job then has no exit report, and its page shows only the
  figures of the newest sample report.
- **Root question:** Which ends of the Boat guest must also end its runner,
  and what links the two lifetimes now that the runner runs in its own
  scope?
- **Options:**
  - **A.** Name the runner scope, and stop it from `ExecStopPost=` of the
    Boat guest unit. Cost: a stop command and a naming rule for the scope.
    Effort: Medium.
  - **B.** Bind each runner scope to its guest unit. Start the scope with
    `--property=BindsTo=<guest unit>` and `--property=After=<guest unit>`.
    systemd then stops the scope on each end of the guest unit: a stop, a
    crash, and the normal end. The two control groups stay apart for the
    out-of-memory rule. Cost: the launcher needs the name of the guest
    unit, and only a real systemd host proves the link. Effort: Medium.
  - **C.** Add a supervisor process that owns both control groups and ends
    the runner when the guest dies. Cost: one more process in each image.
    Effort: Large.
  - **D.** Do B, but with `PartOf=` in place of `BindsTo=`.
    - A stop or a restart of the guest unit then stops the runner scope
      first. A crash and the normal end do not.
    - So the stderr helper keeps draining after the exit report.
    - A crashed guest leaves its job running until the job-end delete,
      with no more figures.
    - Cost: the same launcher change and proof as B.
    - Effort: Medium.
  - **E.** Add a SIGTERM handler to the guest.
    - On SIGTERM, the supervisor stops the runner group with the existing
      `StockRunner::stop` (`crates/ciproxy-guest/src/process.rs:160`) if
      the runner still runs. It then sends the exit report with the final
      figures and exits.
    - Give the Boat guest unit a `TimeoutStopSec=` that bounds the report.
    - E goes with B or D, so that an operator stop also delivers the
      figures. Alone, it covers only SIGTERM, not a crash or SIGKILL.
    - Cost: a signal path in the supervisor loop, and a test with a fake
      signal source.
    - Effort: Medium.
  - **F.** Use the parent-death signal of the kernel.
    - In `pre_exec` of the runner command, set `PR_SET_PDEATHSIG` to
      SIGTERM. The kernel then signals the top runner process, the
      `runuser` parent, when the guest dies, also after SIGKILL or a
      panic. `runuser` passes the signal on to the listener.
    - The normal end leaves the stderr helper alone, and the launcher needs
      no unit name.
    - Cost: `unsafe` code in the Apache-2.0 guest crate. The signal
      follows the thread that started the child, not the process. So a
      later change of the async runtime can end a healthy job. It works
      only on Linux.
    - Effort: Medium.
  - **G.** Keep the separate lifetime, and document it in `image/boat.md`
    and `crates/ciproxy-guest/README.md`.
    - A stopped or failed Boat guest leaves its job running with no more
      reports. The job-end delete then ends the sandbox. An operator stops
      the job through the runner scope.
    - Cost: an operator stop of the guest no longer ends the job. After a
      crash, the figures of the job end at its last sample report.
    - Effort: Small.
- **Left out:**
  - The runner back in the control group of the guest unit, because it
    breaks the user's choice of 23 November 2026 and the Out of memory
    decision at `docs/protocol/job-usage.md:176-180`. On `origin/main`,
    the runner already had its own process group, so the old link was the
    control group.
  - A delegated control group inside the guest unit, because it puts the
    job back inside the service of the guest, which the same decision
    rejects.
  - A transient service in place of the scope, with the same link, because
    B is better on every point. B gives the same link and keeps the runner
    a child of the guest, with its exit code and its process-group stop.
    The "Runner scope" design call of this branch requires that the runner
    stays a child process of the guest.
  - A delete by the control plane when the sample reports stop, because a
    failed sample report never stops the runner
    (`docs/protocol/job-usage.md:140-141`).
- **Wider fix:** B. systemd keeps the link on each end of the guest unit,
  also when the guest cannot run its cleanup, and no guest code must keep
  it. D covers only a stop or a restart of the guest unit, and F depends
  on the thread that started the runner.
- **Previous examples:** `plans/follow-ups/boat-runner-stderr-drain.md:40-44`
  records the same lost control-group cleanup for the stderr helper of the
  runner.
- **Related:** `plans/follow-ups/boat-runner-stderr-drain.md`, the same code,
  and work that this fix changes. With B, the end of the guest unit stops
  the stderr helper again, as on `origin/main`. So the fix of this item
  updates the Reason (`:13-14`), the Background (`:40-47`), and the Left
  out line about a wait in `ExecStop=` (`:102-105`) of that item. Until
  its option A lands, the end of the guest can cut the last stderr line
  again. Neither item waits for the other.
- **Recommendation:** B. systemd ends the scope on each end of the guest
  unit, as `KillMode=control-group` did on `origin/main`, so B needs no
  new decision. D and G keep the behaviour of this branch, in which a
  crashed guest leaves its job running. That can be better for the job of
  the customer. But no recorded decision covers that change of the
  `origin/main` behaviour, so choose D or G only with the user.
  - Give the launcher the name of the guest unit, for example with the
    `%n` specifier in `ExecStart=` of both guest units.
  - Update the launcher test at
    `crates/ciproxy-guest/src/_tests_/platform_tests.rs:26`, the
    `systemd-run` stand-in at `image/check-runner-environment.py:52`, and
    the launcher paragraph at `crates/ciproxy-guest/README.md:45-51`.
  - State in the Out of memory section of `docs/protocol/job-usage.md`
    that an end of the guest unit also ends the runner scope. This branch
    also corrects that section before its merge, and its correction states
    no rule for a stop of the guest.
  - Update `plans/follow-ups/boat-runner-stderr-drain.md` in the same
    change.
  - Before an image release, prove on a real systemd host that a stop, a
    kill of the guest, and the normal end each end the scope. The
    behaviour of `BindsTo=`, `PartOf=`, and `After=` above comes from the
    systemd documentation, not from a run. Both Linux images use Ubuntu
    24.04, whose systemd supports these properties on a scope.
  - Add E only if a stop must also deliver the figures. With `After=`,
    systemd stops the scope first, but it sends SIGTERM to the guest as
    soon as the scope is empty, so the exit report can be cut.
- **Action:** fix after merge, apply B, because the change needs the real
  systemd image checks before an image release, and the defect needs an
  operator stop or a guest crash.
