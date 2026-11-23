# History filter offers providers with no jobs

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, the dashboard screens
  work.
- **Location:** `ts/app/src/screens/jobs/history-filters.tsx`, and
  `docs/mockups/src/screens/jobs/history-filters.tsx`.
- **Reason:** The app and the approved mockup differ, and only the user
  decides which side is right.

- **Source:** Implementation.
- **Category:** Mockup.
- **Terms:**
  - **Place filter:** the filter of the job history that shows the jobs of
    all places, of own machines, or of one overflow provider.
- **Background:**
  - A workspace can connect several overflow providers, such as Boat.
  - The approved mockup gives the place filter one choice for each
    provider that ran jobs in the chosen time window. An empty history
    shows no provider choice.
  - The history API returns the jobs, but not the list of providers that
    ran jobs in the window.
  - So the app offers one choice for each provider of the workspace. With
    Boat as the only provider, this is Boat, as before the change.
- **Problem:** A provider with no jobs in the window still gets a filter
  choice, and an empty history still shows a provider choice.
- **Impact if no action:** With one provider, a customer sees a Boat choice
  that can show no jobs. With several providers, the filter can grow with
  choices that show nothing.
- **Root question:** Which providers does the history place filter offer?
- **Options:**
  - **A.** Keep one choice for each provider of the workspace, and change
    the mockup to match. Effort: Small.
  - **B.** Add the providers that ran jobs in the window to the history
    response, and offer only those, as the mockup shows. Effort: Small.
- **Left out:** Reading the providers from the jobs on the current page,
  because a page holds only part of the window.
- **Wider fix:** None, because one response field and one filter hold the
  rule.
- **Previous examples:** None found.
- **Recommendation:** B. It matches the approved design, and it fits the
  second-provider work in `plans/follow-ups/second-overflow-provider.md`,
  which adds the two-provider tests.
- **Action:** ask the user, because the user decides which side of a
  mockup and product difference is right.
