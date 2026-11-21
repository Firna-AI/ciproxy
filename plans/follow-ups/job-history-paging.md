# History returns every match with no paging

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the Jobs pages.
- **Source:** Implementation.
- **Category:** Performance.
- **Location:** `crates/ciproxy-api/src/handlers/jobs.rs:32`,
  `crates/ciproxy-api/src/handlers/jobs.rs:63`,
  `crates/ciproxy-api/src/handlers/jobs.rs:115`,
  `crates/ciproxy-api/src/wire/jobs.rs:248`,
  `crates/ciproxy-store-pg/src/job_views.rs:35`,
  `crates/ciproxy-store-pg/src/job_views.rs:100`,
  `ts/app/src/view/jobs-history.ts:20`, and
  `docs/mockups/src/screens/jobs/jobs-history.tsx:85`.
- **Reason:** B changes the store, the API, the app, and the mockups, so a
  separate workspace builds it after pull request #53 merges.

- **Terms:**
  - **History:** the tab of Jobs that lists finished jobs within the job
    history of the plan: 7 days on Free, 30 on Team, and 90 on Business.
  - **Keyset page:** a page of rows in a fixed order, with an opaque
    continuation that names where the next page starts.
  - **Retained job:** a job whose metadata ciproxy still keeps. ciproxy
    keeps it for 90 days on every tier, also on Free, as
    `docs/protocol/security.md:246` says.
- **Background:**
  - `GET /api/workspaces/{id}/jobs/history` takes `search`, `place`,
    `from`, and `until`. `JobsHistoryPage` at `wire/jobs.rs:248` returns
    `jobs`, `history_days`, and the banner. It has no total and no
    continuation.
  - The handler at `jobs.rs:63` reads through `read` at `jobs.rs:32`,
    which calls `dashboard_jobs`. The store query at
    `crates/ciproxy-store-pg/src/job_views.rs:35-99` builds every retained
    job of the workspace, because its inner reads have no time bound. Only
    its last step, `visible` at `:100-105`, drops the finished jobs that
    are older than the plan window. The query has no limit.
  - The handler then filters by state, place, search, and date in memory,
    at `jobs.rs:77-104`, and returns every match, newest first. The page
    renders one row for each job, with no virtualized list, at
    `ts/app/src/screens/jobs/job-list.tsx:125`.
  - The live queue at `jobs.rs:45` and the job page at `jobs.rs:115` use
    the same `read`. They then keep only the unfinished jobs, or one job.
    The dashboard repeats the live read every 5 seconds while its tab
    shows. So each of these reads also builds every retained job, such as
    about 75,000 jobs on Team at its included starts.
  - Business includes 150,000 starts a month, so 90 days of retained jobs
    can hold hundreds of thousands of jobs.
  - The History page opens on Today by default. Today narrows the matches
    that the API returns, but the backend still builds every retained job
    before it filters. The page shows every job that the API returns. Its
    footer counts the jobs of the list and names the window of the plan in
    days, such as "11 jobs today · Your plan keeps 30 days of history", at
    `ts/app/src/view/jobs-history.ts:20`.
  - The known-data mockup shows 11 rows of Today with no filter, and the
    footer "11 of 612 jobs today · Team keeps 30 days of history", at
    `docs/mockups/src/screens/jobs/jobs-history.tsx:85`. That footer needs
    pages and a total that the API does not give. Its footer row has no
    "Show more" control, at `:146`. The current-contract state of the same
    file shows the footer of the product.
  - No protocol doc states paging or a total for History.
    `docs/protocol/billing.md` says only that the job history of a tier is
    the window of finished jobs that its pages show.
  - Associated work, with no decision of its own: whichever option the
    user chooses, the same change also gives the live queue and the job
    page narrow store reads, of the unfinished jobs and of one exact job.
    Otherwise those two reads still build every retained job.
- **Problem:** A wide filter on a large workspace returns every match in
  one response. Each read of History, of the live queue, and of the job
  page also builds every retained job of the workspace before any filter.
  Nothing measured the cost. The known-data mockup and the product
  disagree on pages and a total.
- **Impact if no action:** A large workspace can get slow History, live,
  and job pages, on every tier. The mockups keep two versions of the
  History footer.
- **Root question:** How does History deliver the jobs of a large plan
  window?
- **Options:**
  - **A.** Keep one full list, so the count of the list stays the count of
    the matches. First measure History and the live read at a realistic
    Business volume. The seed holds 90 days of jobs, job attempts, runners,
    and Boat starts, on every tier. Choose another option only when a read
    is too slow, and change the known-data footer to the footer of the
    product. Effort: Medium.
  - **B.** Keyset pages without a total. The store query applies the
    filters, the order, and the limit, and each of its inner reads stops
    at the plan window, so no read builds every retained job. The API
    returns a fixed number of rows in that order and a continuation. The
    page shows the count of the rows it has and "Show more". Effort:
    Medium.
  - **C.** Keyset pages with a total for the filters, so the footer can say
    "11 of 612 jobs today", as the known-data mockup does. The store query
    applies the filters, the order, and the limit as in B. The count must
    merge both job sources over the whole window for each read. Effort:
    Medium.
  - **D.** Keep one full list, and move the History filters into the store
    query: the finished state, the plan window, the place, the dates, and
    the search, newest first. Each inner read stops at the plan window.
    Change the known-data footer to the footer of the product. The API
    contract and the app stay the same. This drops the shared assumption
    that the filters stay in the handler, over the full read: that is the
    current code at `jobs.rs:77-104`, not a requirement. Cost: a wide
    filter on a large workspace still returns and renders every match. So
    D bounds the usual read, but not the worst one. Effort: Medium.
  - **E.** Cap the list in the store query. The store query applies the
    filters and the order, with bounded inner reads as in D, and returns
    the newest matches up to a fixed number, such as 200, with a flag when
    more jobs match. The footer then asks the person to narrow the search,
    the place, or the date. No continuation and no total. The known-data
    footer row has no "Show more" control, so a cap is a fair reading of
    that mockup. Cost: on a day with more matches than the cap, older jobs
    show only through a narrower filter. A total, as in the known-data
    footer, costs the count of C. Effort: Medium.
- **Left out:**
  - A limit in the browser, because it still loads every match, so it
    fails the goal.
  - Offset pages, because jobs finish at the top of the list while a
    person reads it. So offset pages repeat or skip jobs, which fails the
    goal. The keyset pages of B and C do not have this defect.
  - Pages or a cap only in the handler, over the shared full read, because
    each read then still builds every retained job.
- **Wider fix:** None. The question is the contract of one route.
- **Previous examples:** None found.
- **Recommendation:** B. The code already shows that the store read, the
  response, and the rendered list grow with the window, and B bounds all
  three without the count of C. The first pass recommended A.
- **User answer, 2026-11-08:** B. Rejected: A, C, D, and E.
  - History gets keyset pages without a total. The store query applies the
    filters, the order, and the limit, with bounded inner reads within the
    plan window.
  - The same change gives the live queue and the job page narrow store
    reads, of the unfinished jobs and of one exact job.
  - The mockups follow B. The History footer gives the count of the rows
    that the page has, with "Show more", and no total of the day.
- **Action:** fix after merge, apply B, because it is store, API, and app
  work for a separate workspace, outside the UI branch.
