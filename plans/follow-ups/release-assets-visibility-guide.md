# Release guide calls public assets private

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the check of final-review finding 1105.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `crates/ciproxy-agent/updates-and-releases.md:30` and
  `deploy/setup.md:183`.
- **Reason:** The stale sentence predates the release-helper move.

- **Terms:**
  - **Release server:** the download service that serves the public agent
    and cache sidecar archives.
- **Background:** The repository and its release assets are public.
  The deployment setup guide states that access model.
- **Problem:** The agent updates guide says that the release server needs
  access to private release assets. This contradicts the setup guide.
- **Impact if no action:** An operator can assume that archive downloads
  need private-repository access.
- **Root question:** How should the agent guide describe release access?
- **Options:**
  - **A.** Correct the sentence to match the public asset access model.
    Effort: Small.
- **Left out:** Changing release access is a separate product change.
  No other wording change corrects this contradiction.
- **Wider fix:** None. The documentation skill already requires aligned
  guides.
- **Previous examples:** None found.
- **Recommendation:** A. It makes the operator guide match deployment.
- **Action:** fix after merge, apply A, because this sentence is outside
  the release-helper move.
