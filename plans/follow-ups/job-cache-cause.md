# The job page guesses why the cache fell back

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, review finding 4.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/view/jobs-detail.ts:113`, in `jobDetailModel`,
  `ts/app/src/view/jobs-format.ts:154`, in `placeOf`,
  `crates/ciproxy-api/src/wire/jobs.rs:181`,
  `crates/ciproxy-protocol/src/cache.rs:63`, and
  `docs/mockups/src/entries/app/jobs/job/index.mockup.tsx:34`.
- **Reason:** No hosted code saves the cache facts of a job yet, so no
  customer sees the caption until a hosted writer lands, and this
  correction must land before that writer.

- **Terms:**
  - **Cache source:** the API fact that names where the restored contents
    of a job came from: this machine, the GitHub cache, or both.
  - **Cache counts:** the entries and bytes that the machine cache measured
    when it handled the restores of a job.
  - **Known-data scene:** a mockup scene of the fixture story, with every
    fact.
  - **Current-contract scene:** a mockup scene with only the facts that
    the API gives today.
- **Background:**
  - `JobView` gives `cache_source` and `cache_counts`, and keeps them
    independent, at `wire/jobs.rs:111`. The counts are "absent for direct
    GitHub and fallback", at `:181`. The API gives no cause and no fallback
    flag.
  - The machine report merges both cases in one value,
    `JobCacheUsage::Github`, "Direct GitHub use or fallback", at
    `cache.rs:63-64`. The store says the same, at
    `crates/ciproxy-store-interface/src/jobs.rs:112-116`.
  - `jobDetailModel` sets `fellBack` when the counts are absent, the source
    is GitHub, and the job ran on an own machine, at `jobs-detail.ts:113`.
    The row then gets the caption "The cache of `<machine>` was not
    available, so the job used the GitHub cache.", at `:119`.
  - Two real cases make that caption false:
    - A restart of the machine cache service marks each active job as
      GitHub use with no counts, at
      `crates/ciproxy-cache-sidecar/src/metrics.rs:77-79`. The guest keeps
      its proxy across the restart, as
      `crates/ciproxy-cache-sidecar/README.md:183` says, and the agent
      recovers or registers the session again, at
      `crates/ciproxy-agent/src/cache.rs:173-175`. The record keeps the
      GitHub value, at `metrics.rs:106`. So the machine cache can serve
      the job while its report says GitHub use.
    - A job uses GitHub directly when the repository metadata is
      unavailable, at `crates/ciproxy-protocol/src/agent.rs:163`. That is
      no failure of the machine cache.
  - When the API gives no machine name, `placeOf` names the place "Own
    machine", at `jobs-format.ts:154`. The caption then reads "The cache of
    Own machine was not available".
  - `docs/protocol/cache.md:139-140` says that a job that fell back to the
    GitHub cache shows only the cache source.
  - The known-data scene `fallback`, at `index.mockup.tsx:34-39`, shows the
    caption from an explicit `fallback` fact of its fixture, at
    `docs/mockups/src/fixtures/job-details.ts:43`.
  - No hosted code saves the cache facts of a job yet, as
    `plans/follow-ups/job-cache-report-writer.md` records. So no production
    page shows the caption today.
- **Problem:** The page turns absent counts into a confirmed cause. A
  GitHub cache source with no counts on an own machine becomes the claim
  that the machine cache was not available. The source and the counts
  describe the origins and the measurements of the contents, not why the
  job used GitHub.
- **Impact if no action:** Once a hosted writer saves these facts, a
  customer can diagnose a cache failure that did not happen, for example
  after a restart of the cache service. The page cannot tell direct GitHub
  use from a fallback, and the broken words "The cache of Own machine"
  reach customers.
- **Root question:** How does the job page explain a job on an own machine
  that used the GitHub cache?
- **Options:**
  - **A.** Show "The GitHub cache" without the inferred caption. Keep the
    measured counts when the API gives them. Add the case of a GitHub
    source with no counts to the existing job view test. The mockup state
    comes first. Effort: Small.
  - **B.** Add an explicit cause of direct GitHub use to the machine
    report, the saved job, and the generated API. Show the caption only
    when that cause confirms a fallback. It starts in the report of the
    machine, because `JobCacheUsage::Github` merges both cases, and it
    needs the hosted writer. It adds a boundary requirement. Effort: Large.
  - **C.** Apply A, and record the gap as for the other Jobs facts that
    the API lacks:
    - Add a current-contract scene of the `fallback` moment: the own
      machine, the GitHub cache source, and no caption. Put it on a linked
      sub-page, because the Jobs current-contract page already shows five
      screens.
    - Name the missing cause in the rationale of the Jobs current-contract
      entry and in `docs/mockups/README.md`.
    - Keep the known-data `fallback` scene unchanged.
    - Add the cause as a fourth fact to
      `plans/follow-ups/jobs-notice-facts.md`, with B as part of its API
      option. The user then chooses there between the known-data scene
      and the current-contract copy. That decision stays open.
    - Effort: Medium.
- **Left out:**
  - A cause inferred from absent counts, because the restart case makes it
    false.
  - Hiding the known cache source, because `docs/protocol/cache.md:124-125`
    asks the job page to show it.
  - A contract rule that a GitHub source with no counts on an own machine
    means a fallback, because the restart case makes it false, and it
    needs Rust changes.
  - A caption that says that the cause is not available, because it states
    a limit that the customer does not need.
  - A change of the known-data `fallback` scene now, because it decides
    which side of a mockup and product mismatch is right. C brings that
    choice to the user.
- **Wider fix:** None. The real-data rule for screens and the
  current-contract pattern of `docs/mockups/README.md` are already in
  force.
- **Previous examples:** Commit `05c7770` corrected the cache-source text
  of the job mockup for mixed origins and zero restores. The store test at
  `crates/ciproxy-store-pg/tests/store/jobs_tests.rs:95` keeps direct GitHub
  origins with no counts.
- **Related:**
  - `plans/follow-ups/job-cache-report-writer.md` adds the hosted writer of
    these facts. This item must land before that writer.
  - `plans/follow-ups/jobs-notice-facts.md` gets the cause as a fourth fact
    when C lands. Its decision and its Action stay open.
- **Recommendation:** C. It keeps the mockups and the product on the facts
  that the API gives, without deciding the richer API question. The first
  pass recommended A.
- **Action:** fix after merge, apply C, because no hosted writer saves these
  facts yet. Complete it before a hosted writer of job cache facts lands.
