# Missing-helper check misses unfiltered journal output

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-runner-start`, review round
  3, finding 8. Its finding check confirmed it, kept the options, and
  added the steps of C.
- **Source:** Review.
- **Category:** Test.
- **Location:** `image/check-boat-runner.py:212-214` and `:236-241`,
  `image/check-boat-runner.py:257-272`, `image/boat-runner-entry.sh:6`,
  `image/boat-build-steps.md:188-189`, and
  `image/boat-runner-checks.md:38-44`.
- **Reason:** Both fix rounds of the branch are used, and the current
  fallback discards stderr, so only a later change can expose it.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Boat entry:** `image/boat-runner-entry.sh`, installed at
    `/usr/local/libexec/ciproxy-runner`, which sends the runner's stderr to
    the stderr helper and then starts the runner.
  - **Stderr helper:** `image/boat-runner-stderr.py`, which sends a
    bounded, redacted copy of the runner's stderr to the sandbox journal
    through `systemd-cat`.
  - **Fallback:** `|| exec /usr/bin/cat` at `image/boat-runner-entry.sh:6`,
    which reads and discards the stderr when the helper ends with an error.
  - **Recording writer:** the stand-in for `systemd-cat` that
    `recording_journal` in `image/check-boat-runner.py:52-58` installs,
    which copies its input to `/observations/journal`.
- **Background:**
  - The orchestrator's call for this work: keep a bounded, redacted copy
    of runner stderr in the sandbox journal, with no protocol change.
    `image/boat-build-steps.md:188-189` states the fallback: "When the
    helper or the journal fails, the runner goes on, and its stderr is
    dropped."
  - The redaction is the requirement. An empty journal after a helper
    failure is the current design, not a requirement. So "no journal
    output" is the right expected value for the current code.
  - `image/boat-runner-entry.sh:4-7` starts the stderr helper as a Bash
    coprocess, with its stdout and stderr on `/dev/null`. When Python ends
    with an error, the fallback runs. So the current fallback never starts
    the journal writer, and it discards the stream correctly.
  - The helper gives its journal writer no environment, at
    `image/boat-runner-stderr.py:112-116`. The build guide states this rule
    at `image/boat-build-steps.md:180-181`.
  - `test_entry_redirects_before_profiles_and_survives_logging_failures`
    at `image/check-boat-runner.py:206-241` runs the real Boat entry in two
    subcases. One has a failed journal writer. The other deletes the
    installed helper and installs the recording writer, at lines 212-214.
    In both, the fixture listener writes a 160 KiB burst to stderr, more
    than a pipe buffer holds, at lines 228-235. Both check only the exit
    status 23, the listener stdout, and an empty original stderr, at lines
    236-241. No line reads the journal file.
  - The working-writer case at lines 252-272 passes the write end of a
    completion pipe to the entry. At lines 257-268, it waits until the
    helper and the writer have closed that pipe, with 30-second hang
    guards. Then it asserts the exact journal bytes, at line 272.
  - No other check reaches the fallback. The helper case at line 156 and
    the error-hook case at line 243 do not run the Boat entry. The
    failed-writer subcase, the working-writer case, and the restore smoke
    of `image/boat-smoke.md` all run a working helper.
  - A read of the code traces this mutation. Nobody ran it.
    - On line 6 of the Boat entry, replace `exec /usr/bin/cat` with this
      command:

      ```sh
      exec /usr/bin/env -i /usr/bin/systemd-cat --identifier=ciproxy-boat-runner-stderr
      ```

    - Python fails on the absent file, and the fallback starts the
      recording writer. `env -i` removes the registration variable, so the
      writer passes its check at `image/check-boat-runner.py:54`.
    - Its `cat` copies the whole stream to the journal file and drains the
      160 KiB burst. The listener ends with status 23, so the subcase
      passes.
    - Without `env -i`, the writer exits with status 33 at once. The burst
      then breaks on a closed pipe, and the status assertion fails. So the
      route that passes is the one that copies the empty environment of
      the helper.
  - The risk is real. The fix that
    `plans/follow-ups/boat-runner-stderr-drain.md` recommends rewrites
    lines 4-12 of the Boat entry, the block with the fallback. That item
    already leaves out "a direct journal stream as the runner's stderr",
    so the idea is known in this code. A later change that adds a record
    for a failed helper could take the same route.
  - The check runs by hand as root, outside the repository gate, as
    `image/boat-runner-checks.md:13-17` and
    `plans/follow-ups/image-checks-in-gate.md` say. So a fix protects only
    the runs where someone runs the check.
  - The Boat build bundle copies all of `image/`, at
    `image/boat_build_bundle.py:50`. So a fix of this check also changes
    the definition folder of the next Boat build.
  - The orchestrator's call on 2026-11-21 deferred this fix under the
    review rule. Both fix rounds of the branch were used. After the second
    fix round, a confirmed finding that is not critical gets
    `fix after merge`. A critical finding is a security problem, a data
    loss, or a High product bug with the likelihood Common. This test gap
    is none of them.
- **Problem:** The missing-helper subcase never checks the file of the
  recording writer. So a changed fallback that sends the raw stream to the
  journal writer with an empty environment still passes, with no filter.
- **Impact if no action:** A later change of the fallback can put
  unfiltered stderr, which can hold a registration value, in the sandbox
  journal while the check passes. The current fallback discards the stream
  correctly.
- **Root question:** How can the check show that a missing filter does not
  put unfiltered output in the journal?
- **Options:**
  - **A.** In the missing-helper subcase, pass a completion pipe to the
    entry, as the working-writer case does. Wait until the fallback
    processes close it. Then require that the recording writer made no
    journal file. It keeps the fix in this subcase, and it copies the pipe
    code a second time. Effort: Small.
  - **B.** Add a separate missing-helper case with one fixture line, and
    require no journal output there. Keep the burst subcase for the drain.
    It isolates the journal assertion, but it starts the entry a second
    time for the same fault and the same requirement. It also adds a full
    entry setup to a file of 276 lines. Effort: Small.
  - **C.** Share one entry runner for each Boat entry case with a recording
    writer. The runner waits for completion and takes a required journal
    expectation, with "none" for no journal file. Keep the existing output
    and exit checks. No entry case can then install the recording writer
    without a journal expectation. Effort: Small. The steps:
    1. Give the runner the root, the registration, the stderr target, and
       the expected journal bytes. Use "none" for no journal file.
    2. Let the runner install the recording writer by default. Let the
       failed-writer subcase give its own writer, with "none".
    3. Move the completion pipe and the two 30-second hang guards of lines
       257-268 into the runner. Do not assert elapsed time.
    4. Give the missing-helper subcase "none". Keep its exact tuple: status
       23, the listener stdout, and an empty original stderr. Keep the
       exact bytes of the working-writer case.
    5. In a scratch copy, change the fallback to
       `exec /usr/bin/env -i /usr/bin/systemd-cat`. See the missing-helper
       subcase fail on its journal assertion. Then run the real check 10
       times.
    6. In the logging-failure item at `image/boat-runner-checks.md:38-44`,
       say that a missing helper also puts nothing in the journal.
    7. When a later change adds a fixed record for a failed helper, change
       only the expected journal of that subcase.
- **Left out:**
  - A journal check when the entry ends, because it misses a route that
    buffers the stream before it starts the writer. For a direct route,
    the burst already forces the file to exist, so the pipe costs only a
    few lines.
  - A live journal, because A is better on every point. A paid restore
    shows the same bytes only after a build. A local systemd journal needs
    a systemd environment.
  - Removal of the unused recording writer from the missing-helper
    subcase, so that a journal route breaks the drain. It fails the goal,
    because a route that tries the writer and then drains with `cat` still
    passes.
  - A rule that the whole stream must reach the private `/dev/null`,
    because it checks how the fallback discards, not what reaches the
    journal. The fixture creates that file as a regular file, at
    `image/check-boat-runner.py:84-85`. A correct fallback that drains
    without a write then fails the rule.
  - A check of the text of the fallback line, because it tests a copied
    constant, which only a new decision can fail.
  - A required expectation in the shared root runner `in_runner_root`,
    because one of its four callers has no journal writer. Another checks
    limits, not exact bytes.
  - A fixed journal record for a failed helper, because it is a product
    change that answers another question. It would only change the
    expected value of each option.
  - A case at the guest level, because a test goes at the lowest level
    that fails. `plans/follow-ups/boat-runner-install-check.md` also
    chose a check without the guest.
  - A step in the repository gate or the build, because each one is a new
    gate, which needs the user.
- **Wider fix:** C. Each Boat entry case with a recording writer states
  its final journal result through the same runner, so no case can leave
  it out. The testing rule to break each path on purpose covers the wider
  class, and it is in force.
- **Previous examples:** Commit `88df2d91` added the working-writer case,
  because no case checked a successful journal delivery. It asserts the
  exact journal bytes at `image/check-boat-runner.py:272`. Commit
  `5ec6ec2c` added the missing-helper subcase, because no case made the
  helper itself fail. So this gap is the third case in this file that
  misses one way its path can break.
- **Related:**
  - `plans/follow-ups/boat-runner-stderr-drain.md`, the same code and an
    order of work. Its option A rewrites `image/boat-runner-entry.sh:4-12`,
    the block with the fallback. When one later branch takes both, apply
    this item first, so that it guards that rewrite. After that option,
    the end of the entry is the completion point, and the pipe of A or C
    stays valid. The order test of that option uses a held writer, so it
    needs no pipe.
  - `plans/follow-ups/boat-stderr-safe-reasons.md`, a shared fixture. Its
    option D adds an entry case with a completion pipe and an exact journal
    line, in its steps 2 and 3. C gives that case its runner and its
    journal check, so apply this item first. With A, that case copies the
    pipe code a third time.
  - `plans/follow-ups/boat-runner-install-check.md`, the same code. Its
    option A installs the runner files through `image/provision.sh` and
    keeps the cases unchanged. Its missing-helper case deletes the helper
    after the install step. The journal result of this item does not
    depend on the install path, so either order works.
- **Recommendation:** C, the orchestrator's call on 2026-11-21, the same as
  the review. It is the structural fix for the class, because no entry
  case can install the recording writer without a journal expectation. It
  reuses the pipe of the working-writer case, and it serves the new entry
  case of the safe reasons item. A is the direct fix, with the same proof
  for this subcase.
- **Action:** fix after merge, apply C, because both fix rounds are used
  and the current fallback does not write to the journal.
