# The chart hides known step starts

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, finding 506,
  with the new problem of its check.
- **Location:** `ts/app/src/view/jobs-usage-chart.ts:190`, in `span`, and
  `:233`, the `starts` of `usageChartModel`,
  `docs/protocol/job-usage.md:204-205`,
  `docs/mockups/src/fixtures/job-usage.ts:45-46`, and
  `docs/mockups/src/screens/jobs/usage-steps.tsx:61`.
- **Reason:** Only an incomplete step record meets the defect, which needs
  a runner that exits during a step or a Boat page that falls back to a
  sample report.

- **Terms:**
  - **Step lane:** the row above the two plots of the job chart. It draws a
    block for each step that lasted, with its number where it fits.
  - **Start mark:** the hairline that marks the start of a step in both
    plots of the job chart.
  - **Exit report:** the report of the runner exit, which carries the final
    samples and steps of the job.
  - **Sample report:** a report that a Boat guest sends every 30 seconds
    while its job runs, with the samples and the steps so far.
- **Background:**
  - The job page draws the CPU and the memory of a job over time, with a
    step lane above two plots. The chart model is `usageChartModel` in
    `ts/app/src/view/jobs-usage-chart.ts:205-241`.
  - `docs/protocol/job-usage.md:204-205` says that the chart has "a mark
    for each step and each out-of-memory kill". The mockup chart says that
    "a hairline marks the start of each step in both plots"
    (`docs/mockups/src/screens/jobs/usage-chart.tsx:5-7`). The header of
    the app model says the same (`ts/app/src/view/jobs-usage-chart.ts:6-7`).
  - A report can hold a step with a start and no end. The guest keeps an
    open step with its start and no end
    (`crates/ciproxy-guest/src/usage/worker_log/steps.rs:110-117`,
    `crates/ciproxy-guest/README.md:131`). The design call "No invented
    times" of this branch keeps a missing end unset.
  - The API keeps that start, and it gives "Complete job" no start
    (`crates/ciproxy-api/src/handlers/job_usage.rs:85-100`). The API test
    `steps_outside_the_samples_or_without_both_times_keep_no_figures`
    pins this (`crates/ciproxy-api/src/handlers/_tests_/job_usage_tests.rs:88`).
  - The steps table shows "Unknown" for such a step, with no figures
    (`ts/app/src/view/jobs-usage.ts:169-172`). The table has no start
    column, so only the chart can show the start.
  - The state reaches the page in two cases. A runner can exit during a
    step. A Boat page can fall back to the newest sample report when the
    delete of the sandbox at the end of the job comes before the exit
    report.
  - The fallback needs unusual timing. The control plane saves an exit
    report that arrives before the delete. In the one recorded live Boat
    run, the exit report arrived about 10.5 seconds before the record of
    the sandbox ended
    (`plans/follow-ups/boat-live-completion-acceptance.md:94-100`).
  - On 23 November 2026, the user chose two more guards for Boat: a hold
    of the delete of up to 15 seconds while no exit report has arrived, and
    the sample reports every 30 seconds. The hold is not built yet, because
    a related decision of this branch waits for the user.
  - The mockup fixture gives a step only a full time span, or none for a
    skipped step (`docs/mockups/src/fixtures/job-usage.ts:45-46`). The
    mockup shows each step without a span as "Skipped"
    (`docs/mockups/src/screens/jobs/usage-steps.tsx:61`). The mockup page
    `docs/mockups/src/entries/app/jobs/cpu-memory/` already shows five
    screens, the most that one page may show.
- **Problem:**
  - The chart takes start marks only from steps with both times. `span()`
    returns `null` when either time is missing
    (`ts/app/src/view/jobs-usage-chart.ts:190`), and the start marks come
    only from those spans (`:233`).
  - A probe of `usageChartModel` with a step "Build" that starts at 2
    seconds and has no end gave `starts: [1]` and no lane block. With a
    skipped step before it and a start at 3 seconds, the last block ended
    at 1.5 seconds, and nothing marked 3 seconds.
  - No mockup shows the "Unknown" row of a step with a start and no end,
    or the chart of that step.
- **Impact if no action:** A reader cannot place a spike or a memory kill
  against the start of the interrupted step. The end of the block before
  it is only a partial workaround.
- **Root question:** How can the chart show where a step ran when the
  report gives its start but no end?
- **Options:**
  - **A.** Read the known starts apart from the spans of the step blocks.
    Keep the duration and the figures of the row unknown. Effort: Small.
  - **B.** Give each step one chart placement: a span, a start only, or no
    mark. Use that placement for the step lane and both plots.
    - First add the state of such a step to the job mockup: its "Unknown"
      row, its start mark in both plots, and no lane block. Put it in a
      nested sub-page, because the `cpu-memory` page already shows five
      screens.
    - Add the regression case to `ts/app/_tests_/jobs-usage-chart.test.ts`
      through `usageChartModel`: a step with a start and no end gets a
      start mark and no lane block. The case must fail on the current code.
    - The review graded B Small. The mockup state makes it Medium.
    - Effort: Medium.
  - **C.** Do B, and draw the last listed step as an open block when it
    has a start and no end.
    - The block runs from its start to the end of the samples, with no end
      edge, because that step is still open when the samples end.
    - The row keeps "Unknown" and no figures. A step with a start and no
      end that a later step follows keeps only its start mark.
    - Add the look to the job mockup first, and add a model test.
    - Cost: a new lane look and a mockup state that no requirement covers,
      so C waits for the user, with the open question of how partial Boat
      figures show. A reader can still read the open block as a duration.
    - Effort: Medium.
- **Left out:**
  - A block from the start of each such step to the end of the chart,
    because it invents a duration for a step that a later step follows. C
    brings it back for the last listed step only, which is still open when
    the samples end.
  - The start only in the steps table, because the chart keeps its gap,
    so it fails the goal.
  - A protocol change to "a mark for each step with both times", because
    it fails the goal and changes a protocol line without a user decision.
  - A chart placement in the API, because the API already sends each known
    time apart (`crates/ciproxy-api/src/handlers/job_usage.rs:118-137`),
    and a placement field adds no fact.
  - A change in the guest, because "No invented times" keeps a missing end
    unset, and the open step of a sample report has no end yet.
- **Wider fix:** B. A missing end then cannot erase a known start from the
  chart model.
- **Previous examples:** `plans/follow-ups/job-usage-overlapping-steps.md`
  records the related case where the guest reads neither time of a
  background step. This case already has the start.
- **Related:**
  - `plans/follow-ups/job-usage-overlapping-steps.md`, the same code and
    part of a shared wider fix. Its options C, D, and E draw an overlapping
    step in another lane row, in the same lane code
    (`ts/app/src/view/jobs-usage-chart.ts:211-235`,
    `ts/app/src/screens/jobs/usage-chart.tsx:45`). The placement of B is
    where that lane row goes, so that fix extends the placement and adds no
    second step list. Its tracker of open steps can put several open steps
    in one sample report, and B handles any number of them. This item does
    not wait for it.
  - `plans/follow-ups/job-usage-short-last-point.md`, the same code, for C
    only. C ends its open block at the end of the samples, and option D of
    that item moves that end to the measured end. No conflict.
  - The open decision "Partial Boat figures look complete" of this branch,
    in `plans/job-step-cpu-and-memory.md` while that plan is open: work
    that waits for its answer, for C only. Its recommended option D runs a
    last step with a start and no end to the end of the samples, with its
    figures so far. C is the chart part of that option. B works with each
    answer: when the answer gives that step a span, the lane draws the
    span, and otherwise the start mark is the whole fix.
- **Recommendation:** B. It keeps each known start in the chart, gives the
  step no more figures, and follows the hairline rule of the mockup. The
  gate fails a source file over 300 lines, so check the length of
  `ts/app/src/view/jobs-usage-chart.ts` after the change.
- **Action:** fix after merge, apply B, because the defect needs an
  incomplete step record, and the steps table still names the step.
