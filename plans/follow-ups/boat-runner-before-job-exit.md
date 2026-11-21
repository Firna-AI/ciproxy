# Boat runners exit before taking a job

- **Severity:** High. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, the official
  overflow and spend-limit live checks of the Boat snapshot
  `2026-11-21-boat-locale-5a493ed6`.
- **Location:** `image/runner-entry.sh:3-20`,
  `crates/ciproxy-guest/src/process.rs:63-87`, and
  `crates/ciproxy-guest/src/process.rs:133-139`.
- **Reason:** The exact cause is not proved, and its diagnosis needs a new
  paid sandbox beyond the six-sandbox limit of the task that found it.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Guest:** the ciproxy program in the image that starts the runner of
    one job and reports the end of that runner.
  - **Runner entry:** `image/runner-entry.sh`, installed as
    `/usr/local/libexec/ciproxy-runner`. It prepares the shell of the
    `runner` account and then starts the listener.
  - **Listener:** `Runner.Listener`, the stock GitHub runner process. It
    writes `Runner_*.log` files in `/opt/actions-runner/_diag`.
  - **Registration value:** the one-time GitHub runner configuration that
    the guest passes in `ACTIONS_RUNNER_INPUT_JITCONFIG`. A dummy value is
    not a registration.
- **Background:**
  - The call path: `ciproxy-boat-guest.service` runs the guest. The guest
    runs `/usr/sbin/runuser --user runner -- /usr/local/libexec/ciproxy-runner`
    with a cleared environment, at
    `crates/ciproxy-guest/src/process.rs:63-87`.
  - The guest sends the runner's stderr to `Stdio::null()`, at line 83.
    It reads stdout only for `: Listening for Jobs`, at lines 133-139, and
    drops each other line. So no startup error of the runner entry is
    kept.
  - `image/runner-entry.sh` runs `set -eo pipefail` at line 3, before it
    sources `/etc/profile` at line 5, `/etc/environment`, and
    `$HOME/.profile`. A failing command in those files ends the entry.
  - In Boat mode, the guest passes no assignment socket. So line 17 copies
    the stock Worker DLL, and line 20 starts the listener with `exec`.
  - `/etc/profile` reads Boat's `/etc/profile.d/nvm.sh`, which the Boat
    build keeps, as `image/boat-developer-tools.json:86-87` says. The
    build inventories of this snapshot and of the snapshot before it record
    the same SHA-256 for it:
    `ef5693062b54c4fd3066c21852848b4acfdfe58e071b56060ea59753da1d4158`.
  - In a local check, the locale profile of the snapshot,
    `/etc/profile.d/ciproxy-boat-locale.sh`, changes nothing for the
    runner. `image/_checks_/boat_locale.py` sources it with `bash -e` for
    `HOME=/home/runner`, and requires exit 0, no stderr, and an unchanged
    environment.
  - Two candidates, neither proved:
    - A saved probe of an earlier Boat build records line 31 of
      `/etc/profile.d/nvm.sh` as
      `__ascii_nvm_bin="$(__ascii_nvm_path 2>/dev/null)"`. An offline
      Bash proof ran that exact line under `set -e` with
      `HOME=/home/runner`. A controlled `__ascii_nvm_path` that returned 1
      ended the shell with status 1, and a return of 0 went on. The real
      function body and the runner's `NVM_DIR`, default alias, and path
      state are unknown. A public code search found no `__ascii_nvm_path`.
    - The stock Worker DLL copy at line 17. The build makes that file and
      sets its owner, and no evidence shows that this step failed.
  - The saved-snapshot metadata endpoint gives no file bytes. For the
    deleted box 6, a process read gave 404 and a log read gave 403. No
    retained artifact holds the full bytes of `/etc/profile.d/nvm.sh`.
- **Problem:**
  - On 2026-11-21, two real Boat jobs on `large` sandboxes failed: the
    official overflow check on box 5, `bx_748nk3ay`, and the resumed
    spend-limit check on box 6, `bx_gbsxpayp`. Box 6 was its own requested
    spend case, not a paid retry of box 5.
  - In both, the start reached the guest, the guest sent its exit report,
    and ciproxy deleted the sandbox. Neither runner took its GitHub demand
    job. The locale, PostgreSQL, and Docker steps of the job never ran.
    - Box 5: start at 03:09:07 UTC, exit report at 03:09:56, end at
      03:10:03. Its guest, cache sidecar, `boat-run.sh`, and
      `boat-start.sh` matched the release fingerprints.
    - Box 6: start at 03:17:37 UTC, exit report at 03:18:24, end at
      03:18:30.
  - 17 redacted samples of box 6, from 03:18:02 to 03:18:24 UTC, show
    these facts:
    - `ciproxy-boat-start.service` ended with `success`, and
      `ciproxy-boat-guest.service` was running.
    - The journal gave these two lines, one directly after the other:

      ```text
      pam_unix(runuser:session): session opened for user runner(uid=1001) by (uid=0)
      pam_unix(runuser:session): session closed for user runner
      ```

    - No `Runner_*.log` appeared in `/opt/actions-runner/_diag`.
    - The evidence does not show whether the entry or the listener
      failed before the first diagnostic log.
  - These checks passed:
    - The restore checks of the snapshot.
    - Empty stderr of the first controller command and of the pinned
      client helpers.
    - The own-machine hold jobs.
    - The 75-second block at a positive spend limit.
  - For both starts, the test adapter had set a one-cent spend limit as a
    fence against more paid starts. Idle retirement does not read that
    limit. For box 6, the operator restored the original cap of 94 cents
    at 03:17:41 UTC, 4 seconds after the start and before the runner
    started. Box 6 still exited. So the fence is not a proved cause.
- **Impact if no action:**
  - Overflow jobs on this snapshot can end before their runner takes a
    job, as both real jobs did.
  - The job still waits after each exit, so ciproxy can start another
    paid sandbox for it. In the spend-limit check, it made such a start
    14 seconds after box 6 ended, and that start ended unsent.
- **Root question:** How does ciproxy find why the Boat runner exits before
  it takes its first job?
- **Options:**
  - **A.** Get the full bytes of `/etc/profile.d/nvm.sh` with the recorded
    SHA-256, and the runner's effective `NVM_DIR`, default alias, and path
    state. Then prove the candidate offline. It needs no new VM. No saved
    artifact or Boat API read gives these bytes, so Boat or the user must
    supply them. Effort: Small.
  - **B.** In a separate task with a new budget of one sandbox, restore
    this published snapshot, and delete the sandbox at the end. No step
    uses a real registration, a real server, or a key. The user must
    approve that task first. Effort: Small. The steps:
    1. Capture the bytes of `/etc/profile.d/nvm.sh`, and check them
       against the recorded SHA-256. Record the runner's effective
       `HOME`, `NVM_DIR`, default alias, and path state.
    2. Run the unchanged runner entry as the guest does, through
       `runuser --user runner` with a cleared environment and no
       registration value. Set a Bash `ERR` trap through `BASH_ENV` that
       prints only the file, the line, and the exit status. Print no
       command text, environment value, or trace.
    3. The `ERR` trap does not run when the `${...:?}` check at line 19
       ends the shell. So also record the exit status of the process,
       and, as yes or no, whether stderr holds the constant text
       `JIT registration required`. Print no other stderr text.
    4. When the entry stops at line 19, run it once more with a known
       dummy value in `ACTIONS_RUNNER_INPUT_JITCONFIG`. That value is no
       registration, names no real server, and holds no key. Line 20
       then starts the listener, which cannot register or take a job.
       Record the exit status, and whether a `Runner_*.log` file
       appears.
    - The outcomes:
      - An exit before line 19 shows the file and the line of a failing
        command in this probe. It can reproduce a candidate cause.
      - A stop at line 19 shows only that the earlier lines passed in
        this probe. It does not prove the cause of the original exit.
      - A listener exit with no log can reproduce a candidate failure in
        the listener start.
      - A listener that writes its log and then rejects the dummy value
        passed its start in this probe. The original cause then stays
        open.
    - The known differences from the real starts: the probe has no real
      registration, and it runs later after the restore. Also,
      `ciproxy-boat-start.service` does not run first. The real entry of
      box 6 ran directly after that service, about 45 seconds after the
      sandbox started. So one sandbox cannot guarantee the exact cause.
  - **C.** In a separate product change, add bounded, redacted diagnostics
    of the runner start and exit. An example is the last stage that the
    entry reached and its exit status, with no command output. Then
    rebuild the snapshot and test it. It helps each later failure, but it
    needs a new snapshot and still a live run. Effort: Medium.
- **Left out:**
  - A guessed NVM fix, such as a guard around line 31, because no
    evidence proves that this line fails. Such a fix may not end the exit.
  - Acceptance of stderr text in the Boat client, because this exit is no
    client rejection: the helpers passed, and the guest started.
  - A new start timeout, gate, or test for the class, because none of
    them finds the cause.
  - More real jobs, because each costs a paid sandbox and, with stderr
    dropped, shows the same exit with no cause.
- **Wider fix:** C. It makes each later exit before the listener visible.
  B can show where this probe exits, but it may not find the cause of the
  observed exit.
- **Previous examples:** Commit `5a493ed6` fixed the effect of a Boat
  profile line on the commands of Boat's controller. There too, a profile
  of Boat's base ran inside a ciproxy start path.
- **Related:** `plans/follow-ups/boat-launched-runner-readiness.md`
  covers starts that send no exit report. Both runners here did send
  their reports, so that item does not explain these exits.
- **Recommendation:** B. The candidates are not proved, and the original
  stderr was dropped. So one bounded probe on the real snapshot is the
  cheapest next step. It can reproduce a candidate cause or rule out the
  early setup, and the original cause can stay open. B was blocked in
  the task that found it, because the user limited that task to six
  sandboxes, and all six were used and deleted.
- **Action:** ask the user, because B needs a new paid sandbox beyond the
  six-sandbox limit that the user set for the task that found it.
