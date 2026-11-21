# An unreported Boat start failure strands its job

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, the options
  check of `plans/follow-ups/boat-live-root-measurement-race.md`, as a
  problem outside the scope of the Boat locale fix. A review check on the
  same day narrowed it to starts that send no exit report.
- **Location:** `crates/ciproxy-control-plane/src/boat_snapshot.rs:58-60`,
  `crates/ciproxy-overflow/src/service/launch.rs:144-148`,
  `crates/ciproxy-scaler/src/lifecycle.rs:58-79`, and
  `crates/ciproxy-scaler/src/decision.rs:22-32`.
- **Reason:** Each option adds a report or a deadline that no requirement
  covers, and the branch that found it had approval only for the locale
  fix.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Guest:** the ciproxy program in the image that starts the runner of
    one job and reports the end of that runner.
  - **Exit report:** the guest's HTTPS callback with the sandbox token,
    which ends the start in the control plane.
  - **Start record:** the overflow backend's record of one Boat start. Its
    phase becomes `Running` when the launch command returns.
- **Background:**
  - The launch command starts `image/boat-run.sh` in the background and
    returns, at `crates/ciproxy-overflow/src/boat/launcher.rs:80`. That
    script writes the root-only `/run/ciproxy/registration.json` and
    queues `ciproxy-boat-start.service`. The backend then sets the start
    record to `Running`, at `launch.rs:144-148`.
  - The guest sends no ready report. It can send its exit report only
    after it reads its registration file, which holds the callback address
    and token (`crates/ciproxy-guest/src/boat.rs:35-66`). It removes the
    file at line 52, before it decodes it.
  - No exit report today:
    - `ciproxy-boat-start.service` fails or hangs before it starts the
      guest. `image/boat-start.sh:3` stops at the first error of the
      service refresh. A failed disk measurement only writes a warning, at
      lines 17-19, and line 20 starts the guest.
      `image/ciproxy-boat-start.service:12` sets no time limit.
    - The guest service does not start.
    - The guest cannot open, check, read, or decode its registration file,
      or cannot build its HTTPS client (`boat.rs:35-66`).
  - An exit report today:
    - A failed runner process start sends `RunnerExited` with no exit code
      (`crates/ciproxy-guest/src/service.rs:70-73`), and `boat.rs:81-92`
      posts it. `crates/ciproxy-guest/src/_tests_/service_tests.rs:163`
      tests this.
    - Each runner exit sends its code, also before the runner listens for
      jobs (`service.rs:156-158`).
  - Still no exit report after the guest can report: a runner that neither
    listens nor exits, a failed report, and a guest that stops before it
    sends the report (`service.rs:76-80`). The guest makes one POST with a
    30-second limit and no retry (`boat.rs:55-60` and `:83-91`).
  - The control plane accepts a report with no exit code
    (`crates/ciproxy-control-plane/src/boat_callback.rs:35`). It sets the
    start to `Deleting`, at
    `crates/ciproxy-store-pg/src/overflow/completion.rs:39-40`, which
    frees the job (`boat_snapshot.rs:55-57`). A second report for the same
    token gets 401 (`completion.rs:28` and `boat_callback.rs:43`).
  - Without a report, the control plane counts the runner as ready. That
    holds while its start record is `Running` and its runner row is absent
    or starting (`boat_snapshot.rs:58-60`). A waiting job claims a ready
    runner of its label, and else a starting one (`lifecycle.rs:58-79`).
    ciproxy starts a new runner only for a job that claims none
    (`decision.rs:22-32`).
  - While a waiting job needs the runner, ciproxy keeps the sandbox
    (`crates/ciproxy-overflow/src/service/inventory.rs:189`). No deadline
    ends a launched start. The sandbox lifetime is the backstop: 6 hours,
    or 2 hours on a Boat free trial (`docs/protocol/overflow.md:119-128`).
  - ciproxy sets no limit on new starts for one job, apart from refusals
    by Boat and the monthly spend limit (section "Spend limit" of
    `docs/protocol/overflow.md`).
  - No protocol doc says how soon ciproxy must replace a Boat start that
    fails after its launch.
- **Problem:**
  - A Boat start that fails after its launch and sends no exit report
    leaves a runner that ciproxy counts as ready. The waiting job claims
    it, so ciproxy starts no other runner for that job. The job waits
    until another runner of the label takes it, someone cancels it, or the
    sandbox lifetime ends.
  - One known case with no report came from a restore check with no
    registration on 2026-11-21. There, `ciproxy-boat-start.service`
    failed in its whole-root disk walk before it started the guest. That
    check had no waiting job, so it stranded no job. Cal's option F has
    since removed that walk: the start reads only `statvfs`, and a failed
    measurement only writes a warning. The
    [measurement record][measurement-record] holds the history.
  - Known cases with a report: on 2026-11-21, the guests of two real jobs
    with registrations, boxes 5 and 6, sent exit reports at 03:09:56 and
    03:18:24 UTC. ciproxy ended each start about 7 seconds later and
    deleted each sandbox. It made another start for the waiting job 14
    seconds after box 6 ended, and that start ended unsent.
  - A later diagnosis proved the cause of those exits. The optional NVM
    lookup on line 31 of Boat's `/etc/profile.d/nvm.sh` returned 1 under
    `set -e` of the runner entry. `image/boat-build-nvm.py` now guards that
    line in the Boat build. No real job with a registration has passed on
    a snapshot with that guard yet. The [exit record][exit-record] holds the
    history.
  - No stranded job has been seen yet. With the disk walk gone, no observed
    cause of a start with no report remains.
- **Impact if no action:** A customer job can wait for hours while its
  sandbox bills with no runner. A new sandbox for the job after the
  lifetime runs the same start again.
- **Root question:** How does a waiting job get another runner when a
  launched Boat start sends no exit report and its runner never becomes
  ready?
- **Options:**
  - **A.** Report a start failure that the guest cannot report. Three
    failures get this report: `ciproxy-boat-start.service` fails, the
    guest service does not start, or the guest fails before it can
    report. The sandbox then reports the failure on the callback with the
    sandbox token. Until the guest reads
    it, `/run/ciproxy/registration.json` holds the callback address and
    token. The control plane then ends the start and deletes the sandbox,
    and the next pass starts another runner. A lasting cause can then
    repeat paid starts and their fees until the spend limit stops them. A
    misses a hang and a lost report. The new report needs the user's
    approval. Effort: Medium.
  - **B.** Give each launched start a ready deadline. When a launched Boat
    runner has taken no job by the deadline while a job waits for it,
    ciproxy retires it and deletes the sandbox. The next pass starts
    another runner. It covers each cause, also a hang. A lasting cause can
    repeat paid starts and their fees until the spend limit stops them.
    The user must approve the deadline and choose its value. A slow but
    healthy start past the deadline gets replaced. Effort: Medium.
  - **C.** Count a launched Boat runner as starting until the guest
    reports ready, and bound that wait. The guest reports ready on its
    callback after the runner listens for jobs. The starting runner still
    keeps the job that started it. When no ready report comes by a
    deadline, ciproxy ends the start and deletes the sandbox. A lasting
    cause can repeat paid starts and their fees until the spend limit stops
    them. It needs a new report and a new deadline, in the guest, the
    callback, and the control plane. Effort: Large.
- **Left out:**
  - A new report for a failed runner start or a runner exit, because the
    guest already reports both, and a second report gets 401.
  - C with no deadline, because a waiting job also claims a starting Boat
    runner, so the job stays stranded.
  - A retry of the start inside the sandbox, because a lasting cause fails
    each attempt, and ciproxy still learns nothing.
  - A shorter sandbox lifetime, because it also ends healthy jobs, which
    can run for 6 hours.
  - A fix of one cause, such as the disk walk that option F removed,
    because the other causes stay.
- **Wider fix:** C. Ready then means a runner that reported ready, and its
  deadline ends each start that never gets there. A and B end the stranded
  start but keep the inferred ready state.
- **Previous examples:** On a machine, the agent reports `RunnerReady`
  (`crates/ciproxy-control-plane/src/runtime_agent_event.rs:26`), and a
  waiting job claims a starting machine runner only while its machine is
  connected (`lifecycle.rs:62-63`).
- **Related:**
  - The [measurement record][measurement-record] is a historical example of
    a start that sends no report. `image/boat-start.sh` and
    `image/measure.py` now remove that cause, not this class.
  - The [exit record][exit-record] holds two runner exits that did send
    their reports, so this item does not explain them.
    `image/boat-build-nvm.py` now guards their cause.
- **Recommendation:** None yet. No option is clearly best, because no
  requirement says how soon ciproxy must replace a failed start. A and B
  combine well: A ends a failure that the guest cannot report at once, and
  B is the backstop for a hang or a lost report. C is the wider fix, at the
  largest effort.
- **Action:** ask the user, because each option adds behaviour at a
  boundary that no requirement covers, and no option is clearly best.

[measurement-record]: https://github.com/Firna-AI/ciproxy/blob/34bc620945cb5a388af949373c209d8e97bcbfb6/plans/follow-ups/boat-live-root-measurement-race.md
[exit-record]: https://github.com/Firna-AI/ciproxy/blob/34bc620945cb5a388af949373c209d8e97bcbfb6/plans/follow-ups/boat-runner-before-job-exit.md
