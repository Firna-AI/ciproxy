# Retained Add reinstalls skip runner preparation

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from PR #32, owner review round 1, finding 2.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-github/src/app/installation.rs`, the `held`
  value of `install`,
  `crates/ciproxy-api/src/access/install_authority.rs`, the values
  `approved` and `billing_hold` of `installation_decision`,
  `crates/ciproxy-store-pg/src/installation_links/activation.rs:104`, the
  insert of `register`,
  `crates/ciproxy-store-pg/src/github/lifecycle/rows.rs:52`, the functions
  `resources` and `event`, and
  `crates/ciproxy-github/src/app/lifecycle.rs:80`, the target of
  `synchronize`.
- **Reason:** This unusual retained-reinstall case repairs itself on the next
  repository refresh.

- **Terms:**
  - **Retained reinstall:** a new install within 30 days of a removal, which
    returns the organization to the workspace that it left.
  - **Add:** an install that an Admin starts with "Add an organization",
    unlike a Direct install, which no Admin started that way.
  - **Admission:** the row of the `organization_activation` table that
    permits an organization to run jobs, beside its billing hold and its
    Admin approval hold.
  - **Preparation:** `AppService::prepare` in
    `crates/ciproxy-github/src/app/reconcile.rs`, which updates the runner
    group, releases deleted label names, and ensures a scale set per label.
  - **Binding:** a row of the `scale_sets` table that ties an organization
    and a label to the remote ID of its GitHub scale set.
- **Background:**
  - A workspace is the account of a customer. Its labels serve the jobs of
    all its organizations, and ciproxy keeps one binding for each
    organization and label.
  - A retained reinstall keeps the admission of the organization. An
    organization that was active comes back active. One that was inactive
    comes back inactive, with the same holds.
  - `docs/protocol/app-removal.md`, in "Removing the App", and
    `docs/protocol/workspaces.md`, in "Inactive organizations", state these
    rules. An install through Add into the workspace that the organization
    left "does the same" as a normal retained reinstall. An active
    organization never turns inactive.
  - A retained reinstall must also prepare the current workspace labels
    before it resumes execution.
  - `install` in `installation.rs` handles the signed install event. It
    reads the decision of `installation_decision` and the retained admission
    row. It sets `held` when the row or the decision has a hold.
  - For an Add, `installation_decision` computes `billing_hold` as "Free and
    more than one organization". Its count adds one for this organization
    while it is not active. It does not read the retained admission row.
  - For an Add claim with no organization yet, `admin_approved` is true only
    when the person who started the Add is an Admin of the workspace now.
  - With `held` and an active App, `install` sets the target Inactive, so
    preparation does not run.
  - Registration then calls `register` in `activation.rs`. Its insert ends
    with `ON CONFLICT(organization_id) DO NOTHING`, so the retained row
    keeps `admitted = TRUE`. `refresh_billing` changes only rows that are
    not admitted.
  - `resources` in `rows.rs` saves bindings only for an Active target.
    `event` then publishes `LifecycleEvent::Active`, because the row is
    admitted.
  - A Direct retained reinstall, and `synchronize` in `lifecycle.rs`, read
    only the saved holds. For the same organization, both prepare before
    they publish.
  - Label create and delete act only on active organizations. So a label
    created while the organization was removed has no binding for it.
  - The control plane runs `RefreshRepositories::refresh` of
    `crates/ciproxy-control-plane/src/repositories.rs` every five minutes.
    It calls `synchronize` for each organization that is not removed, so it
    saves the missing bindings.
  - The retained-reinstall case at the end of this test in
    `crates/ciproxy-api/tests/http/link_hold_tests.rs` starts no new Add
    attempt:
    `free_admission_waits_for_upgrade_or_room_and_downgrade_keeps_admitted_organizations`.
    It exercises Direct mode and checks only that the organization is
    runnable.
  - `label_reactivation_tests.rs` also reinstalls in Direct mode. The unit
    test in `crates/ciproxy-github/src/_tests_/reinstall_tests.rs` uses a
    decision with `billing_hold: false`. `link_role_tests.rs` covers the
    retained holds of an organization that was not admitted.
  - No test catches skipped preparation for an admitted organization that
    comes back through Add.
  - The change that added this item also fixed a separate defect. A Direct
    install whose first event failed after the save of its claim could not
    finish when GitHub delivered the event again. That fix gave Direct and
    Add installs distinct recovery states, and it does not cover this item.
- **Problem:** Reinstall an admitted organization through Add into its
  retained Free workspace while an active sibling remains. The organization
  count sets `billing_hold`, so installation skips preparation, but
  registration preserves `admitted = true` and publishes the organization as
  active. An Add by a person who is no longer an Admin gives
  `admin_approved: false` and the same split.
- **Impact if no action:** A label added during removal has no binding after
  reinstall, so its jobs wait for the repository refresh, which runs every
  five minutes. Until then, the names of labels deleted during the removal
  stay unreleased, and the runner group keeps its old repository list.
- **Root question:** How should retained admission control reinstall
  preparation?
- **Options:**
  - **A.** Preserve admitted authority when calculating reinstall holds.
    Prepare its current resources before completing registration. Add an
    Add-based retained-reinstall regression. Preparation and registration
    still apply admission in separate code. Effort: Small.
  - **B.** Use one effective admission policy for preparation and
    registration. Distinguish retained admission from a new organization's
    initial approval. It changes the decision that `install` and
    `synchronize` act on. Effort: Medium.
  - **C.** Track preparation for each installation separately. Permit
    execution only after that installation's resources are ready. It adds a
    stored readiness state that execution must wait for. Effort: Medium.
  - **D.** Prepare the runner resources of every install with an active
    App, also while a hold applies. Admission alone then decides whether a
    runner starts. It creates GitHub scale sets for organizations that may
    never run, and leaves remote resources to clean up when a held
    organization leaves. It changes the behaviour that `link_hold_tests.rs`
    asserts for a held new organization, so it needs the owner. Effort:
    Medium.
- **Left out:**
  - Relying on periodic repair leaves a successful reinstall ahead of usable
    runner resources.
  - A refresh at once after registration fails the goal, because
    registration already published the organization as active before its
    bindings exist.
  - Treating an Add reinstall as a new organization, with its holds applied
    again, breaks the rules that an Add into the retained workspace does the
    same and that an active organization stays active.
  - A store guard that rejects an Inactive target for an admitted row adds a
    new rejection, which needs the user. B removes the case without it.
  - Fix A inside `installation_decision` reads the retained row outside the
    workspace lock of the registration, while `install` already reads it.
- **Wider fix:** B. It prevents preparation and execution authority from
  using different admission rules.
- **Previous examples:** Commit `a337aaf`, "fix(github): preserve activation
  after label replacement", repaired retained activation after label
  replacement. The [binding guide](../../crates/ciproxy-github/scale-sets.md)
  describes the shared reconciliation service for that area.
- **Related:** `crates/ciproxy-github/src/app/bindings.rs` owns binding
  creation and retirement for label changes, resume, and reinstall. The
  admission policy of B can use that service. This item still changes the
  effective admission policy that `install` and `synchronize` apply.
- **Recommendation:** B. One admission policy best protects the codebase.
- **Action:** fix after merge, apply B, because this unusual
  retained-reinstall case repairs itself on the next repository refresh.
  - Apply one effective admission policy in `install` and in `synchronize`,
    for the billing hold and for the Admin approval hold.
  - A retained organization that was admitted comes back active. Its
    preparation saves its current bindings before registration publishes
    it.
  - A retained organization that was not admitted comes back inactive with
    the same holds. A new install ends no hold.
  - A new organization with no retained row keeps the holds of its initial
    approval from the Add decision.
  - Build on the Direct and Add recovery states of the change that added
    this item. Do not copy them.
  - Add a regression to the HTTP tests in `crates/ciproxy-api/tests/http/`
    with the real store. Start a fresh Add into the retained Free workspace
    while an active Free sibling remains. Create a label while the
    organization is removed.
  - Expect a binding for that label before the organization is published
    active, with no repository refresh. Add the same case for a person who
    started the Add and is no longer an Admin.
  - Keep the cases that pass now: a held new organization gets no scale
    sets, and a retained organization that was not admitted keeps both
    holds.
  - The fix needs no change in `crates/ciproxy-control-plane`.
