# Schema changes break the old control plane

- **Severity:** High. **Likelihood:** Common. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, review finding 2.
- **Location:** `deploy/helm/ciproxy/templates/control-plane.yaml:11`,
  `crates/ciproxy-store-pg/src/factory.rs:27`, and the migrations
  `202611231528_overflow_provider_records`,
  `202611231601_overflow_provider_settings_and_label_order`, and
  `202611231835_metering_by_provider` under
  `crates/ciproxy-schema/migrations/`.
- **Reason:** Each option trades a planned outage, a new gate, or a new
  rejection, and only the user chooses that.

- **Source:** Review.
- **Category:** Product bug.
- **Terms:**
  - **Rolling upgrade:** Kubernetes starts the new control-plane pod while
    the old pod still serves.
- **Background:**
  - The control-plane Deployment uses `RollingUpdate` with
    `maxUnavailable: 0`, so the old pod serves until the new pod is ready.
  - The new pod runs the database migrations at startup. Both pods use the
    same database.
  - The provider-interface change renames the Boat tables, drops the label
    switch columns and the workspace spend-limit column, and changes the
    key of the daily metering rows.
  - Earlier commits `15805cb0` (dropped `machines.kvm_available`) and
    `1f6f8410` (dropped `metering_periods`) met the same overlap.
  - An image rollback selects an older digest with no schema step, as
    `deploy/control-plane-images.md:172` shows, so the old release then
    runs on a schema it cannot read.
  - Today only the main preview runs a control plane, and ciproxy is not
    in production.
- **Problem:** During the overlap, the old pod queries tables and columns
  that the new pod's migrations renamed or dropped.
- **Impact if no action:** A deploy can fail dashboard requests and stop
  runner reconciliation until the old pod ends. An image rollback across
  such a migration leaves the control plane broken.
- **Root question:** How does each deploy and each rollback avoid running a
  control-plane release on a schema that it cannot use?
- **Options:**
  - **A.** One coordinated cutover for this release: stop the old control
    plane before the migration, and start the new one. Document the
    database recovery step. The main preview deploys on each push to
    `main`, so this needs `CONTROL_PLANE_RUNTIME_READY=false` before the
    merge, or `Recreate` for this release only. Effort: Medium.
  - **B.** Stage the change: additive schema and backfill first, and the
    renames, drops, and label-order activation in a later release.
    Effort: Large.
  - **C.** Separate databases for the two versions, and switch traffic after
    a verified data transfer. Effort: Large.
  - **D.** Set the control-plane Deployment to `strategy: Recreate`, so
    Kubernetes stops the old pod before the new pod migrates, for every
    deploy. Update `deploy/runtime.md` and add a rollback paragraph: run
    the down migrations first, or roll forward. Cost: a short outage at
    each deploy, and no control plane if the new pod cannot start.
    Effort: Small.
  - **E.** Mark each migration that the previous release cannot use. The
    deploy script scales the control plane to zero before such a release.
    A new gate check fails an unmarked drop or rename. Effort: Medium.
  - **F.** A schema fence: a control plane that finds a newer migration
    than it knows reports not ready and pauses. This still needs A or D
    for this deploy. Effort: Medium.
- **Left out:** Waiting for the new pod's readiness, because the old pod
  still serves on the changed schema before that point. Compatibility
  views alone, because the daily metering key change still needs B. A
  migration Job before the rollout alone, because the old pod still serves
  after it.
- **Wider fix:** D. It keeps every later migration, whatever its shape,
  from meeting an old release. F keeps rolling updates for releases with
  no migration. E is a detection fix.
- **Previous examples:** `15805cb0` and `1f6f8410` on `main`.
- **Recommendation:** D. It does the cutover of this release with no manual
  step, it protects each later migration, and an outage at each deploy
  costs little while only the preview runs. Production can replace D with
  F before the launch.
- **Action:** ask the user, because each option trades a planned outage, a
  new gate, or a new rejection, and D changes the rollout behaviour on
  `main`.
