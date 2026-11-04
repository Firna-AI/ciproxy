# Machine caches miss GitHub cache deletions

- **Severity:** Low. **Effort:** Medium. **Confidence:** Medium. **Harder
  later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from the answer of the user to item 18 of
  `plans/ciproxy-mvp-foundation.md`.
- **Location:** `docs/protocol/cache.md`, "Cache scopes", and the cache
  sidecar of the implementation plan.
- **Reason:** The user chose to keep machine cache entries with a "Clear
  cache" action for the MVP, and asked for this sync as a follow-up.

- **Terms:**
  - **Machine cache:** the cache disk of an own machine, which sits in front
    of the GitHub cache that `actions/cache` uses.
- **Background:** A cache save goes to the machine cache and to the GitHub
  cache. On 4 November 2026 the user chose that a machine keeps its entries
  after GitHub evicts them, and that the dashboard gets "Clear cache".
- **Problem:** When a person deletes a broken entry on GitHub, a machine can
  still serve its own copy until someone selects "Clear cache".
- **Impact if no action:** A deleted broken entry can come back on own
  machines until a person clears the machine cache.
- **Root question:** How does a machine cache follow the deletions that
  people make on GitHub?
- **Options:**
  - **A.** Keep the manual "Clear cache" only. Effort: Small.
  - **B.** Sync deletions from the GitHub Actions caches API. The product
    App has runner access only, so the sync needs the "Actions" read
    permission. That is a new App permission, which the user must approve,
    and which each organization must then accept on GitHub. The sync cannot
    tell a deletion from an eviction, so it drops a machine entry only when
    GitHub drops it and the entry is newer than GitHub's eviction window.
    Effort: Medium.
- **Left out:** A time limit on each entry, because it also drops good
  entries.
- **Wider fix:** B.
- **Previous examples:** None found.
- **Recommendation:** B, after the MVP.
- **Action:** ask the user, because B needs a new permission of the GitHub
  App, which the user must approve.
