# A failed Boat start strands its waiting job

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, the options
  check of `plans/follow-ups/boat-live-root-measurement-race.md`, as a
  problem outside the scope of the Boat locale fix.
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
  - **Launch:** the helper command that sets off the Boat start of a
    sandbox, after the transfer of the runner registration.
  - **Start record:** the overflow backend's record of one Boat start. Its
    phase becomes `Running` when the launch command returns.
- **Background:**
  - The launch command starts `image/boat-run.sh` in the background and
    returns, at `crates/ciproxy-overflow/src/boat/launcher.rs:80`. That
    script queues `ciproxy-boat-start.service` with `--no-block`. So the
    launch returns before the guest starts. The backend then sets the
    start record to `Running`, at
    `crates/ciproxy-overflow/src/service/launch.rs:144-148`.
  - In a Boat sandbox, the guest sends only its exit report, on an HTTPS
    callback with a token for that sandbox
    (`crates/ciproxy-guest/README.md:54-65`). It sends no ready report.
    When the start fails before the runner starts, the sandbox sends
    nothing.
  - The control plane counts a Boat runner as ready when its start record
    is `Running` and its runner row is absent or starting, at
    `crates/ciproxy-control-plane/src/boat_snapshot.rs:58-60`.
  - A waiting job claims a ready runner of its label, and else a starting
    one, at `crates/ciproxy-scaler/src/lifecycle.rs:58-79`. ciproxy starts
    a new runner only for a job that claims none, at
    `crates/ciproxy-scaler/src/decision.rs:22-32`.
  - While a waiting job still needs the runner of a sandbox, ciproxy keeps
    that sandbox, at `crates/ciproxy-overflow/src/service/inventory.rs:189`.
    No deadline ends a launched start. The sandbox lifetime is the
    backstop: 6 hours, or 2 hours on a Boat free trial
    (`docs/protocol/overflow.md:119-128`).
  - No protocol doc says how soon ciproxy must replace a Boat start that
    fails after its launch.
- **Problem:**
  - A Boat start that fails after its launch returns leaves a runner that
    ciproxy counts as ready. The waiting job claims it, so ciproxy starts
    no other runner for that job.
  - The job then waits until another runner of the label takes it,
    someone cancels it, or the sandbox lifetime ends.
  - One known cause: on 2026-11-21, a Boat start failed in a restore check
    when Boat removed a file during a disk measurement, as
    `plans/follow-ups/boat-live-root-measurement-race.md` records. That
    check had no registration and no waiting job, so it stranded no job.
    No stranded job has been seen yet. That item grades its cause Common,
    because each job start runs the measurement soon after its restore.
- **Impact if no action:** A customer job can wait for hours while its
  sandbox bills with no runner. A new sandbox for the job after the
  lifetime runs the same start again.
- **Root question:** How does a waiting job get another runner when a
  launched Boat runner never becomes ready?
- **Options:**
  - **A.** Report a failed start. When the Boat start or the guest fails
    before the runner starts, the sandbox reports the failure on its
    callback, with its sandbox token. The callback address and token stay
    in the root-only registration file until the guest starts the runner.
    The control plane then ends the start and deletes the sandbox, and the
    next pass starts another runner. It misses a start that hangs or
    cannot reach the control plane. The new report needs the user's
    approval. Effort: Medium.
  - **B.** Give each launched start a ready deadline. When a launched Boat
    runner has taken no job by the deadline while a job waits for it,
    ciproxy retires it and deletes the sandbox. The next pass starts
    another runner. It covers each cause, also a hang. The user must
    approve the deadline and choose its value. A slow but healthy start
    past the deadline gets replaced. Effort: Medium.
  - **C.** Count a launched Boat runner as starting until the guest
    reports ready, and bound that wait. The guest reports ready on its
    callback after the runner listens for jobs. The starting runner still
    keeps the job that started it. When no ready report comes by a
    deadline, ciproxy ends the start and deletes the sandbox. It needs a
    new report and a new deadline, in the guest, the callback, and the
    control plane. Effort: Large.
- **Left out:**
  - C with no deadline, because a waiting job also claims a starting Boat
    runner, so the job stays stranded.
  - A retry of the start inside the sandbox, because a lasting cause fails
    each attempt, and ciproxy still learns nothing.
  - A shorter sandbox lifetime, because it also ends healthy jobs, which
    can run for 6 hours.
  - A fix of one cause, such as the disk measurement, because
    `plans/follow-ups/boat-live-root-measurement-race.md` holds it, and the
    other causes stay.
- **Wider fix:** C. Ready then means a runner that reported ready, and its
  deadline ends each start that never gets there. A and B end the stranded
  start but keep the inferred ready state.
- **Previous examples:** On a machine, the agent reports `RunnerReady`
  (`crates/ciproxy-control-plane/src/runtime_agent_event.rs:26`), and a
  waiting job claims a starting machine runner only while its machine is
  connected (`crates/ciproxy-scaler/src/lifecycle.rs:62-63`).
- **Related:** `plans/follow-ups/boat-live-root-measurement-race.md`
  records one cause of a failed Boat start. Its fix removes that cause,
  not this class. Fix the two items apart.
- **Recommendation:** None yet. No option is clearly best, because no
  requirement says how soon ciproxy must replace a failed start. A and B
  combine well: A ends a reported failure at once, and B is the backstop
  for a start that cannot report. C is the wider fix, at the largest
  effort.
- **Action:** ask the user, because each option adds behaviour at a
  boundary that no requirement covers, and no option is clearly best.
