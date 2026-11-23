# Savings chart marks ignore the shared tokens

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 1205, with the
  rounded bar tops and the chart frame differences that its check found.
- **Source:** Review.
- **Category:** Mockup.
- **Location:** `ts/app/src/screens/savings/starts-chart.tsx`,
  `StartsPanel`, `docs/mockups/src/components/starts-chart.tsx`,
  `docs/mockups/src/components/starts-chart.css`, `ts/tokens/firna.ts`,
  `theme`, `ts/tokens/index.ts`, `markRadii`, `ts/tokens/README.md`, and the
  `@firna/ui` range in `ts/app/package.json` and `ts/mockups/package.json`.
- **Reason:** D needs a new release of `@firna/ui`, a library outside this
  repository, and the severity is Low.

- **Terms:**
  - **Firna:** `@firna/ui`, the shared component library. The dashboard
    draws the Savings chart with its `BarChart`.
  - **Mark radii:** `markRadii` in `ts/tokens/index.ts`: the 1, 2, and 3 px
    corners of small data marks. Examples are slot segments, chart keys,
    and the data ends of thin bars.
  - **Legend key:** the small coloured square beside the name of a chart
    series.
  - **Firna theme:** the theme that `ts/tokens/firna.ts` builds from the
    colours, fonts, and radii of the tokens. The dashboard and the mockups
    both use it.
  - **`CHART_MARKS`:** the fixed mark values of the Firna chart family,
    such as the radius of a bar end.
- **Background:**
  - The Savings page shows the starts of each day as stacked bars, for own
    machines and for Boat. `StartsPanel` in
    `ts/app/src/screens/savings/starts-chart.tsx` draws it with the Firna
    `BarChart`.
  - The mockup draws its own chart with SVG and CSS, in
    `docs/mockups/src/components/starts-chart.tsx` and `starts-chart.css`.
    It dates from 3 November 2026. The dashboard took the Firna chart on 8
    November 2026.
  - The mockup draws 8 px legend keys with the 2 px corner `--r-mark-md`,
    and square bars. It puts the legend and the caption in one footer row.
    Its labels use the ink2 colour with a 16 px gap, and its data table
    stays hidden.
  - Firna 4.0.0 draws 10 px legend keys with 3 px corners, in
    `ChartLegend`. Its public API has no setting for the key size or
    corner. All five Firna charts with a legend use `ChartLegend`.
  - Firna rounds the data end of each bar by its fixed 4 px
    `CHART_MARKS.barRadius`.
  - The Firna chart frame puts the caption above the plot and adds a "Show
    data table" button. It draws the legend labels in the muted ink with a
    14 px gap.
  - The Firna theme reads only `radii` today. `ts/tokens/README.md` and the
    comment of `markRadii` in `ts/tokens/index.ts` say so.
  - `AGENTS.md` makes `@ciproxy/tokens` the one source of the colours,
    fonts, and radii of the product look.
  - A user decision of 4 November 2026 adds no gate or test that scans for
    literal radii. The round 1 fix of the same review kept the Firna
    `radii` map unchanged and moved the other small mockup marks to
    `markRadii`.
  - ciproxy takes `@firna/ui` at `^4.0.0`. npm already has 5.0.0 and 5.1.0,
    and 5.1.0 still fixes the key size in code. So a Firna change needs a
    4.x release, or the upgrade to 5.x.
  - The chart keeps a legend for its two series, so that no value lives in
    colour alone, as the comment of `starts-chart.tsx` says.
  - The mockups render Firna controls through the Firna DOM backend. The
    Firna node build draws charts as plain DOM SVG.
- **Problem:** The dashboard chart differs from the approved mockup, and no
  token sets its marks:
  - Legend keys: 10 px with 3 px corners, not 8 px with the 2 px mark
    corner.
  - Bars: a fixed 4 px round data end, not a square end.
  - Frame: the caption above the plot, a "Show data table" button, and
    muted labels with a 14 px gap. The mockup has one footer row, ink2
    labels, and a 16 px gap.
- **Impact if no action:** The dashboard keeps a chart look that differs
  from the mockup. A later change of `markRadii` cannot reach the legend
  corners or the bar ends.
- **Root question:** Which source sets the chart marks that the dashboard
  and the mockup share, and how do both follow it?
- **Options:**
  - **A.** Accept the fixed Firna shape. Change the mockup to 10 px keys
    with 3 px corners, and record the exception to the token corners.
    Effort: Small.
  - **B.** Add key settings to the public API of Firna, and pass the
    approved size and mark corner from the dashboard chart. Cost: each
    chart call must pass the values, and only those calls follow them.
    Firna needs a release. Effort: Medium.
  - **C.** Turn off the built-in legend, and draw a dashboard legend from
    the tokens. Keep the series toggles and the table view of the chart.
    Effort: Medium.
  - **D.** Add a chart-mark group to the Firna theme. Its defaults are the
    current Firna values.
    - `ChartLegend` reads the key size and corner from the group, so all
      five Firna charts with a legend follow it.
    - The group also holds the bar end, now the fixed 4 px of
      `CHART_MARKS.barRadius`.
    - Set the group once in `ts/tokens/firna.ts`. Use the 8 px key with the
      corner `markRadii.md`. Take the bar end from the tokens, so that it
      matches the square bars of the mockup.
    - Update `ts/tokens/README.md` and the comment in `ts/tokens/index.ts`.
      Both say that the Firna theme reads only `radii`.
    - The dashboard chart code does not change.
    - Cost: a shared Firna theme type changes, and the release needs a 4.x
      release or the 5.x upgrade. B has the same cost. D does not change
      the chart frame.
    - Effort: Medium.
  - **E.** Draw the mockup starts chart with the Firna `BarChart`.
    - Use the series, colours, and caption of the dashboard, with a fixed
      `defaultWidth` for each variant. The mockup then shows the product
      legend, bars, and frame by construction.
    - E drops the assumption that the mockup keeps its own chart. Its
      reason is gone: the mockup chart only predates the Firna chart.
    - Cost: the approved Graphite chart changes to the Firna look. That look
      has 10 px keys with 3 px corners and round 4 px bar ends. It puts the
      caption above the plot and adds a "Show data table" button. The
      tokens set the corners only when D is also done.
    - Effort: Medium.
  - **F.** Override the key style in the dashboard with a scoped web style
    rule, built from `markRadii.md` and the 8 px size, through the root
    `Head`. Cost: the rule depends on the inner element order and the
    inline styles of Firna, and it needs `!important`. A Firna release can
    undo it with no error. It is also the first stylesheet of the app.
    Effort: Small.
- **Left out:**
  - A new Firna default of 8 px keys with 2 px corners: the value would
    then live in Firna, not in `@ciproxy/tokens`. That breaks the token
    rule of `AGENTS.md`.
  - Removing the legend: it fails the goal, because the legend tells the
    reader the two series apart.
  - No change: it fails the goal, because the `mockups` skill keeps the
    mockups aligned with the implementation.
  - A dashboard chart of its own in place of the Firna `BarChart`: not a
    real choice. `AGENTS.md` puts `@firna/ui` first. The Firna chart covers
    the plot, and C already replaces the one part that Firna cannot shape.
- **Wider fix:** D. With B, each chart call must pass the values. D sets
  the marks in the theme that the token package already builds. D together
  with E also removes the second drawing of the chart in the mockup.
- **Previous examples:** Round 1 of the same review found small mockup
  marks with literal corners. Commit `3428fd3` of 17 November 2026 moved
  them to `markRadii`, but no option named the legend key that Firna draws.
- **Related:** `plans/follow-ups/firna-ui-label-weight-token.md`: the same
  class, a fixed Firna value that the theme cannot set. Its fix adds a
  label-weight token to the Firna theme, so both changes can share one
  Firna release. Neither item waits for the other.
  `plans/follow-ups/savings-chart-render-check.md`, the same chart: its
  smoke check counts the drawn bars, so a fix here that draws the bars
  with another element updates that count.
- **Recommendation:** D, changed from B. D needs the same Firna release as
  B. Then the token package sets the key and the bar end once for every
  Firna chart in the dashboard and the mockups. So a new chart or a token
  change cannot bypass `markRadii`.
- **Action:** fix after merge, apply D, because it needs a release of
  `@firna/ui`, and the severity is Low.
  - D aligns the legend keys and the bar ends with the mockup.
  - D leaves the frame differences. E aligns them, but it changes the
    approved chart to the Firna frame. A choice between the mockup and the
    product needs the user, so ask the user before a change aligns the
    frame.
