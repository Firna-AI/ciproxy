# Free after failed payments lacks pace notice

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-05, from the implementation handoff for PR #29.
- **Source:** Implementation.
- **Category:** Mockup.
- **Location:**
  `docs/mockups/src/screens/billing/usage-notices.tsx:42`,
  `docs/mockups/src/fixtures/usage.ts:89`,
  `docs/mockups/src/screens/billing/plan-usage.tsx:68`,
  `docs/mockups/src/entries/app/billing/payments/states.mockup.tsx:56`, and
  `docs/protocol/metering.md:130`.
- **Reason:** This missing notice predates the banner for a Free workspace
  with several organizations.

- **Terms:**
  - **Start:** One job taken by a ciproxy runner, on an own machine or Boat.
  - **Pace notice:** The Plan and usage notice that gives the date when
    expected starts pass the included starts, with the offer.
  - **Free:** The tier that includes 2,000 starts per monthly period.
- **Background:** `docs/protocol/metering.md` requires a notice when
  current starts are below the limit and expected starts are above it.
  This requirement also applies after failed payments end a paid tier.
  `docs/protocol/billing.md` keeps one global banner for an ended tier.
  The pace notice belongs inside Plan and usage, below that banner.
- **Problem:** `usageNotice` admits only `free` and `free-organizations`
  to its Free pace branch. The `moved-to-free` fixture has 420 starts,
  13,400 expected starts, and a limit-passing date of 23 December.
  Its Free period runs from 18 December to 17 January. The current
  predicate returns no pace notice for this scene, despite those values.
- **Impact if no action:** The Free mockup after failed payments omits the
  required forecast and offer before starts pass its limit.
- **Root question:** How should Plan and usage show the required pace
  notice after failed payments move a workspace to Free?
- **Options:**
  - **A.** Add `moved-to-free` to the existing scene condition. Reuse the
    current notice and its fixture values. Effort: Small.
  - **B.** Select the Free pace notice from the usage tier, current starts,
    expected starts, included starts, and available crossing date. Reuse
    the existing notice, offer, and payment banner. Effort: Small.
- **Left out:** Add a separate notice in `PlanUsageScreen`, because A
  uses the existing renderer with fewer changes and no duplicate copy.
  Suppress forecasts after failed payments, because it omits required
  information. Only A and B repair the existing shared notice.
- **Wider fix:** B. A condition based on usage replaces a list of scene
  names, so another qualifying Free scene needs no separate exception.
- **Previous examples:** The `free-organizations` scene already shows
  this notice below its banner for the organization limit. The same
  predicate checks that scene by name.
- **Recommendation:** B. Every qualifying Free scene then uses the same
  existing notice without a new exception for each scene.
- **Action:** fix after merge, apply B. This pre-existing payment scene
  sits outside the completed organization-banner TODO. Preserve the
  one banner for the ended tier and the payment actions of each role.
  Do not show a pre-limit forecast after starts have passed the limit.
  Run `npm --prefix ts run mockups:check` in the later change. Inspect the
  affected screen at desktop widths of 1440 and 720 pixels, and
  mobile width of 390 pixels.
