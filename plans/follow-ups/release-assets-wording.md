# Agent release guide calls public assets private

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 1105 found.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `crates/ciproxy-agent/updates-and-releases.md`, the
  sentence "The release server needs access to the private release
  assets."
- **Reason:** The sentence predates the final review, and no release step
  depends on it.

- **Terms:**
  - **Release server:** the `cli-release-server` service that tells the
    installed agents about new releases. It reads the releases of this
    repository through the GitHub API.
  - **Release assets:** the archives of the agent and the cache sidecar
    that each agent release attaches on GitHub.
- **Background:**
  - `crates/ciproxy-agent/updates-and-releases.md` says: "The release
    server needs access to the private release assets." Commit `d6d18e6`
    wrote it on 5 November 2026.
  - On 5 November 2026 the repository became public, as the user chose.
  - "Store the release token" in `deploy/setup.md` says that
    `Firna-AI/ciproxy` and its release assets are public. The release server
    still needs a token for its GitHub API configuration, with read-only
    access to public repositories and no private repository access.
- **Problem:** The agent release guide calls the release assets private,
  but they are public, as the deploy guide says.
- **Impact if no action:** A reader can give the release server a token
  with private repository access that it does not need.
- **Root question:** How should the agent release guide describe the access
  that the release server needs?
- **Options:**
  - **A.** Change the sentence: the release server reads the public release
    assets, and it needs a read-only token for public repositories only.
    Effort: Small.
  - **B.** Remove the sentence, and link to "Store the release token" in
    `deploy/setup.md`, which gives the token rules. Effort: Small.
- **Left out:** Only these two approaches fix the sentence. A change of the
  release access itself would answer another question.
- **Wider fix:** None. The `documentation` skill already asks each doc to
  match the current facts.
- **Previous examples:** None found.
- **Recommendation:** B. One place then gives the token rules, so the two
  guides cannot disagree again.
- **Action:** fix after merge, apply B, because the sentence predates the
  final review, and no release step depends on it.
