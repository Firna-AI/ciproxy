# Boat registration records outlive lost access

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-07, from PR #35, the independent check of re-review
  finding 6.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-control-plane/src/boat_retirement.rs:47`,
  `crates/ciproxy-control-plane/src/boat_runtime.rs:54`,
  `crates/ciproxy-store-pg/src/organization_lifecycle.rs:68`, and
  `crates/ciproxy-schema/migrations/202611070000_boat_runner_intents/up.sql:5`.
- **Reason:** Complete metadata cleanup reaches organization retention and
  completed take-out bookkeeping beyond the current retirement correction.

- **Terms:**
  - **Boat:** The provider that runs overflow jobs in rented sandboxes.
  - **Registration intent:** A saved Boat runner identity, written before
    ciproxy asks GitHub for its one-use runner registration.
  - **Delivery:** The durable record of one attempted Boat sandbox start.
  - **Take-out:** An Admin action that removes an organization from its
    workspace and uninstalls the GitHub App.
  - **Workspace:** The customer account that owns settings, machines, and
    the Boat account.
- **Background:**
  - Owner App removal ends GitHub access. Running Boat work can finish.
    Launched idle sandboxes can stay until runner exit or their lifetime.
  - `docs/protocol/app-removal.md` requires deletion of organization data
    after 30 days. Other organizations can keep using the workspace.
  - Take-out first retires idle runners and preserves busy jobs, then
    uninstalls the App. `docs/protocol/workspaces.md` keeps its job history.
    Take-out sets no owner-removal timestamp or retention deadline.
  - Boat intents have a workspace foreign key, but no organization foreign
    key. Organization retention cleanup does not delete them.
  - Registration recovery reads intents with ended or absent deliveries.
    It normally removes an intent after positive GitHub retirement.
  - The narrow recovery correction defers known owner-removal work before
    a GitHub call. It preserves the intent for a new install.
  - A busy Boat registration can remain after accepted take-out. Its
    sandbox can later end through its exit report or provider lifetime.
    Its saved intent then has no GitHub access for ordinary retirement.
- **Problem:** Registration metadata has no complete end rule after its
  organization's access and retention end. Owner-removal intents can
  survive the 30-day organization purge. A completed take-out can leave
  ended registration work that repeatedly asks an inaccessible installation.
- **Impact if no action:** Unneeded records remain in routing snapshots.
  Repeated failed requests and warnings can hold the shared runtime lock.
- **Root question:** How should registration records end when their
  organization's access and permitted retention have ended?
- **Options:**
  - **A.** Add separate cleanup for organization purge and terminal Boat
    deliveries after completed take-out. Keep live identities and job
    history. Each path must maintain the same ownership rules.
    Effort: Medium.
  - **B.** Give registration recovery one terminal ownership rule shared
    with organization retention and completed take-out. Use saved
    installation authority and terminal delivery proof. Preserve live
    identities and job history. Stop inaccessible retries and remove
    expired metadata under the existing retention requirements.
    Effort: Medium.
  - **C.** Tie intent deletion to organization deletion through a store
    relationship, and handle terminal take-out registrations separately.
    The relationship cannot by itself stop retries before organization
    deletion or finish a take-out that keeps its history. Effort: Medium.
- **Left out:** Deleting every intent at App removal loses identities of
  live deliveries. Treating each removed App state as retirement proof
  bypasses take-out acceptance. Slower retries still request impossible
  work. A guest retirement mechanism does not restore GitHub access or
  remove expired registration metadata.
- **Wider fix:** B. One terminal ownership rule covers both removal paths
  without making a missing remote answer look like retirement proof.
- **Previous examples:** PR #35's take-out correction covers complete
  reservations before uninstall. Its owner-removal correction defers
  known inaccessible recovery. Neither completes metadata retention.
- **Related:** `plans/follow-ups/boat-guest-retirement.md` addresses earlier
  deletion of idle sandboxes after App removal. This item concerns saved
  registration records after terminal delivery or organization retention.
- **Recommendation:** B. Shared ownership and retention rules prevent the
  two cleanup paths from leaving different registration records behind.
- **Action:** fix after merge, apply B, because this storage lifecycle work
  exceeds the requested take-out and reinstall fix round.
