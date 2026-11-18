# Machine caches miss GitHub cache deletions

- **Severity:** Low. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from the answer of the user to item 18 of
  `plans/ciproxy-mvp-foundation.md`. On 2026-11-17, the final review of the
  branch `calummoore/ci-proxy-load-balancer` corrected option B after the
  check of its finding 1207: the App already has "Actions" read.
- **Location:** `docs/protocol/cache.md`, "Cache scopes", the cache
  sidecar of the implementation plan, and the GitHub cache entry list,
  `CacheEntries::cache_entries` in
  `crates/ciproxy-github/src/rest/cache_metadata.rs`.
- **Reason:** The user chose to keep machine cache entries with a "Clear
  cache" action for the MVP, and asked for this sync as a follow-up.

- **Terms:**
  - **Machine cache:** the cache disk of an own machine, which sits in front
    of the GitHub cache that `actions/cache` uses.
- **Background:** A cache save goes to the machine cache and to the GitHub
  cache. On 4 November 2026 the user chose that a machine keeps its entries
  after GitHub evicts them, and that the dashboard gets "Clear cache".
  "Cache scopes" in `docs/protocol/cache.md` also says that the machine
  cache keeps its copy of an entry that a person deletes on GitHub. The App
  has had the "Actions" read permission since 4 November 2026. The control
  plane already lists the GitHub cache entries of a repository with it, in
  `CacheEntries::cache_entries`, for the restore order of the cache.
- **Problem:** When a person deletes a broken entry on GitHub, a machine can
  still serve its own copy until someone selects "Clear cache".
- **Impact if no action:** A deleted broken entry can come back on own
  machines until a person clears the machine cache.
- **Root question:** How does a machine cache follow the deletions that
  people make on GitHub?
- **Options:**
  - **A.** Keep the manual "Clear cache" only. Effort: Small.
  - **B.** Sync deletions from the GitHub Actions caches API. The App has
    had the "Actions" read permission since 4 November 2026, and the control
    plane already lists the GitHub cache entries with it. So the sync needs
    no new App permission and no new list adapter. Until 17 November 2026
    this option said that the App had runner access only, and that
    "Actions" read would be a new permission. The sync cannot tell a
    deletion from an eviction. So it drops a machine entry only when GitHub
    drops it and the entry is newer than GitHub's eviction window.
    Effort: Medium.
- **Left out:** A time limit on each entry, because it also drops good
  entries.
- **Wider fix:** B.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/job-facts-from-the-hook.md`: it asks whether the App
    can drop the "Actions" read permission. B of this item syncs through
    the entry list of the GitHub cache, which needs that permission, so a
    drop of the permission would block B. Neither item waits for the other,
    but their answers must agree.
  - `plans/follow-ups/follow-up-decision-sync.md`: this item is the second
    case of that class. Its old option B kept a fact that a later decision
    had replaced. Neither item waits for the other.
- **Recommendation:** B, after the MVP.
- **Action:** ask the user, because B changes a rule of "Cache scopes" in
  `docs/protocol/cache.md`. The rule says that the machine cache keeps its
  copy of an entry that a person deletes on GitHub. It comes from the
  answer of the user of 4 November 2026.
