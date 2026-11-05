# Overflow flow and report dates disagree

- **Severity:** Low. **Effort:** Medium. **Confidence:** High. **Harder later:**
  No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-05, from the re-review of PR #19.
- **Source:** Review.
- **Category:** Mockup.
- **Location:**
  `docs/mockups/src/entries/user-flows/peak-overflow-to-boat.mockup.tsx:21` and
  `docs/mockups/src/fixtures/savings.ts`.
- **Reason:** The release date and report range mismatch both exist in the base
  of the calendar-day cost change.

- **Terms:**
  - **Overflow flow:** The mockup sequence in which a release fills the
customer's machines and jobs run on Boat.
- **Background:** The mockups show one consistent workspace story, as
  `docs/mockups/README.md` requires. The overflow flow ends on the Savings page
  to show what its jobs cost. Its description names a release on 18 November
  2026. The Savings fixture covers 19 October through 17 November 2026
  inclusive.
- **Problem:** The final report ends before the release that the flow describes.
  It cannot show the costs of those release jobs. The range and release date
  come from separate literals.
- **Impact if no action:** A reviewer sees a report that cannot substantiate the
  flow's conclusion about the release cost.
- **Root question:** How should the overflow flow keep its release and report in
  one consistent time range?
- **Options:**
  - **A.** Align the relevant fixture dates and flow text so the Savings range
includes the release. Verify every frame and cost caption against the chosen
fixture range. Effort: Small.
  - **B.** Do A and give the flow's release and report one shared fixture date
source. Derive the report bounds and displayed date labels from it. Check
existing consumers before choosing its owner. Effort: Medium.
- **Left out:** Remove the Savings step. It would no longer show what the
  overflow jobs cost. Change only one date without checking the other frames. It
  can leave the same inconsistency in job details. A doc-prose test cannot
  protect the rendered timeline. Two implementation approaches remain useful.
- **Wider fix:** B. One fixture date source removes independent release and
  report literals.
- **Previous examples:** The Savings fixture already derives its period label
  and monthly cost allocation from its first and last UTC dates. Its catalogue
  entry reads the cost fixture instead of repeating a literal total.
- **Recommendation:** B. It aligns the story and prevents the release and report
  dates from drifting apart again.
- **Action:** fix after merge, apply B. This pre-existing mockup issue is
  outside the calendar-day cost change.
