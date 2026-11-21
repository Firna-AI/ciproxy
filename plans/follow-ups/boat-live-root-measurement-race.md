# Boat start fails when a provider file vanishes

- **Severity:** High. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, the first
  restore check of the Boat snapshot `2026-11-21-boat-locale-5a493ed6`.
  An options check on the same day added F and G and changed the
  recommendation from E to F.
- **Location:** `image/boat-start.sh:17`, `image/measure.py:12-16`, and
  `image/measure.py:30-31`.
- **Reason:** The user approved only the Boat locale fix for the branch
  that found it. The user asked to get each other live failure with its
  cause and options.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Boat start:** the root service `ciproxy-boat-start.service`. It runs
    `image/boat-start.sh`, which refreshes a restored sandbox and then
    starts the guest.
  - **Guest:** the ciproxy program in the image that starts the runner of
    one job and reports the end of that runner.
  - **Walk:** a `du` scan that reads the metadata of each file below a
    folder to add up their sizes.
- **Background:**
  - Each Boat job runs in a sandbox that Boat restores from the saved Boat
    snapshot. The launcher launches the guest of each sandbox once. A
    marker folder stops a second launch, at
    `crates/ciproxy-overflow/src/boat/launcher.rs:80`.
  - The Boat start runs once, with no time limit and no retry, as
    `image/boat.md:164` says. `image/boat-start.sh` uses
    `set -euo pipefail`. Its service refresh must stay strict, because the
    runner takes no job before Docker and containerd run the snapshot
    versions (`docs/protocol/agent-images.md:133-138`).
  - After the refresh, line 17 runs
    `python3 /opt/ciproxy-image/measure.py / --job`. Line 18 then starts
    `ciproxy-boat-guest.service`.
  - `image/measure.py` walks the whole root filesystem twice with
    `du --summarize --one-file-system --block-size=1`, once with
    `--apparent-size`. It reads the free space with `os.statvfs`, which
    walks no folder. It prints one JSON object to the journal.
  - The walk came from the earlier launch in a `chroot`, which walked the
    image tree under `/opt/ciproxy/rootfs`. Boat did not change that tree.
    Commit `873f47c2` moved the launch onto Boat's own system, so the walk
    now reads Boat's live root.
  - No program reads that JSON. Operators read only the free bytes in the
    journal, as `image/boat-smoke.md:184-185` and `:208`,
    `image/README.md:165-166`, and `image/boat.md:175` say.
  - No protocol, doc, or program needs the Boat installed size. Scheduling
    uses the logical image root length, not installed file bytes
    (`docs/protocol/job-routing-capacity.md:31-32`). The Boat root size
    comes from the machine release manifest, at
    `crates/ciproxy-control-plane/src/boat_start.rs:61-67` and
    `crates/ciproxy-control-plane/src/config.rs:117-121`.
  - Only `image/boat-start.sh` uses `--job`. The machine export runs the
    same script without it, at `image/export.sh:29` and `:38`. It writes
    `installed-size.json` and `measurement.json`. That walk reads a fixed
    tree, so `measure.py` stays strict there.
  - Boat keeps changing its own files after a restore, also after the
    sandbox is ready. On the restored `large` sandbox, `df` showed
    82,365,460,480 bytes in use, and each walk reads the metadata of all
    those files. In the successful rerun, Docker became active at 02:25:29
    UTC, and the guest logged its error at 02:25:38 UTC. The two walks ran
    in that gap.
  - Each job start walks the root soon after its restore, while Boat still
    changes its files. The backend launches in one pass after it adopts the
    sandbox, at `crates/ciproxy-overflow/src/service/launch.rs:57-148`.
    The failed start of 2026-11-21 came later than that, after other
    restore checks. So the race needs no unusual input, timing, or setup.
  - A failed start strands its job. The backend marks a start as running
    when the launch command returns, at `launch.rs:144-148`. The control
    plane counts such a runner as ready while no report says otherwise, at
    `crates/ciproxy-control-plane/src/boat_snapshot.rs:58-60`. A waiting
    job claims a ready runner of its label, at
    `crates/ciproxy-scaler/src/lifecycle.rs:58-79`. ciproxy then starts no
    other runner for that job, at `crates/ciproxy-scaler/src/decision.rs:22-32`.
  - The test fixture replaces `python3` with a stub that exits 0, at
    `image/_checks_/boat_fixture.py:71-73`. So no check runs `measure.py`
    in the Boat start now. A regression needs the real `measure.py` and a
    `du` that exits 1. The repository gate skips the image checks, as
    `plans/follow-ups/image-checks-in-gate.md` records.
- **Problem:**
  - On 2026-11-21, Boat removed `/var/lib/ascii-lazy/active` during the
    walk of the first Boat start after a restore. GNU `du` exited 1, so
    `subprocess.check_output` raised `CalledProcessError`.
  - The Boat start then failed before it started the guest. Its journal
    gave these lines. The excerpt leaves out some traceback lines.

    ```text
    du: cannot access '/var/lib/ascii-lazy/active': No such file or directory
      File "/opt/ciproxy-image/measure.py", line 30, in main
        "installed_apparent_bytes": du(rootfs, True),
    subprocess.CalledProcessError: Command '['du', '--summarize', '--one-file-system', '--block-size=1', '--apparent-size', '/']' returned non-zero exit status 1.
    ciproxy-boat-start.service: Main process exited, code=exited, status=1/FAILURE
    ciproxy-boat-start.service: Failed with result 'exit-code'.
    ```

  - The snapshot had built with 82 passed provisioners and 5 documented
    skips. `image/measure.py` and `image/boat-start.sh` did not change in
    that branch. Of the 23 restore checks, two failed: the Boat start and
    the guest.
  - Then `python3 image/boat.py run SANDBOX_ID`, with no registration, ran
    the unchanged Boat start again on the same sandbox. The walk passed.
    The guest started and ended with its missing-registration error, as
    it does without a registration. All 23 restore checks then passed,
    with PostgreSQL and Docker.
  - The saved evidence proves one failed first start and one successful
    unchanged rerun on the same sandbox. It gives no failure rate.
- **Impact if no action:**
  - When the race hits a job start, no runner starts in that sandbox.
    ciproxy counts the launched runner as ready, so it starts no other
    runner for that job.
  - The job waits and the sandbox bills until another runner of the label
    takes the job or someone cancels it. Otherwise Boat ends the sandbox
    at its lifetime: 6 hours, or 2 hours on a Boat free trial.
  - A new sandbox for the job then runs the same start again.
- **Root question:** How does each Boat job reach its runner while Boat
  still changes its files after a restore, with its free job disk still
  recorded?
- **Options:**
  - **A.** Keep the strict Boat start. Add an operator step to
    `image/boat-smoke.md`: after a start that fails on a vanished file,
    run `boat.py run` again on the same sandbox. This worked once on
    2026-11-21, in a check with no registration. For a registered run,
    give the registration file again: a run with no file removes the
    delivered registration, at `image/boat-run.sh:12`. So it is no
    recovery for a real job, and a customer job gets no automatic rerun,
    because each sandbox launches its guest once. Effort: Small.
  - **B.** Let the Boat start go on after a failed measurement. In
    `image/boat-start.sh`, a failed `measure.py` prints a warning to the
    journal, and the guest still starts. `measure.py` stays strict for the
    machine export. Each start still walks the whole root, and a failed
    walk loses the free bytes of that start. The walks still delay each
    guest start by several seconds. Effort: Small.
  - **C.** Start the guest before the measurement: swap lines 17 and 18
    of `image/boat-start.sh`. The free bytes then come after the guest
    starts, and the walk competes with the job for disk reads. A failed
    walk still marks the Boat start as failed. Effort: Small.
  - **D.** Measure only the free space at each start. In `--job` mode,
    `measure.py` skips both walks and prints only its `statvfs` fields.
    The release evidence then has no installed size of the Boat snapshot.
    No protocol, doc, or program needs that size. D also removes the walk
    time from each guest start. Effort: Small.
  - **E.** Do D, and record the installed size once at build time. The
    Boat build runs `measure.py /` after its provisioners, and keeps the
    result with its reports. Update `image/boat.md`, `image/boat-smoke.md`,
    and the report list of `image/boat-build-steps.md`. No requirement or
    reader needs the Boat installed size. As written, the strict
    build-time walk adds a new way for the Boat build to fail. The
    always-on rules leave a new build error to the user. Effort: Medium.
  - **F.** Do D, and make a failed measurement only a warning:
    - In `--job` mode, `measure.py` prints only its `statvfs` fields.
    - When `measure.py` fails, `image/boat-start.sh` prints a warning to
      the journal and then starts the guest. The service refresh stays
      strict, and `measure.py` stays strict outside `--job`.
    - A failed measurement then loses only the free bytes of that start.
    - Add the regression to `image/check-boat-run.py`. Update
      `image/boat.md:175` and `image/checks.md`.
    - Effort: Small.
  - **G.** Take the measurement out of the Boat start:
    - Remove line 17 of `image/boat-start.sh`. Remove the `--job` mode of
      `measure.py`, because nothing calls it after that.
    - Before `run`, the smoke reads the free bytes through the Boat command
      API with `df --block-size=1 /`. Each smoke then needs that operator
      step. `image/boat.md` and `image/boat-smoke.md` change.
    - Effort: Small.
- **Left out:**
  - A retry of the walk or of the whole start unit, because each attempt
    can meet the same race, and F removes the race.
  - An exclude list of Boat's folders, such as `/var/lib/ascii-lazy`,
    because Boat does not document them and can add others.
  - A wait for the end of Boat's restore work before the walk, because
    Boat gives no known signal for that end.
  - A tolerant walk in each mode of `measure.py`, because the machine
    export must fail on a wrong measurement.
  - A report of a failed start, or a start deadline in the control plane,
    because it does not answer this root question. It replaces the sandbox
    after the failure, so each race still costs a new sandbox and a delay.
    `plans/follow-ups/boat-launched-runner-readiness.md` holds it.
  - Free bytes from the guest program, because F is better on each point
    that a requirement or a reader names.
- **Wider fix:** F. It keeps the guest start for each failure of the
  measurement, also when a later change adds a step to it. D and E remove
  only the walk, which is the detail that failed.
- **Previous examples:** Commit `5a493ed6` kept the locale of Boat's
  controller usable while Boat runs `locale-gen` after a restore. There
  too, Boat changed its files after the sandbox was ready.
- **Related:** `plans/follow-ups/boat-launched-runner-readiness.md` holds
  the wider class: any failed Boat start strands its waiting job. This
  item removes one cause of such a failure. Fix the two items apart.
- **Recommendation:** F. This replaces the first recommendation, E. F
  removes the walk, which nothing needs. It also makes each later
  measurement failure a journal warning, so the measurement cannot stop a
  customer job again. E adds a build step and a report that nothing needs.
  D is the smaller choice if the start must stay strict on its
  measurement.
- **Action:** ask the user, because the user approved only the locale fix
  for that branch, and asked to see each other live failure with its
  options.
