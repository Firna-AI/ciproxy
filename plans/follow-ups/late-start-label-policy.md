# Late runner starts ignore label changes

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 1001.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-store-pg/src/own_results.rs`,
  `apply_own_runner_result`, its `Registered` and `Prepared` arms,
  `crates/ciproxy-store-pg/src/own_result_guard.rs`, `load`,
  `crates/ciproxy-store-pg/src/own_reservation.rs`, `require_authority`,
  `crates/ciproxy-store-interface/src/runtime_results.rs`,
  `OwnRunnerResult`, `crates/ciproxy-control-plane/src/own_start.rs`,
  `execute_own_start`, `crates/ciproxy-control-plane/src/preparation.rs`,
  `prepare_job`, and `crates/ciproxy-store-pg/src/overflow/authority.rs`,
  `ACCOUNT`.
- **Reason:** The race needs a label save inside the registration wait of
  one runner. Then at most one job runs under the old policy, as an idle
  old-size runner already can until the next pass.

- **Terms:**
  - **JIT configuration:** a one-time secret configuration from GitHub that
    registers one runner. ciproxy asks GitHub for it before it starts the
    runner.
  - **Reservation:** the saved row of a new own-machine runner in the state
    `Starting`. `require_authority` checks it when the row commits.
  - **Result guard:** the saved state that allowed a remote request.
    `own_result_guard::load` reads and locks it again before a result
    saves.
  - **Binding:** the scale set that connects a label to one organization on
    GitHub. A removed binding has `remote_removed_at` set.
  - **`Prepared` result:** `OwnRunnerResult::Prepared`. It lets the runtime
    send a start or a job preparation to the machine.
- **Background:**
  - A label sets the size of a job VM and its destinations: own machines,
    Boat, or both.
  - "Custom labels" in `docs/protocol/labels.md` says: "After the customer
    changes the size of a label, each job of the label that a runner takes
    gets the new size." It also says that a job that runs during the
    change keeps its VM.
  - The scaler plans a new own-machine runner, and the runtime saves a
    reservation. `require_authority` checks under row locks that the
    machine is not removed or draining, and that the organization is
    active and admitted.
  - It also checks that the label is not deleted, allows own machines, and
    has the requested size. It takes `FOR UPDATE` on the workspace, the
    machine, the organization, the activation, and the label rows.
  - Then `execute_own_start` asks GitHub for a JIT configuration. This
    network wait can take seconds.
  - After the wait, the `Registered` result saves the GitHub runner ID.
    `apply_own_runner_result` accepts it only when the guard still matches,
    the organization is admitted and active, and the machine is not
    draining. It reads no label and no binding.
  - Then the runtime applies `Prepared` and sends `StartRunner` to the
    machine. `Prepared` only compares the guard.
  - The guard holds the runner, the organization, the machine, its session,
    the activation, and the jobs of the runner. It holds no label and no
    binding.
  - The scaler stops an idle runner whose label changed at its next pass,
    every 10 seconds. `obsolete` in `crates/ciproxy-scaler/src/lifecycle.rs`
    returns `LabelChanged`, `DestinationDisabled`, or
    `OrganizationRemoved`. It skips a runner that has a job.
  - `dispatch_retirement` in
    `crates/ciproxy-control-plane/src/remote_retirement.rs` returns at once
    while a create task of the runner runs.
  - The Boat path already checks the label switch, the binding, and the
    exact requested size at each start stage. The `ACCOUNT` query of
    `crates/ciproxy-store-pg/src/overflow/authority.rs` holds these checks.
  - `OwnRunnerResult::Prepared` serves two steps. `execute_own_start` uses
    it before the start delivery. `prepare_job` uses it before it sends
    the facts of a job that already started.
  - `route_job` in `crates/ciproxy-control-plane/src/events.rs` reaches
    `prepare_job` only for a job in the phase `Started` or later. So a
    label check on `Prepared` would also hold back the facts of a running
    job.
  - `crates/ciproxy-control-plane/runners.md` says that a late GitHub
    registration keeps its actual identity for GitHub-first cleanup by its
    unique name, and starts no runner.
- **Problem:** An Admin can save a label while a JIT request waits. Neither
  the `Registered` result nor the `Prepared` check reads the current label
  or binding. So a runner that was reserved before the save can start with
  the old size, or on own machines after the Admin turned them off.
- **Impact if no action:** The late runner can take one queued job before
  the next pass stops it. That job runs with the old size, or on a
  destination that the customer disabled. A rerun of the job is the
  workaround.
- **Root question:** How does a runner whose registration wait spans a
  label save start only under the current label rules, while running jobs
  keep their VMs and facts?
- **Options:** Every option must give the start delivery its own result,
  apart from the `Prepared` result of job preparation. Otherwise the label
  rule holds back the facts of a running job.
  - **A.** Check the current label eligibility before the `Registered`
    result saves and before the start delivery. Add a PostgreSQL
    regression case for a label change during the wait. Effort: Small.
  - **B.** Share one startup eligibility rule between the reservation and
    the final result. Include the current label size, the destination
    switch, and the binding. Keep the GitHub identity of a refused late
    result, so that cleanup can remove it by name. Effort: Medium.
  - **C.** Give the label policy a stored generation. Save that generation
    in each reservation, and refuse a start when it changed. Cost: a
    migration and changes to the label writers. C uses one generation for
    all fields. So it also refuses starts after an edit that does not affect
    own machines, such as a change of the Boat switch. Effort: Medium.
  - **D.** Gate the start with the retirement rule of the scaler.
    - After the registration wait, read a fresh routing snapshot. Run the
      pure rule behind `retire_obsolete` in
      `crates/ciproxy-scaler/src/lifecycle.rs` for this runner.
    - When the rule returns `LabelChanged`, `DestinationDisabled`, or
      `OrganizationRemoved`, refuse the start and run the late-registration
      cleanup.
    - A late start then fails whenever the next pass would stop the runner
      for a policy reason. A new stop reason needs no second copy.
    - The check runs outside the store transaction. This is safe, because
      it only gates a send. A label save just after the commit has the same
      effect as a save just after the delivery.
    - D drops the assumption of A, B, and C that the check runs inside the
      store result transaction, under row locks. The source of that
      assumption is the option text and the guard design.
    - Cost: one snapshot read and one scaler run for each start.
    - Effort: Medium.
  - **E.** Make one label-and-binding startup predicate for both
    destinations.
    - It checks four things. The label is not deleted. The binding is not
      removed. The destination switch is on. The requested CPU, memory, and
      disk equal the label.
    - Use it in the Boat authority, in `require_authority` of the
      own-machine reservation, and in the own-machine start gate. A new
      label rule then reaches every start path of both destinations.
    - E drops the split into one Boat rule and one own-machine rule. The
      source of that split is the code: the Boat authority already checks
      each start stage, since commit `f3d4b149`.
    - Cost: the Boat check is one raw SQL query over the JSON delivery
      record. The shared part needs a SQL fragment that takes two input
      forms, and every Boat label regression must pass again.
    - Effort: Medium.
- **Left out:**
  - A check of only the returned registration target: it fails the goal,
    because it does not see a size or destination change.
  - A withdrawal of pending reservations in the label writers,
    `update_label` and `retire_dashboard_label`: B is better on every
    point. No saved marker tells an undelivered reservation from a VM that
    already boots, so it needs a new marker. Each policy writer would also
    need its own copy of the rule.
  - A reconciliation pass on label save: it fails the goal, because no pass
    has to run inside the registration wait.
  - A saved withdrawal when the scaler stops a runner whose create task
    still runs: it fails the goal for the same reason.
  - The agent or GitHub as the place of the check: not a real choice. The
    open-source agent does not know the hosted label policy. GitHub gives
    a queued job to any idle runner of the scale set.
- **Wider fix:** B. It keeps the reservation and the start on one rule. E
  also joins the Boat rule, but it needs a restructure of the raw Boat SQL.
- **Previous examples:**
  - Commit `1c486f90` of 4 November 2026 fixed idle Boat fallback runners
    that ignored the current size policy. It shared the Boat size rule
    between new starts and idle runners.
  - Commit `f3d4b149` of 5 November 2026 made the Boat delivery check the
    label switch, the binding, and the exact requested size at each start
    stage. The user approved that shared Boat authority for this class.
- **Related:**
  - `plans/follow-ups/result-guard-lock-rule.md`: the same code, and
    options that conflict. Both items change the row locks of
    `own_result_guard::load`.
    - B must not call `require_authority` as it is. That function takes
      `FOR UPDATE` on the workspace and organization rows.
    - In the result transaction, those locks bring back the foreign-key
      lock cycle that commit `5db2e701` removed at the workspace row. The
      other item removes the same cycle at the organization row.
    - So B shares only the predicate, under the lock modes of the other
      item. Apply that item first, or in the same change.
  - `plans/follow-ups/vm-size-conversion.md`: the same code, under D only.
    D reads the scaler sizes through `snapshot::size` of
    `crates/ciproxy-control-plane/src/snapshot.rs`, which that item moves
    to one shared conversion. The options do not conflict.
  - `plans/follow-ups/label-size-change-window.md`: the same rule of
    `docs/protocol/labels.md`. That item asks whether the doc must state
    the 10-second window of idle old-size runners. Neither item waits for
    the other.
- **Recommendation:** B, the same as the review, with two conditions:
  - Give the start delivery its own result type. Job preparation keeps
    `Prepared` with no label rule.
  - Share the predicate of `require_authority`, with the binding check
    that the Boat authority has. Do not share its `FOR UPDATE` statements.

  B replaces the partial copy in the `Registered` arm with the reservation
  rule. It matches the shared Boat authority of `f3d4b149`, and it needs no
  schema change. C needs a migration and refuses starts after unrelated
  edits. D adds a snapshot read to each start.
- **Action:** fix after merge, apply B with its two conditions, because it
  is Medium and Rare. At most one job runs under the old policy, and a
  rerun of that job is the workaround.
