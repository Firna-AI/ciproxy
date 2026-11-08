# Retired label query has no caller

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-07, from PR #41, owner fix round 1 re-review, finding 3.
- **Source:** Review.
- **Category:** Dead code.
- **Location:**
  `crates/ciproxy-store-interface/src/dashboard/store.rs:124`,
  `crates/ciproxy-store-pg/src/dashboard/mod.rs:141`, and
  `crates/ciproxy-store-pg/src/dashboard/resources.rs:58`.
- **Reason:** The cleanup must change the dashboard store interface and
  implementation. The branch that records this item keeps every file of
  the concurrent dashboard continuation work unchanged.

- **Terms:**
  - **Retired label:** A deleted workflow label whose stored history stays
    available.
  - **Binding reconciliation:** The GitHub service operation that creates
    current scale sets and retires old ones.
- **Background:**
  - ciproxy uses GitHub runner scale sets for workflow labels.
  - `crates/ciproxy-github/src/app/bindings.rs` now owns reconciliation.
  - The replaced API creation and maintenance loops were the only callers
    of the retired-label query.
  - At `8b503af`, a source search for `retired_dashboard_labels` finds only
    its declaration and implementation.
  - A search for `resources::retired(` finds only that implementation.
  - PgStore is the only implementor. Both store packages set
    `publish = false`.
  - No product, test, script, generated API, or dynamic reference calls the
    query.
  - No annotation, feature flag, plan, or protocol requirement retains it.
- **Problem:** The store trait and PgStore still expose the unused
  `retired_dashboard_labels` method. Its `resources::retired` helper reads
  a list that the product never uses.
- **Impact if no action:** Maintainers must preserve an unused interface
  and query whenever their store types change.
- **Root question:** How should the codebase finish replacing the old API
  label cleanup path?
- **Options:**
  - **A.** Remove the trait method, PgStore method, query helper, and imports
    that become unused. Use a separate follow-up branch after the store
    files become available. Effort: Small.
- **Left out:** Keeping or renaming the query retains the unused path.
  Calling it from the new reconciler adds an unnecessary dependency.
  Only removal completes this cleanup. Deleting the helper alone breaks
  its caller in the protected implementation. The dashboard continuation
  work starts from code that still has the old callers. Adding this
  cleanup to that work would impose a merge order.
- **Wider fix:** None additional. The existing removal rule requires
  deletion of code whose last caller a change removes.
- **Previous examples:**
  `plans/follow-ups/runner-string-patch-removal.md` records another old
  path retained after its replacement.
- **Recommendation:** A. Removing the unused path best protects the
  codebase.
- **Action:** fix after merge, apply A, because the fix must change
  the dashboard store interface and implementation reserved for concurrent
  continuation work.

Preserve retired label rows, job history, and completion tracking. This
cleanup removes a read API, not retained data. Keep
`retired_label_workspaces` and `resources::retired_workspaces`, which
maintenance still uses.

The current removal leaves four unused imports:
`Label` in the interface, implementation, and resource module, plus
`LabelRow` in the resource module. Remove those imports. Keep the
`LabelRow` definition and the
`scale_sets` table import, which remain live. Recheck references on the
follow-up branch before deleting anything:

```sh
rg -n 'retired_dashboard_labels|resources::retired\(' crates
```

No migration or new regression test is needed for this dead-code removal.
Run the existing store and label tests, the deletion audit, and the
complete gate.
