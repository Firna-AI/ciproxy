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
- **HTTP evidence, 2026-11-07:** Both public protocol paths reproduced the
  early local return with a checked feature-branch session.
  - The v1 assertion expected `cacheKey` to equal `deps-exact`.
    The response gave `deps-old`.
  - The v2 assertion expected `matchedKey` to equal `deps-exact`.
    The response gave `deps-old`.
  - Each case saved a default-branch local prefix and made an exact
    upstream candidate available. Each failure came from its assertion.
  - A separate guard saved `deps-retained` in the feature scope, removed
    its upstream copy, and restarted the machine service. Both protocols
    restored `retained-feature` from that local prefix while an upstream
    exact candidate remained available.
  - The upstream fixture had no ref metadata index. These checks proved
    the ordering fault and retained local behavior. They did not prove
    trusted upstream source-ref binding.
- **Current integration, 2026-11-07:** The machine has no trusted
  connection to hosted cache entry metadata.
  - `JobFacts` and `JobScope` live in
    `crates/ciproxy-protocol/src/jobs.rs`. `JobFacts` identifies the job.
    `JobScope` holds the machine-checked readable refs. Neither identifies
    an upstream entry's ref.
  - `CacheRegistration` and `RuntimeCredentials` live in
    `crates/ciproxy-protocol/src/runtime.rs`. They carry the original
    GitHub cache credentials and URLs. They carry no hosted metadata
    endpoint or metadata credential.
  - `AssignmentService` in `crates/ciproxy-github/src/assignments.rs`
    supplies `JobFacts` from repository snapshots and saved job facts.
    Its `AssignmentStore` in
    `crates/ciproxy-github/src/persistence/assignment.rs` supplies no cache
    entry metadata.
  - `crates/ciproxy-control-plane/src/events.rs` sends those facts through
    the gateway. The gateway has no cache metadata request or response.
  - `crates/ciproxy-cache-sidecar/src/download.rs` decodes matched keys
    and download URLs from GitHub. Its legacy `LegacyEntry` type in
    `crates/ciproxy-cache-sidecar/src/wire.rs` omits the optional `scope`
    and `creationTime` fields that the
    [upstream contract][legacy-entry] declares. The v2 response has no
    entry ref. The combined resolver still needs trusted metadata.
    `crates/ciproxy-cache-sidecar/src/blobs.rs` retains fills within the
    reader's writable or private scope.
- **Scope dependency, 2026-11-07:** One concrete integration route needs
  these existing files, plus new metadata service code:
  - `crates/ciproxy-control-plane/src/composition.rs` would construct a
    hosted metadata service with App, HTTP, store, and runtime boundaries.
    It could mount the service through `Hosted::router` or inject it into
    a gateway channel. Concurrent control-plane work protected this file
    during the implementation attempt.
  - `crates/ciproxy-agent/src/application.rs` would supply the trusted
    hosted connection and host identity boundary to the cache integration.
    Its current `HttpCache::new` call supplies only `Commands`.
  - `crates/ciproxy-agent/src/cache.rs` would connect that channel to each
    checked cache session. Its current registration carries no hosted
    metadata access.
  - `crates/ciproxy-agent/src/cache_admin.rs` would carry host-only
    metadata configuration or responses through the sidecar's admin
    listener.
  - This is one concrete route. API route alternatives can avoid the
    protected composition file. They still need agent transport outside
    this item's named cache-sidecar and control-plane area.
  - The implementation scope excluded agent changes. These integration
    changes need scope and concurrent ownership coordination. Completing
    the concurrent control-plane work alone does not supply the channel.
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

[legacy-entry]: https://github.com/actions/toolkit/blob/main/packages/cache/src/internal/contracts.ts
