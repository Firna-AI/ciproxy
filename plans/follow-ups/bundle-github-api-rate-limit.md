# Boat bundles can hit the GitHub API limit

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-20, from `conductor/m27a-boat-native-image`, review
  round 2, a new problem that the check of the metadata-cache finding
  found.
- **Location:** `image/pin_recipe.py` (`warm_github_cache`) and
  `image/boat-build.py` (`bundle`).
- **Reason:** The branch moved the release answers out of the recipe tree;
  the number of host calls needs its own small change.

- **Terms:**
  - **Bundle:** the archive that `image/boat-build.py bundle` writes for a
    native Boat build. It holds the rendered recipe, the build plan and
    the ciproxy programs.
  - **Release answers:** public GitHub API responses about releases, which
    the bundle step fetches on the host so that the Boat sandbox needs no
    GitHub call for them.
- **Background:** Without `--metadata-cache`, each bundle fetches about 17
  release answers from the GitHub API with no credential. GitHub allows 60
  such calls an hour for each address.
- **Problem:** A fourth bundle within one hour from the same host can stop
  with HTTP 403.
- **Impact if no action:** An operator who builds several bundles in a row
  must wait, or pass `--metadata-cache` with a folder of earlier answers.
- **Root question:** How should repeated bundle builds on one host avoid
  the unauthenticated GitHub API limit?
- **Options:**
  - **A.** Keep a host cache of release answers between bundle runs, keyed
    by the request, and reuse it before a new call. Effort: Small.
  - **B.** Use a host GitHub credential for the bundle calls when one is
    present, and never put it into the bundle. Effort: Small.
  - **C.** Document the limit and the `--metadata-cache` option only.
    Effort: Small.
- **Left out:** Fetching the answers in the Boat sandbox, because each
  sandbox then makes the same calls and can hit the same limit.
- **Wider fix:** A. It removes repeated calls for every bundle.
- **Previous examples:** None found.
- **Recommendation:** A. It needs no credential and keeps the bundle
  reproducible.
- **Action:** fix after merge, apply A, because the limit only affects
  repeated bundle builds within one hour.
