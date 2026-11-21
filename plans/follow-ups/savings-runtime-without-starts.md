# Zero starts hide real Savings runtime

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  finding 15 of its second review round, with two phrase defects that the
  check of that finding found in the same code.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/screens/savings/savings-screen.tsx:75`,
  `ts/app/src/view/savings-report.ts:48`,
  `ts/app/src/view/savings-report.ts:87`,
  `ts/app/src/view/savings-report.ts:156`,
  `docs/mockups/src/screens/savings/savings.tsx:117`,
  `docs/mockups/src/screens/savings/savings.tsx:132`, and
  `docs/mockups/src/screens/savings/savings.tsx:177`.
- **Reason:** The defect needs a page that opens after a job crossed the
  start of the range and before the next start, and the fix costs the same
  later.

- **Terms:**
  - **Savings:** the dashboard page that compares what the jobs cost with
    ciproxy against the same jobs on GitHub-hosted runners and on
    Blacksmith. It shows This month, Last month, or the last 30 days.
  - **Start:** the moment when a ciproxy runner takes a job. A start counts
    on the UTC date of that moment.
  - **Runtime:** the exact minutes that jobs ran. The report splits the
    runtime of a job at each UTC midnight.
  - **Content model:** one view function that decides what the report
    shows: the empty report, or the report with its figure and tables, and
    each count and range phrase.
- **Background:**
  - The metering report keeps each start on its taken-at date, and it
    splits runtime at UTC midnight, at
    `crates/ciproxy-metering/src/report/runtime.rs:14-18`. The test at
    `crates/ciproxy-metering/src/_tests_/runtime_tests.rs:30-36` proves it.
    A job from 23:59:30 on 4 November to 00:01 on 5 November gives
    5 November 0 starts and 1 minute.
  - `docs/protocol/metering.md:52` counts a start in the period in which a
    runner took the job. `docs/protocol/metering.md:191` says that the
    report shows the starts and the minutes on own machines and on Boat.
  - The API sends the starts and the minutes of each place.
    `crates/ciproxy-api/src/wire/savings.rs:48-49` sends the minutes as an
    exact decimal string, without invoice rounding.
  - The screen shows the empty report whenever the starts of both places
    add up to zero, at `ts/app/src/screens/savings/savings-screen.tsx:75-92`.
    `totalStarts` at `ts/app/src/view/savings-report.ts:48-50` adds only
    the starts. The mockup makes the same choice at
    `docs/mockups/src/screens/savings/savings.tsx:177-183`.
  - The empty report of This month or Last month says "No jobs ran this
    month" or "No jobs ran last month", with "Choose another range to see
    your savings.", at `savings-report.ts:163-167`.
  - The savings figure uses the start count as the job count, as in "less
    than Blacksmith for the same 17,760 jobs", at `savings-report.ts:80-87`
    and in the mockup at `savings.tsx:112-119`.
  - Two more phrases of the report are wrong:
    - An empty Last 30 days says "Your savings report starts with your
      first job", at `savings-report.ts:156-161` and `savings.tsx:145`. It
      says so also for a workspace whose earlier jobs still show under
      Last month. An empty range cannot prove a first job, and no API fact
      says whether a workspace ever ran a job.
    - A range with one start says "for the same 1 jobs", at
      `savings-report.ts:87` and `savings.tsx:117`.
  - `centsSign` at `ts/app/src/view/exact-format.ts:82-86` reads only the
    whole part of a decimal string. It is for whole cents. For minutes, it
    treats 30 seconds as zero.
  - The Savings mockup pages "States" and "Report states" show five screens
    each. The page "Facts of the current API", at
    `docs/mockups/src/entries/app/savings/current-contract/states.mockup.tsx`,
    shows one screen.
  - `ts/app/_tests_/savings-view.test.ts` has no case for an empty or a
    runtime-only range. The browser fixture at
    `ts/app/scripts/fixtures-savings.ts:77-82` sets the starts and the
    minutes to zero together.
- **Problem:** The page shows the empty report whenever both start counts
  are zero, also when the range has runtime. A job from 23:59 on
  31 October to 00:01 on 1 November gives This month 0 starts and 1 minute,
  and until the next start the page says "No jobs ran this month" and
  hides that minute and its costs. The two phrases above are also wrong.
- **Impact if no action:** A customer cannot see real costs of the selected
  range until another job starts, and the title says that no job ran. Last
  30 days is a workaround, but it shows a different range.
- **Root question:** How should Savings show work that began before its
  selected range?
- **Options:**
  - **A.** Require zero runtime as well as zero starts before the page shows
    the empty report. Add a regression with runtime from a start in the
    month before. The regression needs the choice in an exported view
    function, because no test under `ts/app/_tests_/` renders a component.
    Effort: Small.
  - **B.** Give the report view one content model that tells an empty range
    from a range with starts or runtime. Make the screen use that model,
    and add the same regression. Match the mockup to the runtime-only case,
    on a page with room. Effort: Small.
  - **C.** Show the cost and runtime tables after every successful read.
    Put the no-start guidance above those tables instead of returning
    early. This changes the empty-report design and needs its own mockup
    decision. Effort: Medium.
  - **D.** Apply B, and let the same content model give the savings figure
    its job phrase:
    - Show the report when the range has starts or minutes above zero.
      Keep the empty report only when the starts and the minutes are both
      zero.
    - Compare the exact decimal minutes. Do not use `centsSign`, because
      it reads only whole cents and treats 30 seconds as zero.
    - Keep the full runtime and cost report for jobs that started before
      the range.
    - Leave the job count out of the figure when the range has no starts,
      so that the figure never says "for the same 0 jobs".
    - Correct the two phrases above in the same model and in the mockup,
      with no lifetime flag and no new API figure. An empty Last 30 days
      says only what the range shows. One start reads "1 job".
    - Add the runtime-only state to the Savings mockup first, on "Facts of
      the current API" or on a new linked sub-page, because "States" and
      "Report states" show five screens each.
    - Then add the report and figure regressions to
      `ts/app/_tests_/savings-view.test.ts`.
    - Effort: Small.
  - **E.** Let the API count the jobs that ran in the range: its starts,
    plus the jobs that began earlier and still ran in it. Use that count
    for the empty choice and the savings figure. It needs a new anonymous
    daily fact, a store migration, metering and wire changes, generated
    types, a protocol line, and a mockup. It adds a requirement, so it
    needs a user decision. Effort: Large.
- **Left out:**
  - A report that drops the runtime of jobs that began before the range,
    because the range then loses real minutes and alternative costs.
  - A report that counts the whole runtime of a job on its taken-at date,
    as starts count, because the selected range still hides the minutes
    that ran in it.
  - A server flag for an empty range, because the page already gets the
    starts and the minutes of each place. A flag adds a contract change and
    no fact.
- **Wider fix:** D. The content model owns both the visibility of the
  report and its count and range phrases, so no screen uses the start count
  as a job count by itself.
- **Previous examples:** The midnight test at `runtime_tests.rs:10` keeps
  starts and runtime apart in the backend. No earlier frontend defect of
  this class was found.
- **Recommendation:** D. The same content model keeps the runtime visible
  without the claim "the same 0 jobs", which B alone would give in the
  savings figure. The first pass recommended B.
- **Action:** fix after merge, apply D, because a range with runtime but no
  new starts is Rare, and the fix costs the same later.
