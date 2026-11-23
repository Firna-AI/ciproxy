# Steps that run at the same time lose their figures

- **Severity:** High. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1, findings 3 and
  106.
- **Location:** `crates/ciproxy-guest/src/usage/worker_log/steps.rs:62` and
  `:70`, `crates/ciproxy-api/src/handlers/job_usage/series.rs:83`, and
  `docs/protocol/job-usage.md:91`.
- **Reason:** The best option changes the protocol rule that each point
  counts in one step, and no user decision covers that change.

- **Terms:**
  - **Background step:** a GitHub Actions step that runs while the later
    steps of the job run. GitHub made background steps and `parallel`
    groups public on 25 June 2026.
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
    (`crates/ciproxy-api/src/handlers/job_usage/series.rs:83`).
  - The protocol doc says: "a point belongs to the step that holds the
    middle of the point. Reason: the runner log gives step times in whole
    seconds, and each point then counts once"
    (`docs/protocol/job-usage.md:91`). The design expected overlap only
    from an odd log.
  - The guide `ts/site/src/content/docs/docs/guides/choose-a-label-size.md`
    says that a step's figures include the service containers and the
    processes that run during that step.
  - The steps come from the Worker log in the VM, by the user's decision
    of 23 November 2026, and not from the GitHub API.
  - In runner `v2.338.2`, the `StepsRunner` lines give the name of each
    step ("Processing step: DisplayName="). `StepsRunner.cs:245-248` sends
    a background step past `RunStepAsync`, which writes the start and
    result lines of the other steps. `BackgroundStepCoordinator.cs:215`
    writes the start and the end of a background step instead. With a free
    slot, its lines "waiting for slot." and "acquired slot." come before
    "queued" (`BackgroundStepCoordinator.cs:85-87` and `:217-219`).
- **Problem:**
  - The guest reads only `StepsRunner` lines and holds one current step.
    A probe with the background trace format of the pinned runner gave no
    start and no end for a completed background step, so its row shows no
    time and no figures.
  - A guest fix alone moves the loss to other steps. A background step
    comes before the steps that run during it, so under the rule of the
    first step, it takes every point of those steps, and they show a dash.
  - No test gives two spans the same point. A change that removes the
    check for a point that another step already holds leaves every test
    green.
  - More places assume no overlap: the rows "Set up job" and "Complete
    job" take their times from the first and the last listed steps
    (`crates/ciproxy-api/src/handlers/job_usage.rs:78-99`); the chart
    draws all step blocks in one row (`ts/app/src/view/jobs-usage-chart.ts:223`
    and `ts/app/src/screens/jobs/usage-chart.tsx:45`); the live check
    `image/check-worker-steps.py:78` requires each step to start after the
    end of the step before it, and its workflow
    `image/worker-steps-smoke/workflow.yml` has no background step; and
    the runner pin guidance at `image/checks.md:244` compares the fixture
    only with `StepsRunner.cs` and `HostTraceListener.cs`.
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
    open steps by identity in one tracker. Bind a step ID at its first
    coordinator line after the "Processing step" line of that step. Keep
    the steps in the order of their "Processing step" lines. Keep the
    page rule. Effort: Medium.
  - **C.** Do B. Then count a point in each step whose span holds its
    middle, so each row shows the VM during its own span, as the guide
    says. Start "Complete job" at the latest step end. Draw a step that
    overlaps an earlier block in a second chart row, and update the job
    mockup. Add a background step and a `parallel` group to the live check
    and accept their overlap, and name `BackgroundStepCoordinator.cs` in
    `image/checks.md`. Change the protocol rule. Cost: one kill shows in
    each row that ran at that time. Effort: Large.
  - **D.** Do B. Then keep one step for each point: a shared point goes to
    the step that started last, and a tie goes to the later step in
    display order. Make the same other changes as C. Cost: the steps of a
    `parallel` group that start in the same second give their shared time
    to the last of them, so the others show part of their time or a dash,
    against the guide. Effort: Large.
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
- **Wider fix:** C. A rule without a claim order holds for every order
  and overlap of steps.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/job-usage-short-last-point.md`, the same
  code: both change `crates/ciproxy-api/src/handlers/job_usage/series.rs`,
  with no conflict. `plans/follow-ups/job-usage-step-links.md`, the same
  code: its options B and F extend the step tracker of B.
- **Recommendation:** C. It is the only rule that keeps the guide true for
  every step. D is the best option that keeps the rule that each point
  counts once. Do not apply A or B alone. With C or D, add a unit case to
  `crates/ciproxy-api/src/handlers/_tests_/job_usage_tests.rs`: two spans
  that share an interior full point, which holds the memory peak and a
  kill, with the exact figures of each step.
- **Action:** ask the user, because the recommended option C changes the
  protocol rule that each point counts in one step, and no user decision
  covers that change.
