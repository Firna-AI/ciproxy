# Local fallbacks hide better GitHub matches

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from PR #10, orchestrator finding 4.
- **Source:** Review.
- **Category:** product bug.
- **Location:** `crates/ciproxy-cache-sidecar/src/download.rs:41`,
  `crates/ciproxy-cache-sidecar/src/download.rs:116`, and the control
  plane's future trusted cache metadata boundary.
- **Reason:** The complete resolver needs trusted upstream scope data and
  control-plane integration outside the cache-sidecar diff.

- **Terms:**
  - **Machine cache:** Persistent cache storage on a customer's machine.
  - **GitHub cache:** The upstream cache service used by GitHub Actions.
  - **Cache scope:** The repository and ref that permit a job to restore
    an entry.
  - **Restore key:** An ordered fallback key prefix for a cache lookup.
  - **Fill:** A retained local copy downloaded during a GitHub restore.
- **Background:** GitHub cache lookup orders entries by allowed branch
  and key. The job's own branch precedes its pull request base and default
  branch. Each scope searches exact keys before prefix matches, in the
  requested key order. Different machines can hold different entries.
  ciproxy also retains entries after GitHub deletes or evicts them.
- **Problem:** Both current lookup paths return a local candidate before
  asking GitHub for a better one. A default-branch prefix on the machine
  can therefore hide an exact primary-key match on the job's own branch.
  Comparing key strings cannot establish which branch has precedence.
- **Impact if no action:** Jobs restore older fallback contents while a
  higher-priority entry is available on GitHub. The selected entry depends
  on which machine runs the job.
- **Evidence:** An isolated executable lookup expected `deps-exact` from
  the feature branch. The machine instead returned default-branch
  `deps-old` with contents `old-default`.
- **Requirement:** `docs/protocol/cache.md`, "Cache scopes", requires the
  same keys, versions, and allowed scopes as GitHub. The existing key and
  branch ordering must apply across both content stores. Retained local
  entries must remain available after upstream deletion or eviction.
- **Root question:** How can lookup preserve key and branch priority
  across the machine cache and the GitHub cache?
- **Options:**
  - **A.** Obtain trusted upstream scope metadata when a local fallback
    could lose to another candidate. Compare the candidates before
    returning the fallback. This retains separate lookup paths.
    Effort: Large.
  - **B.** Use one resolver for local and GitHub candidates with trusted
    scope identities. Apply the existing ordering once to the combined
    candidates. The control plane must supply a trusted source of upstream
    entry scope. Persist the identities needed for retained entries.
    Effort: Large.
  - **C.** Maintain a shared cache metadata index across machines and
    GitHub. Resolve through that index. This adds persistent shared state
    and its lifecycle. Effort: Large.
- **Left out:** GitHub-first lookup can hide a retained entry from a
  higher-priority local branch. Key-only comparison cannot determine
  branch priority. Guest-supplied scope metadata is not trusted.
- **Wider fix:** B removes separate ordering decisions for both stores.
- **Previous examples:** None found in the searched primary history or
  plans. The related fill-provenance defect returned `default-value`
  after an accepted feature-branch save of `feature-value`. Its separate
  fill ownership fix does not establish upstream candidate scope.
- **Related:** PR #10's fill ownership fix prevents a restored copy from
  claiming an immutable branch save key. It does not resolve this ordering
  problem. Trusted upstream scope identities could address both causes.
- **Recommendation:** B. One resolver preserves the required ordering
  without a separate shared metadata index.
- **Action:** fix after merge, apply B, because this Medium, Large fix
  needs a trusted scope source and control-plane changes outside the diff.
- **Done when:** Both protocol paths select a feature-branch exact entry
  over a default-branch local prefix. Tests also preserve higher-priority
  retained entries when GitHub has only a lower-priority candidate. The
  resolver accepts no job-provided scope authority.
