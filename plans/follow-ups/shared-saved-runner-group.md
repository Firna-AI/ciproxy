# Two environments can share a saved runner group

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), review finding 1.
- **Location:** `crates/ciproxy-github/src/app/reconcile.rs:77`,
  `crates/ciproxy-github/src/rest/groups.rs:46`,
  `crates/ciproxy-github/scale-sets.md`, section "Runner groups".
- **Reason:** The check that settles the item needs read access to the
  deployed databases of main preview and production, which only the
  operator has.

- **Terms:**
  - **Environment:** one deployed control plane with its own GitHub App,
    such as main preview or production. `deploy/README.md`, section
    "Environments", lists them.
  - **Managed runner group:** the private GitHub runner group that an
    environment keeps in each organization. Its name is
    `ciproxy-<app id>-<organization id>`.
  - **Saved group ID:** the GitHub ID of that group, stored as
    `runner_group_id` with the scale sets of an organization, also in
    retained registrations.
- **Background:**
  - Until pull request #142, every environment named its group
    `ciproxy-<organization id>`. With #142, the name holds the GitHub App
    ID of the environment.
  - `AppService::prepare` in `crates/ciproxy-github/src/app/reconcile.rs`
    takes the saved group ID first. `RestClient::sync_group` in
    `crates/ciproxy-github/src/rest/groups.rs` reads that group under any
    name. It then sets the managed name and the repository list, and the
    group keeps its scale sets. Only when no saved ID finds a group does
    the sync look up the full managed name, or create the group.
  - A read-only check on 23 November 2026 found no saved group in the
    production database. Production deploys only from protected `main`,
    after a successful main preview of the same commit.
    `plans/follow-ups/production-launch.md` holds its launch.
- **Problem:** Code without #142 makes two environments that sync one
  organization find the same group by the shared old name, and both save
  its ID. After the upgrade, each environment still reads its saved ID
  first. Each sync then renames the shared group to its own managed name
  and writes its own repository list.
- **Impact if no action:** If a shared group exists, the two
  environments overwrite each other's group name and repository list
  every 300 seconds, and both keep using the same scale sets. Jobs can
  lose access to runners.
- **Root question:** How does each environment end the upgrade alone on
  its own runner group, also when two environments shared one group
  before it?
- **Options:**
  - **A.** Before production links an organization that main preview
    manages, confirm that production runs #142, or run one read-only
    query of each database for the saved group IDs of that organization.
    When no ID is shared, nothing more is needed. When one is shared,
    give the group to one environment, let the other environment's jobs
    finish, and connect it to a new group. Needs database access. Effort:
    Medium.
  - **B.** Give both environments new groups when they share an old-name
    group, and move them after the old jobs finish. Effort: Large.
  - **C.** Record one owner App for each remote group, and read that
    record before every update. This needs a store that both environments
    share. Effort: Large.
  - **D.** Use the managed name as the owner marker. Before an
    environment changes the group of its saved ID, it reads the group
    name. When the name is the managed name of another App for the same
    organization, it leaves that group and its scale sets alone, finds or
    creates its own group, and creates fresh scale sets in it. This
    changes the guide rule that the saved ID finds the group under any
    name, and needs a path for saved scale sets in another group. Effort:
    Large.
  - **E.** Keep the code. Rely on the check of 23 November 2026, and link
    no organization in production until production runs #142. The guide
    already says that two environments that saved one group ID share it
    until an operator splits it. Effort: Small.
- **Left out:** A saved-ID path that accepts only the current or the old
  managed name breaks the rule that a group keeps its ID after a rename by
  an organization owner. The test
  `owned_group_survives_rename_and_sdk_recovers_204_absence_with_typed_missing_signals`
  in `crates/ciproxy-github/tests/integration/recovery_http_tests.rs`
  protects that rule. A deploy check that compares saved group IDs across
  databases adds a gate.
- **Wider fix:** D. It prevents a shared saved ID wherever it comes from.
  C needs new shared infrastructure.
- **Previous examples:** Commit `fa3e1e6` introduced reuse by saved group
  ID.
- **Related:** `plans/follow-ups/production-launch.md` is the step where
  production first links an organization. Option A belongs before it.
- **Recommendation:** A. Only the one-time move off the old name can make
  a shared group, and the check of 23 November 2026 found none. D best
  protects the codebase.
- **Action:** ask the user, because the check of option A needs access to
  the deployed databases.
