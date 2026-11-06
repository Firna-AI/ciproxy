# Split binding reconciliation can block organization resume

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from PR #30, owner review round 2, finding 5.
- **Location:** `crates/ciproxy-api/src/handlers/label_cleanup.rs:16`,
  `crates/ciproxy-api/src/handlers/labels.rs:172`,
  `crates/ciproxy-api/src/maintenance.rs:46`,
  `crates/ciproxy-github/src/app/reconcile.rs:60`,
  `crates/ciproxy-github/src/app/lifecycle.rs:84`,
  `crates/ciproxy-github/src/app/installation.rs:150`,
  `crates/ciproxy-store-pg/src/scale_sets.rs:80`, and
  `crates/ciproxy-store-pg/src/scale_set_tracking.rs:14`.
- **Reason:** The shared service rewrites resume and reinstall code beyond
  the change that added this item, and fix A stops the failure without
  delaying control-plane integration.

- **Terms:**
  - **Organization:** a GitHub organization with the ciproxy App, in the
    App state active, suspended, or removed.
  - **Scale set:** the GitHub runner resource for one organization and one
    label, which GitHub can also find by the label name.
  - **Binding:** a row of the `scale_sets` table that ties an organization
    and a label to the remote ID of their scale set.
  - **Retired binding:** the binding of a deleted label, whose row and job
    references stay also after remote cleanup records completion.
  - **Completion tracking:** the GitHub message listener and the job
    records that deliver the results of the unfinished jobs of a retired
    binding.
- **Background:**
  - A workspace is the account of a customer. Its labels serve the jobs of
    all its organizations. ciproxy keeps one scale set and one binding for
    each organization and label.
  - A label has no rename. A delete and a new label with the same name
    give a safe rename, as
    [Custom labels](../../docs/protocol/labels.md#custom-labels) says.
  - After an unsuspend, the jobs of the organization start on ciproxy
    runners again, as
    [Suspending the App](../../docs/protocol/app-removal.md#suspending-the-app)
    says. A new install within 30 days of a removal joins the organization
    to its workspace again, as
    [Removing the App](../../docs/protocol/app-removal.md#removing-the-app)
    says.
  - The API path retires bindings in `cleanup` of `label_cleanup.rs`. A
    label delete, a same-name label create, and `Maintenance::reconcile`
    run it. It acts only on active organizations.
  - `cleanup` renames a busy retired scale set to `ciproxy-retired-` and
    the binding ID. It removes an idle one and records `remote_removed_at`.
  - A label create then checks the name with GitHub through `prepare` in
    `labels.rs`, and asks the GitHub path to refresh each active
    organization.
  - The GitHub path creates and saves bindings in `AppService::prepare` of
    `reconcile.rs`. Resume calls it through `synchronize` in
    `lifecycle.rs`, and reinstall through `install` in `installation.rs`.
  - Before direct fix A, `AppService::prepare` read bindings only through
    the store query at `scale_sets.rs:80`, which hides the bindings of
    deleted labels. For a label with no binding, `ensure` in
    `crates/ciproxy-github/src/scale_sets/sdk.rs` returns the scale set
    that already has the label name.
  - The `scale_sets` table has the unique constraint
    `(organization_id, github_id)`. Job rows refer to their binding row.
  - The `listeners` query of `scale_set_tracking.rs` keeps the listener of
    a retired binding while it has unfinished jobs or pending deliveries.
    Remote cleanup waits for them and for the session lease of the
    listener.
  - In the reproduced failure, one workspace had an active and a suspended
    organization. A label was deleted and created again with the same
    name.
  - `cleanup` skipped the suspended organization, so its old scale set
    kept the name. On resume, `AppService::prepare` gave the old remote ID
    to the replacement label.
  - The save then conflicted with the retained binding and failed with
    `[store] record already exists`. A real-store probe reproduced this,
    also after maintenance retries. The organization stayed suspended in
    ciproxy, and new jobs of all its labels stayed blocked.
  - The change that added this item applied direct fix A in
    `AppService::prepare`, which resume and reinstall share. It now reads
    the pending active and retired bindings through the narrow
    `InstallationStore` adapter in
    `crates/ciproxy-github/src/persistence/installation.rs`. It renames
    the remote scale sets of retired bindings before `ensure`.
  - The store query at `scale_sets.rs:80` still serves active routing
    without change.
  - Fix A skips bindings with confirmed removal, keeps every row and its
    history, and deletes nothing remotely. A missing remote scale set
    counts as a released name. Other GitHub errors stay errors.
  - A regression test covers the reproduced case. Fix A adds no new
    service, refusal, or constraint.
  - The repository owner chose this split: fix A at once, and B after the
    merge.
- **Problem:** Binding retirement runs in the API path, and binding
  creation runs in the GitHub path. Before fix A, the GitHub path lacked
  the retired name release, so resume failed. Both paths still apply the
  binding rules in separate code, so a rule that one path lacks can block
  resume again.
- **Impact if no action:** Each later change to the binding rules must
  edit both paths. A miss can again keep a resumed or reinstalled
  organization inactive, with new jobs of every label blocked.
- **Root question:** How can label replacement preserve an organization's
  ability to resume?
- **Options:**
  - **A.** Release retired names before preparing active bindings during
    resume and reinstall. Add the reproduced regression. The change that
    added this item applied A. Effort: Small.
  - **B.** Share binding creation and retirement through one GitHub
    reconciliation service for label changes, resume, and reinstall.
    Preserve completion tracking. Label create and delete, the maintenance
    retry, resume, and reinstall then call that service. It keeps each
    retired row, its job references, and its listener until cleanup
    records completion. Effort: Medium.
  - **C.** Refuse replacement creation until every organization can
    release the old binding. This adds a user-visible restriction and
    needs approval. Effort: Medium.
- **Left out:** Reusing or deleting the retained database row reassigns
  or discards historical references. Changing nothing would have left
  resume blocked. Stopping at fix A keeps the binding rules in two places.
- **Wider fix:** B. It makes every activation handle retained bindings
  before it uses their names.
- **Previous examples:** Commit `c98e5a2` added the rename that releases
  the name of a draining binding. Only the API cleanup path called it, not
  resume or reinstall.
- **Recommendation:** B. Shared reconciliation best protects existing
  label behavior.
- **Action:** fix after merge, apply B, because the shared service reaches
  beyond the change that added this item, and fix A already stops the
  failure.
