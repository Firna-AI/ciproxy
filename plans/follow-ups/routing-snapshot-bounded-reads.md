# Routing reads load the whole workspace history

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, two problems that the check
  of finding 902 found.
- **Source:** Review.
- **Category:** Performance.
- **Location:** `crates/ciproxy-store-pg/src/routing_snapshot.rs`,
  `snapshot`, `crates/ciproxy-store-pg/src/own_runners.rs`, `load`,
  `crates/ciproxy-store-pg/src/own_result_guard.rs`, `load`,
  `crates/ciproxy-store-pg/src/boat_results.rs`, `load`,
  `crates/ciproxy-control-plane/src/snapshot.rs`, `build`,
  `crates/ciproxy-control-plane/src/preparation.rs`, `prepare_job`, and
  `crates/ciproxy-control-plane/src/runtime_reconciliation.rs`,
  `reconcile_workspace_inner`.
- **Reason:** No production workspace holds such a history yet, and a
  bounded read changes what each routing pass sees.

- **Terms:**
  - **Routing snapshot:** `RoutingSnapshot`, the one read of a workspace
    that a routing pass uses: machines, reports, labels, jobs, runners, and
    Boat records.
  - **Result guard:** the saved state that allowed a remote request. Two
    loaders read and lock it before a result saves:
    `own_result_guard::load` for own machines, and `load` in
    `boat_results.rs` for Boat.
  - **Retained job:** a row of `github_jobs`. The store deletes a completed
    job 90 days after it ends.
  - **Saved runner:** a row of `runners`. Only the deletion of its
    organization or workspace removes it.
- **Background:**
  - `snapshot` in `routing_snapshot.rs` reads every retained job of the
    workspace and parses each JSON body. It reads every saved own runner
    with its command row, through `own_runners::load`, and every Boat
    runner row. It also parses every machine report.
  - The job purge of `crates/ciproxy-store-pg/src/github/jobs.rs` deletes a
    completed job 90 days after it ends. No code deletes a runner row.
  - Each ephemeral runner takes one job, so the runner rows grow with every
    start.
  - Each reconciliation pass, each runner event, and each job update loads
    one snapshot. The tick runs every 10 seconds.
  - `reconcile_workspace_inner` routes each job of the snapshot.
  - `build` in `crates/ciproxy-control-plane/src/snapshot.rs` keeps only
    the jobs of the last 24 hours as warm-runner history. It filters them
    in memory, after the full read.
  - The own-machine guard load reads and locks with `FOR UPDATE` every
    retained job of the scale set of the runner. Then it keeps only the
    jobs of that runner. It also parses every machine report of the
    workspace, and keeps one machine.
  - The Boat guard load repeats the same job query.
  - `prepare_job` runs the own-machine guard load up to twice for each
    running job on each pass. It reads the guard once, and the result write
    reads it again.
  - "Tiers" in `docs/protocol/billing.md` gives the Business tier 150,000
    starts a month and 90 days of job history. So one workspace can hold
    about 450,000 retained jobs, and its runner rows grow by about 150,000
    a month.
  - The control plane runs as one replica with a memory limit of 1 GiB. An
    out-of-memory kill stops routing for every workspace.
  - On 17 November 2026 the final review removed the copy of the whole
    snapshot from each preparation task. Each pass still loads one whole
    snapshot.
  - The metering tables refer to runner rows with `ON DELETE CASCADE`. So a
    deletion of runner rows also deletes their metering facts.
- **Problem:** Each routing read grows with the age and the volume of the
  workspace, not with its current work. Each guard load also locks every
  retained job of a scale set, and parses every machine report.
- **Impact if no action:** In a large workspace, each pass gets slower and
  larger, and it can push the one control-plane pod past its memory limit.
  The wide `FOR UPDATE` locks also make the listener writes of the same
  scale set wait longer.
- **Root question:** How do routing and result checks read only the rows
  that the current work needs?
- **Options:**
  - **A.** Narrow the guard loads only. Select the jobs of the runner in
    SQL, by its GitHub ID, and read one machine report, not all. Cost: the
    routing snapshot still grows with the history. Effort: Small.
  - **B.** Narrow the routing snapshot too, and keep the guard narrowing of
    A.
    - Read only these rows:
      - the open jobs
      - the jobs of the last 24 hours, for the warm-runner history
      - the runners that are not stopped
      - the current Boat runners
    - Check each consumer of the snapshot, such as the reconciliation loop,
      the stop recovery, and the Boat projection, against the narrower
      read.
    - Add a PostgreSQL test for each bound.
    - Cost: more consumers to check than in A.
    - Effort: Medium.
  - **C.** Keep the snapshot between passes, and apply only the changes.
    Cost: a cache with its own invalidation for every writer of jobs,
    runners, and reports. The first load still reads the whole history, and
    the cache keeps it in memory. Effort: Large.
- **Left out:**
  - A purge of stopped runner rows: it fails the goal alone, because the
    jobs of 90 days stay. It also deletes the metering facts that refer to
    those rows.
  - Paging the snapshot read: the scaler needs the whole relevant set at
    once. B removes the rows that it does not need instead.
- **Wider fix:** B. It bounds every routing read by the current work. A
  fixes only the guard loads.
- **Previous examples:**
  - `plans/follow-ups/job-history-paging.md` records the same class in the
    Jobs pages: each read builds every retained job before any filter.
  - On 17 November 2026 the final review found that each preparation task
    copied the whole snapshot, and fixed it in the branch.
- **Related:**
  - `plans/follow-ups/result-guard-lock-rule.md`: the same code. That item
    changes the lock modes of the guard loads, and A here changes which
    rows they read. The options do not conflict. Both can go in one change,
    in either order.
  - `plans/follow-ups/job-history-paging.md`: the same class in other code.
    Neither item waits for the other.
- **Recommendation:** B. It bounds each routing read by the current work,
  so the cost no longer grows with the age of the workspace. A is the small
  first step.
- **Action:** fix after merge, apply B, because no production workspace
  holds such a history yet, and B changes what each routing pass reads.
