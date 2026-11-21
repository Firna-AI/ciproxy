# No hosted code saves the cache facts of jobs

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, a problem outside the review scope that the finding
  check of review finding 4 reported.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-protocol/src/cache.rs:27`,
  `crates/ciproxy-store-interface/src/jobs.rs:132`, `insert_job`,
  `crates/ciproxy-store-interface/src/jobs.rs:134`, `update_job`,
  `crates/ciproxy-store-pg/src/jobs.rs:118`,
  `crates/ciproxy-store-pg/src/job_views.rs:71`, and
  `docs/protocol/cache.md:124`.
- **Reason:** The fix is hosted backend work in the gateway and the store
  crates, which the UI branch does not own.

- **Terms:**
  - **Machine cache report:** `MachineCacheReport` in
    `crates/ciproxy-protocol/src/cache.rs:14`, part of each machine report:
    the cache space, the seven-day counts, and `jobs`, the cache facts of
    each job.
  - **Job cache report:** `JobCacheReport` in `cache.rs:41-52`: the trusted
    organization and repository of one job, its exact run, job, and runner
    assignment, its job ID, and its usage, with counts when the machine
    cache handled its restores.
  - **Store projection:** the cache columns of the `jobs` table,
    `cache_source`, `cache_restores`, and `cache_restored_bytes`, at
    `crates/ciproxy-schema/src/jobs.rs:22-24`, which the job view reads.
- **Background:**
  - The agent sends a full machine report on each connection. Its `cache`
    field is a machine cache report, at
    `crates/ciproxy-protocol/src/reports.rs:34`, whose `jobs` field lists a
    job cache report for each job, at `cache.rs:27`.
  - The gateway saves the latest full report of each machine,
    `ReceivedReport` at `crates/ciproxy-store-interface/src/gateway.rs:24`.
    The Machines page reads only the cache space and the seven-day counts
    of that report, at `crates/ciproxy-api/src/handlers/machine_views.rs:64`.
  - No hosted code reads `MachineCacheReport.jobs`. The `jobs` table, with
    its cache columns, is written only by `insert_job` and `update_job`, at
    `crates/ciproxy-store-pg/src/jobs.rs:118-160`, and no code outside the
    tests calls them.
  - The job view query reads the source and the counts from that table, at
    `crates/ciproxy-store-pg/src/job_views.rs:71-73`. So no real job gets a
    cache source or counts, and the job page shows "Cache: Unknown", at
    `ts/app/src/view/jobs-detail.ts:124`.
  - `docs/protocol/cache.md:124-125` asks the job page to show the cache
    source of each job, and `:137-140` the counts when the machine cache
    handled the restores.
  - The wire and the store keep the source and the counts independent, at
    `crates/ciproxy-api/src/wire/jobs.rs:111` and
    `crates/ciproxy-store-interface/src/jobs.rs:112-116`. The store test at
    `crates/ciproxy-store-pg/tests/store/jobs_tests.rs:95` keeps direct
    GitHub origins with no counts.
  - A job cache report names the job by its organization and its exact
    assignment, so the writer can match it only to a job of the same
    workspace.
- **Problem:** The agent reports the cache facts of each job, but no hosted
  code saves them for the job that they describe.
- **Impact if no action:** The job page shows "Cache: Unknown" for every job
  that ran, so the requirement of `cache.md` stays unmet, and a customer
  cannot tell whether a job used the machine cache.
- **Root question:** How do the cache facts that the agent reports for each
  job reach the job that they describe?
- **Options:**
  - **A.** Save the facts when the gateway stores each machine report. For
    each job cache report, find the job of the same workspace with the same
    organization and exact assignment, and save its source and its counts
    in the store projection through the store interface. Keep the source
    and the counts independent, and save nothing for a report that matches
    no job of the workspace. Add real-store tests for the match, the
    workspace scope, and each usage kind. Effort: Medium.
  - **B.** Project the facts from the saved latest report in a hosted
    worker that runs after each report, with the same match, scope, and
    tests as A. Effort: Medium.
- **Left out:**
  - A new agent message or a new wire field, because the existing report
    already carries every fact that the job page needs.
  - Facts that the API reads from the latest machine report when a page
    asks, because a later report replaces the saved one, and the job view
    would then need a second source.
- **Wider fix:** None. The question is one missing writer.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/job-cache-cause.md` removes a caption that
  guesses the cause of a fallback from absent counts. It must land before
  this writer, or the guess reaches customers with the writer.
- **Recommendation:** A. One write path at the receipt of each report keeps
  the facts in the order of the reports, with no second process.
- **Action:** fix after merge, apply A, because the writer is hosted backend
  work outside the UI branch, and the protocol already requires the cache
  source.
