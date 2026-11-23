# One rule for a label that runs at full size

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch `calummoore/copying-llm-xioi`,
  review round 2, the wider fix of finding 9.
- **Location:** `ts/app/src/view/setup-agent-prompt.ts`, the function
  `fullSize`. `ts/app/src/view/settings-label-fit.ts:110`, the function
  `boatOnly`.
- **Reason:** The wider fix reaches the Settings view code, which is
  outside the change that found it.

- **Terms:**
  - **Label fit:** the `fit` of each label in the Setup and Settings
    responses. Its `placement` is `machines_then_boat`, `machines_only`,
    `boat_only`, `waits`, or `unavailable`. Its `boat.state` is
    `largest_fallback` when only the largest Boat size runs the label, and
    that size is smaller than the label.
  - **Agent prompt:** the text that "Copy prompt for your agent" in the
    first-job step of Setup copies for a coding agent.
- **Background:** The agent prompt lists only labels that some destination
  runs at their full size. Its private function `fullSize` is false when
  the placement is `waits`, or when the placement is `boat_only` and
  `boat.state` is `largest_fallback`. Settings reads the same
  `largest_fallback` state inline in `boatOnly` to describe the label.
- **Problem:** Two places read the same fit facts to answer "does a job of
  this label get its full size?". A later change to the rule can reach one
  place and not the other.
- **Impact if no action:** Setup and Settings can disagree about whether a
  label runs at full size.
- **Root question:** Where does the app decide that a label runs at its
  full size?
- **Options:**
  - **A.** Keep the two places. Effort: Small.
  - **B.** Move `fullSize` to a shared label view module, such as
    `ts/app/src/view/settings-label-fit.ts`, and use it in the prompt and in
    `boatOnly`. Effort: Small.
- **Left out:** A new placement value in the API duplicates data that the
  API already returns in `boat.state`.
- **Wider fix:** B.
- **Previous examples:** None found.
- **Recommendation:** B. One rule serves both screens.
- **Action:** fix after merge, apply B, because it changes Settings code
  outside the change that found it.
