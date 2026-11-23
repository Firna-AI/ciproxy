# Reports stretch the last point past measurement

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1, finding 1.
- **Location:** `crates/ciproxy-guest/src/usage/series.rs:86`,
  `crates/ciproxy-protocol/src/usage.rs:23`,
  `crates/ciproxy-overflow/src/job_usage.rs:80`, and
  `crates/ciproxy-api/src/handlers/job_usage/series.rs:48`, `:116`, and
  `:126`.
- **Reason:** Each option changes the Points rule of the protocol doc
  `docs/protocol/job-usage.md`, which gives every point one width, and no
  user decision allows a shorter last point.

- **Terms:**
  - **Guest:** the ciproxy program in each job VM. It samples the CPU and
    the memory of the VM once per second.
  - **Point:** one entry of the samples of a job. It holds the average
    CPU, the peak memory, and the out-of-memory kills of one or more
    consecutive seconds.
  - **Point width:** the seconds that each point covers, the field
    `point_seconds` of `JobUsage`.
- **Background:**
  - The samples of a job hold at most 3,600 points. When a job runs
    longer, the guest merges each two neighbouring points, and the point
    width doubles. So the width is always a power of two.
  - The samples end when the runner exits. So the last point often covers
    fewer seconds than the width. `crates/ciproxy-guest/README.md` says:
    "The last point can cover part of its stated width."
  - The guest averages the CPU of the last point over the seconds that it
    measured. The report gives only the width, so each reader treats the
    last point as full.
  - The job page gives each point to the step that holds the middle of
    the point.
- **Problem:** A report claims time that the guest did not measure. In a
  probe, the guest measured 7,201 seconds, and the report described 7,204
  seconds: the last point described one measured second as four. Four
  readers assume a full last point: the 5-minute time check of the control
  plane at `crates/ciproxy-overflow/src/job_usage.rs:80`, and the end, the
  middle, and the CPU weight of the API at
  `crates/ciproxy-api/src/handlers/job_usage/series.rs:48`, `:116`, and
  `:126`.
- **Impact if no action:** The job page can give the final memory peak or
  the final out-of-memory kill of a job to the wrong step. The short point
  also gets too much weight in the average CPU, and the chart and
  "Complete job" end after the measured end.
- **Root question:** How can every reader know the time that each point
  measured, also when the runner exit cuts the last point short?
- **Options:**
  - **A.** Send only complete points, and update the protocol doc. This
    can drop the final memory peak or the final kill. Effort: Small.
  - **B.** Send the measured length of the last point. Use one shared
    calculation in the API for the end, the middle, and the CPU weight of
    each point. Update the protocol doc. Effort: Medium.
  - **C.** Give every point its own measured start and end, and update the
    protocol doc. This also allows gaps, but it changes more of the report
    and of its readers. Effort: Large.
  - **D.** Do B, but put the rule on the protocol type. Add the measured
    end of the samples to `JobUsage` as one optional field: the start of
    the first point plus the seconds that the guest counted
    (`Series::seconds()`). Add one helper on `JobUsage` that gives the end,
    and the span, the middle, and the weight in seconds of each point. Use
    it in the control plane check, in the API end, middle, and CPU
    averages, and in the guest test. A reader clamps the end into the last
    point, so a broken value changes only that point, and a missing field
    means a full last point. Add one column to the migration of job usage,
    and update the wire snapshots and the Points section of the protocol
    doc. `ciproxy-protocol` has its own Apache-2.0 licence and gets only a
    calculation on its own type. Effort: Medium.
- **Left out:**
  - The end from the runner exit time, because a failed read can end the
    samples before the runner exits, and the control plane clock is not
    the VM clock.
  - Sampling after the runner exit until the last point is full, because
    the samples end when the runner exits, and each exit report waits up
    to one point width longer.
  - Merging the last seconds into the point before, because it breaks the
    same rule and shows the final kills too early.
  - A list of runs, each with its own start and end, because with one run
    it is D, and with several runs it is C.
- **Wider fix:** D. It takes every span calculation out of the readers. C
  is wider only for gaps and uneven points, and the design keeps those out
  of the report.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/job-usage-overlapping-steps.md`, the same
  code: both change `crates/ciproxy-api/src/handlers/job_usage/series.rs`.
  The two changes do not conflict, so make them one after the other.
  `plans/follow-ups/job-usage-chart-open-step-start.md`, the same code, for
  its option C only: C ends its open block at the end of the samples, and D
  here moves that end to the measured end.
  `plans/follow-ups/job-usage-report-size-limit.md`, the same format: C here
  adds times to each point, which raises the size of a full report.
- **Recommendation:** D. It puts the span rule in the one type that every
  reader decodes, so the control plane check and the API cannot drift
  apart, and the clamp adds no new reason to drop figures.
- **Action:** ask the user, because every option changes the Points rule
  of `docs/protocol/job-usage.md`, which gives each point one width, and
  no user decision allows a shorter last point.
