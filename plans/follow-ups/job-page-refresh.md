# Finished jobs stay marked as running

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, finding 502,
  with two new problems of its check.
- **Location:** `ts/app/src/screens/jobs/jobs-hooks.ts:93`,
  `ts/app/src/state/jobs-detail.ts:19`,
  `ts/app/src/screens/jobs/job-screen.tsx:99`,
  `crates/ciproxy-api/src/handlers/job_usage.rs:37`,
  `crates/ciproxy-api/src/wire/jobs.rs:260`, and
  `docs/protocol/job-usage.md:216`.
- **Reason:** The job page also reads once on `main`, and the fix adds a
  refresh cycle and an API field for the whole job page, outside the files
  of the CPU and memory panel.

- **Terms:**
  - **Job page:** the dashboard page of one job, in
    `ts/app/src/screens/jobs/job-screen.tsx`. It shows a header with the
    status, a Details panel, and the CPU and memory panel.
  - **Exit report:** the report in which the guest, the ciproxy program in
    the job VM, says that the runner exited. It carries the final CPU and
    memory figures of the job.
  - **Sample report:** a report that a Boat guest sends every 30 seconds
    while its job runs. It holds the figures so far.
  - **Page store:** `createPageStore` in `ts/app/src/state/page-store.ts:76`.
    While a screen watches it, it reads again on an interval until a read
    gives a good page that does not wait.
  - **Jobs Live:** the dashboard queue page. It reads again on an interval
    while its tab shows.
- **Background:**
  - `useJob` loads the job page once, when the page mounts
    (`ts/app/src/screens/jobs/jobs-hooks.ts:93-95`). The job store has no
    refresh (`ts/app/src/state/jobs-detail.ts:19-41`).
  - So the header status, the "Running for" row of the Details panel
    (`ts/app/src/view/jobs-detail.ts:190`), and the panel sentence keep
    their first state. On `origin/main` the page already read once, so its
    header status froze. This branch adds the other two.
  - The panel of a running job says "The figures show when the job ends."
    (`docs/protocol/job-usage.md:216`). A finished job without figures
    says "Figures are not available for this job." (`:219-220`).
  - Two separate events end a job for the page:
    - GitHub's completion message sets the completed state
      (`crates/ciproxy-github/src/listener/messages.rs:29`). "Completed"
      means that GitHub has completed the job
      (`crates/ciproxy-store-interface/src/jobs.rs:20`).
    - The figures come later, with the exit report. On an own machine, the
      agent ends the job VM first, and its exit report then carries the
      figures (`docs/architecture/data-flow.md:32-36`).
    - A job can also get its GitHub times later, after timing recovery
      (`crates/ciproxy-store-interface/src/job_views.rs:48`).
  - The store marks the usage of an accepted exit report as final, with
    `StoredJobUsage::is_final`
    (`crates/ciproxy-store-interface/src/job_usage.rs:17-18`). It also
    keeps `received_at`. The API read of the usage uses neither
    (`crates/ciproxy-api/src/handlers/job_usage.rs:33-39`).
  - The completed state cannot wait for the figures, because History,
    retention, and the Boat cleanup use it
    (`crates/ciproxy-control-plane/src/boat_jobs.rs:45`).
  - A direct link can open the page of a running job. So can a busy slot
    on Machine detail, which links to its running job
    (`ts/app/src/screens/machines/machine-slots.tsx:65`,
    `ts/app/src/view/machines-routes.ts:48`).
  - The Billing usage store already waits on a server state with the page
    store: it passes `waits: (page) => statusWaits(page.status)`
    (`ts/app/src/state/billing.ts:56-71`). `useLive` calls `store.watch()`
    and returns its stop (`ts/app/src/screens/jobs/jobs-hooks.ts:43-46`).
    The shared poller reads at once when the tab shows again
    (`ts/app/src/state/poller.ts:66-70`).
- **Problem:**
  - The page reads the job once. In Chrome, the fixture supplied completed
    data, but the panel still showed the running sentence after 65
    simulated seconds. A reload drew the figures.
  - The API read ignores `is_final`. So a read after GitHub completes a
    job, and before the exit report arrives, shows "Figures are not
    available for this job." for an own-machine job. For a Boat job, it
    shows the newest sample report as the figures.
    `docs/protocol/job-usage.md` does not cover that gap.
  - So a refresh that stops at the first completed read can keep that
    state until a reload.
- **Impact if no action:** A reader who waits on the page does not see the
  result of the job or its figures without a reload.
- **Root question:** How does an open job page read again until the server
  holds the final state of the job, its result and its final figures, and
  then stop?
- **Options:**
  - **A.** Give the reader a refresh action, and say when to use it. Cost:
    a new action, and a change of the running sentence of
    `docs/protocol/job-usage.md:216`, which needs the user. Effort: Small.
  - **B.** Reuse the refresh pattern of Jobs Live while the page shows a
    running job. Stop on completion or when the page closes. Add a check
    from running to finished. Cost: it stops at GitHub's completion,
    before the figures arrive. Effort: Medium.
  - **C.** Notify open job pages when jobs complete. Cost: a delivery
    channel that the dashboard does not need for this one state. Effort:
    Large.
  - **D.** Do B with the page store, and read on for a settle time after
    the first completed read.
    - Make the job store a `createPageStore`, as the Billing stores do. In
      `useJob`, call `watch()` and return its stop, as `useLive` does.
    - `waits` holds while the job is queued or running. It also holds for
      a settle time after the first read that shows the job completed, for
      example 2 minutes. In that time, the exit report replaces "Figures
      are not available for this job." or a Boat sample.
    - The page store stops by itself, keeps the last good page when a
      refresh fails, and pauses in a hidden tab.
      `plans/follow-ups/shared-poller-lifetime.md` already covers its
      `watch()`, so the job store needs no restart guard of its own.
    - Show a failed refresh as `ts/app/src/screens/jobs/live-view.tsx:45-52`
      does. The job page mockups have no state for a failed refresh, so
      add one to them first.
    - Add a store test with fake timers: running, then completed without
      figures, then measured, then no read scheduled. Use the style of
      `ts/app/_tests_/setup.test.ts:253`.
    - Cost: the settle time is a guess at the delivery time. The page
      cannot tell a Boat sample from final figures, so it reads for the
      whole settle time and can still stop on a sample. Each read repeats
      the job read of the whole workspace, as each Live read does.
    - Effort: Medium.
  - **E.** Do D, but let the API say when the page is final.
    - Add one field to `JobDetailPage`
      (`crates/ciproxy-api/src/wire/jobs.rs:260`), for example `settled`.
    - The field is false while the job is queued or running. It is also
      false while a completed job with a runner has no final usage and
      finished less than a limit ago, for example 5 minutes.
    - The page store waits while `settled` is false, as the Billing stores
      wait on `statusWaits`.
    - Regenerate the API types with `npm --prefix ts run api:generate`.
      Add an API test for each case of the field, and the store test of D.
      Record the limit as a design call.
    - Cost: one wire field and its Rust test. In the seconds before the
      exit report, the panel still shows the state that the protocol gives
      a finished job.
    - Effort: Medium.
  - **F.** Do E, but put the wait in the existing running state of the
    usage.
    - The API gives the usage state "running" until the usage is final or
      the limit passes.
    - The page waits while the job is not completed or the usage state is
      "running".
    - Change the running and unavailable lines at
      `docs/protocol/job-usage.md:216-220`.
    - Cost: it changes a protocol requirement, so it needs the user. For a
      few seconds, the header says that the job ended, while the panel says
      that the figures show when the job ends.
    - Effort: Medium.
- **Left out:**
  - A read of every finished job, because it reads old jobs that never
    change, and E is better on every point.
  - A copy change alone that tells the reader to reload, because the open
    page still shows the running state, so it fails the goal.
  - A read only when the tab shows again, because a reader who keeps the
    page in view never sees the end. The page store of D and E already
    reads at once when the tab shows again.
  - A refresh of the usage alone, because the header and the Details panel
    stay stale, and it adds a route.
  - The Live store as the trigger, because Live drops the job at GitHub's
    completion, which is the early stop of B.
  - A completed state that waits for the figures, because History,
    retention, and the Boat cleanup use the completed state from GitHub.
  - A cheaper read of one job, because it does not solve this problem.
    `plans/follow-ups/job-history-paging.md` holds it.
- **Wider fix:** E. B and D must know the order in which the completion and
  the figures arrive, so they are direct fixes.
- **Previous examples:** `ts/app/_tests_/setup.test.ts:253` checks that
  Setup stops polling when the first job completes.
  `plans/follow-ups/shared-poller-lifetime.md` records why a page close
  must stop the refresh cycle.
- **Related:**
  - The open decision "Partial Boat figures look complete" of this branch,
    in `plans/job-step-cpu-and-memory.md` while that plan is open: the same
    gap, with no conflict. It decides how the page shows figures that come
    from a sample report, and it waits for the user. Its recommended
    option D marks figures as final only from an accepted exit report, and
    the page then shows a note on other figures. E reads `is_final` only
    to decide when the page stops reading.
  - `plans/follow-ups/job-history-paging.md`, the same code. To find one
    job, the job page read loads every job of the history window of the
    workspace (`crates/ciproxy-api/src/handlers/jobs.rs:116-120`). Each
    refresh of D, E, or F repeats that load. The user's answer of 8
    November 2026 in that item gives the job page a read of one exact job.
    Neither item waits for the other.
  - `plans/follow-ups/job-usage-step-links.md`, options that conflict, for
    its option E only. That option reads the GitHub steps each time the
    API serves the job page, so each refresh here would repeat that read.
    If the user picks that option, its read must not run on each refresh.
- **Recommendation:** E. The server holds `is_final`, so it can say when
  the page is final, and the page reuses the store that already waits on
  a server state. B stops at GitHub's completion, so the open page can end
  on "Figures are not available for this job." or on a Boat sample until a
  reload. D is the fallback when the API field is not wanted. E changes no
  protocol line. It only adds a field to the job page response.
- **Action:** fix after merge, apply E, because the job page reads once on
  `main` too, and the fix adds a refresh cycle and an API field for the
  whole page, outside the files of the panel.
