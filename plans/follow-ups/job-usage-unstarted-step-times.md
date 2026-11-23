# Steps that fail before running get no times

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, the new problem
  of the check of finding 302.
- **Location:** `crates/ciproxy-guest/src/usage/worker_log/steps.rs:65` and
  `:79-101`, in `Steps::observe`, and `image/checks.md:244-245`.
- **Reason:** Only a step whose `env` or `if` fails to evaluate meets the
  defect, and the outcome rules defer a rare product bug whose fix gets no
  harder later.

- **Terms:**
  - **Worker log:** the log that the GitHub runner writes for each job in
    `_diag/Worker_*.log`. The guest, the ciproxy program in the job VM,
    reads the steps from it.
  - **`StepsRunner` line:** an `Info` line of the runner class
    `StepsRunner` in the Worker log, such as
    `Processing step: DisplayName='Build'`.
  - **Step span:** the time from the start of a step to its end. The job
    page gives a step the figures of the points whose middle is in its span.
- **Background:**
  - The images pin runner `v2.338.2` (`image/pins.env:11`). The paths that
    start with `src/` are in its repository `actions/runner` at that tag.
  - The guest reads each step from four `StepsRunner` lines of that runner
    (`crates/ciproxy-guest/src/usage/worker_log/steps.rs:15-43`):
    - "Processing step: DisplayName=" gives the name.
    - "Starting the step." confirms that the step runs.
    - "Step result:" gives the end.
    - "Skipping step due to condition evaluation." marks a skipped step.
  - The steps must match the steps on GitHub, with times within one second.
    The planned end-to-end check of this branch compares them. It has not
    run yet, because it needs real machines.
  - Before its merge, this branch takes the start of a step from its
    "Processing step" line, because the runner starts the timeline record
    just after that line (`src/Runner.Worker/StepsRunner.cs:81-89`). A
    step keeps that start only when its "Starting the step." line confirms
    that it ran, so a background step gets no early start.
  - The same fix adds a rule to the pin guidance at
    `image/checks.md:244-245`: a step starts at the line that the runner
    writes just before `ExecutionContext.Start()`, and it ends at the line
    next to `Complete()`.
  - The `env` or the `if` of a step can fail to evaluate
    (`StepsRunner.cs:131-138` and `:237-241`). The runner then completes
    the step as failed. Only `RunStepAsync` writes "Starting the step." and
    "Step result:" (`:293` and `:355`), so such a step has neither line.
  - Just after the runner completes such a step, it writes "Update job
    result with current step result 'Failed'." (`StepsRunner.cs:274-277`).
    A step that ran is already closed at its "Step result:" line. A queued
    background step gets "No need for updating job result" instead
    (`:282`).
  - A `hashFiles` call in an `if` can run up to its limit of 120 seconds
    (`src/Runner.Worker/Expressions/HashFilesFunction.cs:16`) and then
    fail.
  - The next "Processing step" line closes such a step with no times
    (`crates/ciproxy-guest/src/usage/worker_log/steps.rs:65-77`). The API
    then gives it no duration and no figures.
  - A skipped step keeps no times (`docs/protocol/job-usage.md:83`).
- **Problem:** A step whose `env` or `if` fails to evaluate gets no times,
  while GitHub gives it a start and an end. Its row shows "Unknown" with no
  figures, and up to two minutes of the job go to no step.
- **Impact if no action:** The step that failed shows no duration and no
  figures, although a reader often looks for exactly that step.
- **Root question:** How does a step that fails before it runs get the
  span that GitHub gives it?
- **Options:**
  - **A.** Give such a step the span from its own two lines.
    - The start is the time of its "Processing step" line. The end is the
      line "Update job result with current step result 'Failed'.". Read
      that line only while a step is open that has no start.
    - Ignore that line when no step is open, or when the open step has a
      start. A step that ran and failed is already closed when the line
      comes. Today the parser fails the whole read, and so drops every
      step, on an event line that comes when no step is open
      (`crates/ciproxy-guest/src/usage/worker_log/steps.rs:79`).
    - Add the line to the pin guidance at `image/checks.md:244-245`, and
      add a fixture case.
    - Cost: one more message shape for the runner pin check. A background
      step whose `env` or `if` fails also gets these times, but GitHub
      gives it no start. The runner starts the record of a background step
      only at its slot (`StepsRunner.cs:86-90`), and `Complete()` sets
      only the finish time (`src/Runner.Worker/ExecutionContext.cs:528`).
      The guest cannot tell such a step from a sequential step.
    - Effort: Small.
- **Left out:** Fewer than three approaches are real, because the runner
  writes one line at the `Info` level that ends such a step, and each other
  source fails or is rejected:
  - The lines "Caught exception from expression for step.env" and "Caught
    exception from expression." (`StepsRunner.cs:134` and `:225`), because
    they need two message shapes, and A covers both failures with one.
  - An end at the next "Processing step" line, because a queued background
    step also has no start there, so it would get a wrong span.
  - Gaps that the API closes, because the API cannot tell such a step from
    a background step, which also has no times today.
  - The step telemetry line (`src/Runner.Worker/ExecutionContext.cs:1309`),
    because the runner writes it only for a step whose handler set its
    type (`src/Runner.Worker/Handlers/Handler.cs:54-72`,
    `ExecutionContext.cs:1265`), and such a step reaches no handler.
  - The jobs API of GitHub, because the user's decision of 23 November
    2026, option F, takes the steps from the VM, and
    `docs/protocol/job-usage.md:79-82` rejects the API.
  - No change, with a doc line that such a step shows "Unknown", because
    it drops the match with GitHub for that step.
- **Wider fix:** None that is not blocked. One record from the runner code
  that reports each step to GitHub would give every step its times. That
  needs a runner patch, which the Steps decision at
  `docs/protocol/job-usage.md:79-82` rejects, because Boat runs the stock
  runner.
- **Previous examples:** Before its merge, this branch moved the start of
  a step from "Starting the step." to the "Processing step" line, for the
  same reason: a step boundary must be the line next to `Start()` or
  `Complete()`.
- **Related:**
  - `plans/follow-ups/job-usage-overlapping-steps.md`, the same code and
    one shared rule: each step boundary is the log line next to
    `ExecutionContext.Start()` or `Complete()`. Its rewrite of the tracker
    must keep the end line of A for a step with no start. Neither item
    waits for the other.
  - `plans/follow-ups/job-usage-step-name-expressions.md`, the same wider
    fix: its option C, a runner record, would also give these steps their
    times.
- **Recommendation:** A. It gives such a step the span that GitHub shows,
  from two lines that the runner writes next to `Start()` and `Complete()`.
  Add a unit case with an `if` that fails to evaluate: the step gets the
  time of its "Processing step" line as its start, and the time of the
  "Update job result" line as its end. The case must fail on the current
  code.
- **Action:** fix after merge, apply A, because the likelihood is Rare, and
  the fix gets no harder later.
