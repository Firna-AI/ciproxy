# Scheduler supply loses immutable registration identity

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-07, from PR #35 on
  `conductor/mvp-m15-overflow`, review finding 1, wider fix C.
- **Location:** `crates/ciproxy-scaler/src/snapshot.rs:43`,
  `crates/ciproxy-scaler/src/lifecycle.rs:82`,
  `crates/ciproxy-control-plane/src/snapshot.rs:147`,
  `crates/ciproxy-control-plane/src/boat_snapshot.rs:42`,
  `crates/ciproxy-control-plane/src/boat_authority.rs:82`,
  `crates/ciproxy-github/src/registrations.rs:55`,
  `crates/ciproxy-github/src/scale_sets/mod.rs:18`, and
  `crates/ciproxy-store-pg/src/github/scale_sets.rs:55`.
- **Reason:** The shared identity change reaches scaler, GitHub registration,
  and own-machine code outside the Boat milestone's allowed files.

- **Terms:**
  - **Label:** A workflow name in `runs-on` that selects a job's VM size.
  - **Scale set:** The GitHub resource that assigns jobs to runners for one
    organization and label.
  - **Binding:** A stored row that connects an organization and label to a
    remote scale-set ID.
  - **Boat:** The provider that runs overflow jobs in rented sandboxes.
  - **Registration target:** The exact label, binding row, and remote
    scale-set ID used when ciproxy registers a runner.
- **Background:**
  - ciproxy runs GitHub Actions jobs on customer machines first. It uses
    Boat when those machines are busy.
  - The scaler counts available runners before it starts new runners.
    Its labels, waiting jobs, and runners retain names instead of exact
    registration targets in `crates/ciproxy-scaler/src/snapshot.rs`.
  - Deleting a label retains its old binding while jobs or a listener need
    it. Creating the same name gives the replacement new label and binding
    IDs. Old runners cannot serve the replacement scale set.
  - `docs/protocol/labels.md` requires running jobs of a deleted label to
    finish. `docs/protocol/overflow.md` requires deletion of unneeded idle
    sandboxes. Each sandbox runs one runner that takes one job.
  - Direct fix A, committed in `2cac3ee`, excludes obsolete idle Boat
    runners through current label IDs in `boat_snapshot.rs`.
    `boat_authority.rs` uses that mapping for retirement. Running jobs stay
    protected. A real-store regression covers same-name replacement.
    Before A, the old sandbox could bill until exit or its lifetime limit.
  - Own-machine mapping in `snapshot.rs` still copies the saved label name.
    The scaler's `claim` matches only organization and label name.
    This path has the same replacement mismatch.
  - A binding's internal ID does not prove its remote scale-set ID.
    `save_on` in `crates/ciproxy-store-pg/src/github/scale_sets.rs` retains
    the row when it updates `github_id`.
  - `ensure` in `crates/ciproxy-github/src/scale_sets/sdk.rs` can create a
    replacement when the saved remote scale set is missing. Resume,
    reinstall, and repository refresh can call this path.
  - `RegistrationService::create` uses the remote ID from its own context
    read. `JitRegistration` returns the runner ID and configuration only.
    A later binding read cannot prove the registration's remote target.
  - Saved Boat runners and own-machine runner rows retain internal IDs,
    but no registration-time remote scale-set ID. Own-machine storage uses
    typed columns, so saving that remote ID needs a migration.
  - The code permits the remote identity gap. Nobody has reproduced a live
    runner surviving the loss of its remote scale set. Keep this case
    separate from the confirmed same-name replacement mismatch.
- **Problem:** Name-based matching can count an old idle runner against
  replacement demand that GitHub cannot give it. The Boat label-ID check
  fixes the reported replacement case, but own-machine matching still
  loses that identity. Neither path retains the exact remote registration
  target through the shared scaler.
- **Impact if no action:** Replacement jobs can wait for capacity that
  cannot serve them. An obsolete idle runner can escape retirement.
- **Root question:** How can ciproxy count an idle runner only for waiting
  jobs that GitHub can give it?
- **Options:**
  - **A.** Check current label and binding identities in the shared Boat
    mapping. Retire obsolete idle supply and preserve running jobs.
    Commit `2cac3ee` applies this direct fix. It leaves own-machine mapping
    and remote target capture unchanged. Effort: Small.
  - **B.** Retire obsolete idle Boat runners during label deletion before
    preparing a replacement. Provider failures then affect label changes.
    It does not correct own-machine matching or lost remote identity.
    Effort: Medium.
  - **C.** Carry immutable registration targets through scaler labels,
    waiting jobs, runner supply, and their callers. Update Boat and
    own-machine mappings together. Capture the binding row and exact
    remote scale-set ID in the registration result and save them with each
    runner. Include the organization and label identities. Migrate the
    own-machine storage and handle older records without inventing their
    registration target.

    Preserve running work and retire obsolete idle
    supply. Add real-store regressions for both runner types and binding
    replacement. The migration and rollout can increase the effort to
    Large. Effort: Medium.
  - **D.** Refuse same-name label creation until old idle runners end.
    This adds a customer restriction and needs user approval.
    It does not cover replacement of a remote scale set under one binding.
    Effort: Medium.
  - **E.** Extend A with a saved remote scale-set ID for Boat runners.
    Return the exact ID used by `RegistrationService::create` through
    `JitRegistration`. A separate read before registration can differ.
    This leaves own-machine and shared scaler matching unchanged.
    Effort: Medium.
- **Left out:** Changing nothing or changing a displayed name cannot make
  an old runner serve replacement demand. Registering another runner in
  the old sandbox breaks the one-runner rule. A startup-only check cannot
  correct an idle runner that already launched.
- **Wider fix:** C. Exact registration targets remove name equality as
  proof that a runner can serve a waiting job on either destination.
- **Previous examples:** Commit `c98e5a2` retains exact retired bindings
  during label replacement. Commit `a337aaf` repairs another replacement
  identity failure. The own-machine mapping in
  `crates/ciproxy-control-plane/src/snapshot.rs` has the same cause.
- **Related:** `plans/follow-ups/label-binding-reconciliation.md` shares
  the label replacement trigger. It unifies binding creation and retirement
  across label changes, resume, and reinstall. It does not change runner
  supply matching or capture registration-time remote identity.
- **Recommendation:** C. It protects Boat and own-machine scheduling from
  the same identity loss. A already protects the reported Boat case.
- **Action:** fix after merge, apply C, because shared scaler, GitHub
  registration, and own-machine changes exceed the Boat milestone's scope.
