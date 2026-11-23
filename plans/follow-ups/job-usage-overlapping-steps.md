# Steps that run at the same time lose their figures

- **Severity:** High. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1, findings 3 and
  106. Review round 2 checked finding 3 again, added option E, and
  corrected options B, C, and D.
- **Location:** `crates/ciproxy-guest/src/usage/worker_log/steps.rs:57` and
  `:65`, in `Steps::observe`,
  `crates/ciproxy-api/src/handlers/job_usage/series.rs:83-96`, in
  `step_figures`, `ts/app/src/view/jobs-usage-chart.ts:223-232`, the
  `lane` of `usageChartModel`, and `docs/protocol/job-usage.md:93-95`, the
  rule of a point in Steps.
- **Reason:** The best option changes the protocol rule that each point
  counts in one step, and no user decision covers that change.

- **Terms:**
  - **Background step:** a GitHub Actions step that runs while the later
    steps of the job run. GitHub made background steps and `parallel`
    groups public on 25 June 2026.
  - **`parallel` group:** a group of steps that run at the same time. The
    runner turns it into background steps with a wait step after them.
  - **Worker log:** the log that the GitHub runner writes for each job in
    `_diag/Worker_*.log`. The guest in the job VM reads the steps from it.
  - **Point:** one entry of the samples of a job: the average CPU, the
    peak memory, and the out-of-memory kills of some seconds.
  - **Step span:** the time from the start of a step to its end.
- **Background:**
  - The job page shows a row for each step with its time, its average
    CPU, its peak memory, and its kills. It gives each point to the step
    whose span holds the middle of the point. When two spans hold it, the
    first step in display order gets it
    (`crates/ciproxy-api/src/handlers/job_usage/series.rs:83-96`).
  - The protocol doc says: "a point belongs to the step that holds the
    middle of the point. Reason: the runner log gives step times in whole
    seconds, and each point then counts once"
    (`docs/protocol/job-usage.md:93-95`). The design expected overlap only
    from an odd log.
  - The guide `ts/site/src/content/docs/docs/guides/choose-a-label-size.md`
    says that a step's figures include the service containers and the
    processes that run during that step (`:41-43`).
  - Each figure covers the whole VM, by the decision at
    `docs/protocol/job-usage.md:27-29`. ciproxy does not measure which
    process used the CPU.
  - The steps come from the Worker log in the VM, by the user's decision
    of 23 November 2026, and not from the GitHub API. The page numbers the
    steps in their order (`docs/protocol/job-usage.md:91-92`), and the
    guest lists them in the order of their "Processing step" lines.
  - The images pin runner `v2.338.2` (`image/pins.env:11`). The runner
    files below are in `src/Runner.Worker/` of its repository
    `actions/runner` at that tag. `RunnerService.cs`,
    `HostTraceListener.cs`, and `Tracing.cs` are in `src/Runner.Common/`.
  - In that runner, the `StepsRunner` lines give the name of each
    step ("Processing step: DisplayName="). `StepsRunner.cs:245-248` sends
    a background step past `RunStepAsync`, which writes the start and
    result lines of the other steps (`:290-293`, `:355`).
  - The `BackgroundStepCoordinator` writes the lines of a background step
    under its own trace name (`RunnerService.cs:30-41`,
    `HostTraceListener.cs:137`). It names the step by its context name,
    not by its display name (`BackgroundStepCoordinator.cs:63`):
    - "waiting for slot." (`:217`), on the `StepsRunner` thread, before
      the loop takes the next step (`:61-87`).
    - "acquired slot." (`:219`), just before the record starts (`:221`).
    - "completed with result:" (`:269`), just after the record completes
      (`:268`).
    - "queued" (`:87`). With a free slot, "waiting for slot." and
      "acquired slot." come before it.
  - Other background steps write "acquired slot." and "completed with
    result:" from other threads at any time. So the first coordinator line
    after a "Processing step" line can name another step.
  - A `parallel` group becomes background steps with a wait step after
    them (the GitHub changelog of 25 June 2026, `JobExtension.cs:365-395`).
    The wait step is a normal step that waits inside `RunStepAsync`
    (`JobExtensionRunner.cs:32-34`, `BackgroundStepCoordinator.cs:149-169`).
    So its span overlaps each step of the group, and a group of N steps
    needs N + 1 lane rows.
  - The runner runs up to 10 background steps at once by default
    (`BackgroundStepCoordinator.cs:29`, `JobExtension.cs:524-525`). It also
    adds the wait step "Wait for all background steps" after each
    background step that no `wait` or `cancel` step covers
    (`JobExtension.cs:528-546`).
  - Steps that run one after another never share a point, because
    `StepsRunner` ends each one before it processes the next
    (`StepsRunner.cs:254-257`).
  - Before its merge, this branch takes the start of a sequential step
    from its "Processing step" line, when its "Starting the step." line
    confirms the run. So a background step gets no early start.
- **Problem:**
  - The guest reads only `StepsRunner` lines and holds one current step
    (`crates/ciproxy-guest/src/usage/worker_log/steps.rs:12` and `:57`).
    The next "Processing step" line closes a background step with no times
    (`:65-77`). A probe with the background trace format of the pinned
    runner gave no start and no end for a completed background step, so
    its row shows no time and no figures. When it is the first step, "Set
    up job" also loses its end
    (`crates/ciproxy-api/src/handlers/job_usage.rs:78-96`).
  - A guest fix alone moves the loss to other steps. A background step
    comes before the steps that run during it, so under the rule of the
    first step, it takes every point of those steps, and they show a dash.
  - No test gives two spans the same point. A change that removes the
    check for a point that another step already holds leaves every test
    green.
  - More places assume no overlap:
    - The rows "Set up job" and "Complete job" take their times from the
      first and the last listed steps
      (`crates/ciproxy-api/src/handlers/job_usage.rs:78-100`).
    - The chart draws all step blocks in one row
      (`ts/app/src/view/jobs-usage-chart.ts:223-232`,
      `ts/app/src/screens/jobs/usage-chart.tsx:45`).
    - The live check `image/check-worker-steps.py:78-89` requires each step
      to start after the end of the step before it. Its workflow
      `image/worker-steps-smoke/workflow.yml` has no background step.
    - The runner pin guidance at `image/checks.md:244-245` compares the
      fixture only with `StepsRunner.cs` and `HostTraceListener.cs`.
    - Three design calls of this branch, in
      `plans/job-step-cpu-and-memory.md` until the merge and in the pull
      request description after it. "Worker log steps" gives as its reason
      that the pinned runner writes the four `StepsRunner` lines for each
      step, which is false for background steps. "Step figures" says that
      a point belongs to the step that holds its middle, so each point
      counts once. "Step spans" says that the earlier step gets the point
      when two spans overlap.
- **Impact if no action:** A job that uses background steps shows no
  times or figures for them. A guest fix without a change of the rule
  hides the figures of the steps that do the work.
- **Root question:** How can the job page show the times and the figures
  of steps that run at the same time?
- **Options:**
  - **A.** Add a second tracker in the guest for the coordinator lines of
    background steps, and join each to its step by the step ID. Keep the
    page rule. Effort: Medium.
  - **B.** Turn both kinds of log lines into step events, and track all
    open steps by identity in one tracker.
    - Keep the steps in the order of their "Processing step" lines. Keep
      the page rule.
    - Join: bind each coordinator ID at its "waiting for slot." line to the
      step of the last "Processing step" line. Match the later lines of
      that ID by the ID only. Take the start from "acquired slot." and the
      end from "completed with result:".
    - Do not bind an ID at the first coordinator line after a "Processing
      step" line, because that line can come from another background step.
    - Join tests: another step's "completed with result:" line between a
      "Processing step" line and its "waiting for slot." line, and a step
      that gets its slot after a later step started.
    - Effort: Medium.
  - **C.** Do B. Then count a point in each step whose span holds its
    middle, so each row shows the VM during its own span, as the guide
    says.
    - Start "Complete job" at the latest step end.
    - Draw a step that overlaps an earlier block in a second chart row, and
      update the job mockup.
    - Add a background step and a `parallel` group to the live check and
      accept their overlap, and name `BackgroundStepCoordinator.cs` in
      `image/checks.md`.
    - Change the protocol rule.
    - Cost: one kill shows in each row that ran at that time. A second row
      also cannot draw a `parallel` group, because a group of N steps and
      its wait step need N + 1 rows, and up to 10 background steps run at
      once. E corrects the lane.
    - Effort: Large.
  - **D.** Do B. Then keep one step for each point: a shared point goes to
    the step that started last, and a tie goes to the later step in
    display order.
    - Make the same other changes as C, with the lane of E.
    - Cost: each later sequential step starts after a background step, so
      it takes the shared points. The runner also adds a wait step after
      each background step that no `wait` or `cancel` step covers. So under
      D a background step shows a dash in nearly every job, not only in
      `parallel` groups.
    - The steps of a `parallel` group that start in the same second also
      give their shared time to the last of them, so the others show part
      of their time or a dash, against the guide.
    - Effort: Large.
  - **E.** Do C with a lane of as many rows as the steps need.
    - Lane: put each step block in the first lane row where it overlaps no
      earlier block. A `parallel` group of N steps and its wait step then
      get N + 1 rows. Show a group of three in the job mockup.
    - Guide: in the "Find the step" section of
      `ts/site/src/content/docs/docs/guides/choose-a-label-size.md`, say
      that steps that run at the same time show in separate rows and share
      their figures.
    - Cost: a large group makes the lane taller, up to 11 rows at the
      default limit. The rows of a `parallel` group and its wait step show
      nearly the same figures, because they cover the same time of one
      VM. The panel note already says that each figure covers the whole
      VM (`docs/protocol/job-usage.md:212-213`).
    - Effort: Large.
- **Left out:**
  - The GitHub API, because the user's decision of 23 November 2026 takes
    the steps from the VM: each job then costs a GitHub call, and the times
    come from another clock.
  - A runner patch, because Boat runs the stock runner.
  - The step telemetry line of `ExecutionContext.cs:1309`, because it has
    no display name, comes only when a step completes, and carries step
    error text.
  - A background mark in the report, because the page can find a
    background step from the spans alone.
  - A split of a shared point between its steps, because it breaks the
    decision that each figure covers the whole VM
    (`docs/protocol/job-usage.md:27-29`). ciproxy does not measure which
    process used the CPU.
  - Figures for the process tree of each step, because it breaks the same
    decision, and service containers run outside the tree of the step that
    starts them.
  - A separate table section for background steps, because it breaks the
    order rule of `docs/protocol/job-usage.md:91-92`.
  - Step figures that the guest computes, because E is better on every
    point. The API computes the figures, so that a change of the rule
    needs no migration, by the design call "Where to compute" of this
    branch.
  - One start line and one end line for every kind of step is not a choice.
    Runner `v2.338.2` writes none at the `Info` level, and
    `Tracing.Entering` writes only a `Verbose` method name
    (`Tracing.cs:110-112`).
- **Wider fix:** E. A rule without a claim order holds for every order and
  overlap of steps, and its lane and its join hold for any number of steps
  that run at once.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/job-usage-short-last-point.md`, the same code: C, D,
    and E change `step_figures`, and option D of that item changes
    `midpoint_index`, both in
    `crates/ciproxy-api/src/handlers/job_usage/series.rs`. No conflict.
  - `plans/follow-ups/job-usage-step-links.md`, the same code: its options
    B and F extend the step tracker of B.
  - `plans/follow-ups/job-usage-chart-open-step-start.md`, the same code.
    Its option B gives each step one chart placement in `usageChartModel`
    (`ts/app/src/view/jobs-usage-chart.ts:190` and `:223-233`). Put the
    lane row of C, D, or E into that placement, in one change or after it.
    No conflict.
  - `plans/follow-ups/job-usage-unstarted-step-times.md`, the same code and
    one shared rule: each step boundary is the log line next to
    `ExecutionContext.Start()` or `Complete()`. For a background step, that
    line is "acquired slot." or "completed with result:". The rewrite of
    B must keep the rule of that item.
  - `plans/follow-ups/job-usage-step-name-expressions.md`, the same code.
    Its options read final names into the tracker that B rewrites. A match
    by start time is not unique for steps that start in the same second.
- **Recommendation:** E. It keeps the rule of C, and its lane holds any
  number of steps that run at once. D is the best option that keeps the
  rule that each point counts once. Do not apply A or B alone.
  - With C, D, or E, add a unit case to
    `crates/ciproxy-api/src/handlers/_tests_/job_usage_tests.rs`: two spans
    that share an interior full point, which holds the memory peak and a
    kill, with the exact figures of each step.
  - Replace the design calls "Worker log steps", "Step figures", and "Step
    spans" in the record of the fix, with the new rule and its date.
- **Action:** ask the user, because the recommended option E changes the
  protocol rule that each point counts in one step
  (`docs/protocol/job-usage.md:93-95`), and no user decision covers that
  change.
