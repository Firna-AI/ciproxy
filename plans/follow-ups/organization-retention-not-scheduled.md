# Organization retention has no production caller

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-07, from the independent finding check of the billing,
  savings, runner-exit, and Boat-registration follow-up branch.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-store-pg/src/organizations.rs:178`,
  `crates/ciproxy-store-pg/src/workspaces.rs:125`, and
  `crates/ciproxy-control-plane/src/accounting.rs:59`.
- **Reason:** Both missing callers predate the four selected fixes and
  need a separate runtime-retention change.

- **Terms:**
  - **Owner removal:** Removal of the GitHub App by an organization owner.
  - **Retention deadline:** The saved time after which removed data must
    be deleted.
  - **Accounting worker:** The hosted worker that repairs job timing,
    refreshes metering, and then deletes expired raw jobs.
  - **Anonymous totals:** Workspace usage records with no job or
    organization identifier.
- **Background:** `docs/protocol/app-removal.md` requires deletion of
  organization data 30 days after owner removal. When the last
  organization leaves, workspace data has the same deadline. The store
  saves these deadlines and exposes transactional purge queries.
  Metering must refresh before raw data deletion to preserve anonymous
  totals, as `crates/ciproxy-metering/README.md` requires.
- **Problem:** No production caller invokes
  `OrganizationStore::purge_removed_organizations` or
  `WorkspaceStore::purge_expired_workspace`. Repository searches find
  only their trait declarations, PostgreSQL implementations, and tests.
  The accounting worker calls only `purge_github_jobs` after a successful
  metering refresh. Boat registration metadata cleanup does not delete
  the organization or workspace itself.
- **Impact if no action:** Removed organization and workspace data stay
  past their declared deadlines. The retention behavior exists only in
  direct store tests.
- **Root question:** How should production delete expired organization
  and workspace data after accounting preserves its totals?
- **Options:**
  - **A.** Extend the accounting worker's retention sequence. Discover
    retained workspaces as well as active ones. After a successful,
    current metering refresh, run organization purge and then empty
    workspace purge. Keep independent workspaces running after a failure.
    Add a real-PgStore regression through `Accounting::pass` that advances
    the trusted clock past both deadlines. Assert exact deletion and
    preservation of sibling data and anonymous totals. Effort: Medium.
  - **B.** Add both purge calls to the dashboard API's periodic
    maintenance. Refresh metering before each purge and serialize that
    sequence with accounting work. Add a real-store maintenance
    regression for both deadlines. This adds a second owner for the
    accounting-before-deletion order. Effort: Medium.
  - **C.** Add a separate supervised retention worker. Give it complete
    retained-workspace discovery and a metering-refresh barrier before
    either purge. Test its public pass with the real store. This adds
    another task, schedule, and shutdown path. Effort: Medium.
- **Left out:** A store-only regression already covers the queries but
  cannot find an absent runtime caller. A database timer cannot perform
  the required metering refresh. A manual SQL deletion has no reliable
  schedule or accounting barrier.
- **Wider fix:** A. One production retention sequence keeps raw job,
  organization, and workspace deletion behind the same accounting rule.
- **Previous examples:** The workspace purge has the same missing-caller
  problem as the organization purge. The existing raw-job purge in
  `AccountingWorker::workspace` shows the required refresh-before-delete
  order.
- **Recommendation:** A. It reuses the existing supervised accounting
  boundary and keeps retention ordering in one place.
- **Action:** fix after merge, apply A, because the missing scheduling is
  outside the selected follow-up fixes and already exists on the target
  branch.
