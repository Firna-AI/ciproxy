# Savings smoke passes when the chart draws nothing

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, a problem
  outside the scope that the check of finding 509 found.
- **Location:** `ts/app/scripts/browser-pages-savings-screens.ts:26-29`, in
  `visibleCostRows`, and `ts/app/src/screens/savings/starts-chart.tsx:21`.
- **Reason:** The branch that found the gap does not change the Savings
  page or its smoke.

- **Terms:**
  - **Savings smoke:** the browser check of the Savings page in
    `ts/app/scripts/browser-pages-savings-screens.ts`. It opens each
    Savings case of the fixture harness in Chrome.
  - **Starts chart:** the "Starts by day" panel of the Savings page. It
    shows the job starts of each day as stacked bars, for own machines and
    for Boat (`ts/app/src/screens/savings/starts-chart.tsx`).
  - **Firna `BarChart`:** the bar chart of the shared library `@firna/ui`,
    version 4.0.0 in the dashboard. It draws each bar as an SVG `path`
    with the fill colour of its series.
- **Background:**
  - `visibleCostRows` checks the cost rows of the report. It then checks
    that the "Starts by day" region and the cost table have visible boxes
    that do not overlap (`ts/app/scripts/browser-pages-savings-screens.ts:26-29`,
    `separateRegions` in `ts/app/scripts/browser-pages-helpers.ts:123-136`).
  - Nothing reads the chart inside that region. The `Panel` gives the
    region its title, so the region keeps its box when the chart draws no
    bars.
  - `StartsPanel` draws the chart with the Firna `BarChart`. The series
    use `chartColors.own` for own machines and `chartColors.boat` for Boat
    (`ts/app/src/screens/savings/starts-chart.tsx:21-43`). `BarChart` takes
    a `testID` for its frame, and the panel passes none today.
  - The `Page` type of the browser driver has `evaluate` and `getByTestId`
    (`ts/app/scripts/browser-driver.ts:86` and `:91`), so a check can find
    the chart by its test ID and count its marks.
  - The browser smokes run only by hand. No gate phase and no CI step runs
    them, and `plans/follow-ups/dashboard-tests-outside-gate.md` waits for
    the user on that.
- **Problem:** The Savings smoke passes when the starts chart draws no
  bars, for example after a Firna release or a broken series. It checks
  only that the "Starts by day" region exists.
- **Impact if no action:** A Savings page with an empty chart passes every
  check, and a reader sees no starts.
- **Root question:** How does the Savings smoke prove that the starts chart
  draws its bars?
- **Options:**
  - **A.** Count the bars in the Savings smoke. Pass a `testID` to the
    `BarChart` of `StartsPanel`. In the report case, count the `path`
    elements of each series colour inside the chart with `page.evaluate`.
    Expect one bar of a series for each day on which the fixture has starts
    of that series. Cost: the check depends on the inner markup of the
    Firna chart, so a Firna release can break it with no product change.
    Effort: Small.
  - **B.** Add one shared check of the drawn series for all dashboard
    charts: a helper that each chart smoke uses to count the drawn marks of
    each series. It catches each chart that draws nothing. It is a test for
    the class, so it needs the user. Effort: Medium.
  - **C.** Compare a screenshot of the starts chart with a stored image. It
    also catches a wrong colour or a clipped label. Cost: each design,
    font, or Chrome change makes the image fail, and a person must judge
    each new image. It is a test for the class, so it needs the user.
    Effort: Medium.
- **Left out:**
  - A check of the accessible name or the table view of the chart, because
    both stay when the bars go, so it fails the goal.
  - Another model test of `startsSeries`, because the model stays right
    when the renderer draws nothing.
  - A test inside `@firna/ui`, because the library lives outside this
    repository, and the dashboard must still prove that its own chart call
    draws.
- **Wider fix:** B. It is a detection fix for every chart, and it waits for
  the user, because it is a test for the class.
- **Previous examples:** The CPU and memory panel of the job page had the
  same gap: its browser check read only the names and the text of the
  chart. Before its merge, the branch
  `calummoore/action-cpu-and-memory-usage` fixes it with one helper in
  `ts/app/scripts/browser-pages-job-usage.ts` that counts the drawn marks
  of the panel by kind.
- **Related:** `plans/follow-ups/savings-chart-firna-marks.md`, the same
  chart. Its fix changes the marks of the Firna chart, such as the bar ends
  and the legend keys. A count of the bar paths still holds after that
  fix. If that fix draws the bars with another element, update the count.
  Neither item waits for the other.
- **Recommendation:** A. It proves that the starts chart draws its bars, in
  the existing case and file. B best protects the codebase, because it also
  covers each later chart.
- **Action:** fix after merge, apply A, because the branch that found the
  gap does not change the Savings files.
