# Machine confirmations replace their original scene

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the owner review of pull request #71.
- **Source:** Review.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/screens/machines/machine-scenes.tsx` and
  `docs/mockups/src/screens/machines/machine-cache.tsx`, the scene selection
  and Clear cache destinations.
- **Reason:** These links already use the steady scene on the primary
  branch. The owner deferred them from the unknown-statistics correction.

- **Terms:**
  - **Scene:** the fixture machine and screen state that a mockup shows.
  - **Steady scene:** the healthy measured fixture of hetzner-ax42-1.
  - **Confirmation:** the Clear cache dialog over a machine page.
- **Background:** The machine mockups show healthy, draining, saved-offline,
  and agent-behind states. An Admin opens Clear cache from the Cache panel.
  A scene record supplies the machine, open confirmation, and navigation
  paths. The unknown-statistics scenes have their own confirmation context.
- **Problem:** The draining, saved-offline, and agent-behind scenes still
  link to the steady confirmation. That confirmation shows the steady
  machine instead of the machine state that opened it. Cancel and Clear
  cache then open pages of the steady scene.
  - `app/machines/machine/states/draining` shows Draining, Stop draining,
    and cache disk 83 of 420 GB. Its confirmation shows Healthy and cache
    disk 83 of 178 GB.
  - `app/machines/machine/changes/saved-offline` also loses its Saved and
    Draining notices when that confirmation opens.
  - `app/machines/machine/health/agent-behind` shows agent 0.4.1 with 0.4.2
    available. Its confirmation instead shows 0.4.2 as up to date.
  - Each opens `app/machines/machine/states/clear-cache`. Cancel opens
    `app/machines/machine`. Clear cache opens
    `app/machines/machine/changes/clear-requested`.
- **Impact if no action:** Reviewers lose the original machine state while
  they follow the cache action. The mockups cannot show that action in
  those three states.
- **Root question:** How should these cache actions preserve the machine
  scene that opened them?
- **Options:**
  - **A.** Add scene-specific confirmations and destinations for the three
    remaining states. Use the existing scene records. Cancel returns to
    the same page. Clear acts in place unless that scene has a result
    page. Cost: more catalogue entries. Effort: Small.
  - **B.** Show plain Clear cache buttons in these states until their
    confirmations exist. This avoids wrong navigation but leaves the
    journeys incomplete. Effort: Small.
- **Left out:** A separate client-side dialog system would duplicate the
  static catalogue. The existing scene records already carry the required
  context, so these are the two useful approaches.
- **Wider fix:** A. The scene record keeps each machine and its dialog
  destinations together. No new navigation mechanism is needed.
- **Previous examples:** The cache-unknown, docker-unknown, and
  both-unknown machine scenes use the same record structure. The
  organization mockups keep their context in
  `docs/mockups/src/screens/settings/organization-scenes.tsx`.
- **Recommendation:** A. It completes the journeys while preserving their
  machine context.
- **Action:** fix after merge, apply A, because the owner kept these
  pre-existing scene links outside the unknown-statistics fix.
