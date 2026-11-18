# Result guard locks can form deadlocks

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 1003, with the lock
  cycle of the first Boat placement that its check found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-store-pg/src/own_result_guard.rs`, `load`,
  `crates/ciproxy-store-pg/src/boat_results.rs`, `load`,
  `crates/ciproxy-store-pg/src/github/jobs.rs`, `save`,
  `crates/ciproxy-store-pg/src/github/attempts.rs`, `save`,
  `crates/ciproxy-store-pg/src/overflow/deliveries.rs`, `save`,
  `crates/ciproxy-store-pg/src/overflow/placements.rs`, `save`, and
  `crates/ciproxy-store-pg/execution.md`.
- **Reason:** Each cycle needs an unusual overlap of two transactions, and
  PostgreSQL ends each deadlock with an error that a later pass repeats.

- **Terms:**
  - **Result guard:** the saved state that allowed a remote request. Two
    loaders read and lock it before a result saves:
    `own_result_guard::load` for own machines, and `load` in
    `boat_results.rs` for Boat.
  - **Demand row:** the saved current state of one scale-set job, a row of
    `github_jobs`.
  - **Attempt row:** one assignment of that job, a row of
    `github_job_attempts`, saved apart from its demand row.
  - **Delivery row:** the saved record of one Boat start, a row of
    `overflow_starts`.
  - **Key-share lock:** the lock that PostgreSQL takes on a parent row when
    it checks the foreign key of a new child row. `FOR UPDATE` on the
    parent blocks it. `FOR NO KEY UPDATE` does not.
- **Background:**
  - A result guard makes the compare and the write of a remote result
    atomic. Each loader locks every row that it compares until the
    transaction commits. `crates/ciproxy-store-pg/execution.md` states
    this.
  - The loaders lock parent rows before child rows. The store README says
    that writes lock the workspace before its children.
  - The own-machine loader takes `FOR NO KEY UPDATE` on the workspace row.
    It takes `FOR UPDATE` on the organization, the machine, the runner, the
    command row, and the organization activation. It also takes
    `FOR UPDATE` on every demand row of the scale set of the runner.
  - The Boat loader takes `FOR NO KEY UPDATE` on the workspace row. It takes
    `FOR UPDATE` on the Boat intent row, the organization, the delivery
    row, the runner, and every demand row of the scale set.
  - The guards delete only one of these rows: the Boat `Retired` result
    deletes the Boat intent row. They change one unique key: the
    own-machine `Retired` result writes a new unique `command_id` to the
    command row. They delete no other locked row and change no other key.
  - The listener write, `save` in `crates/ciproxy-store-pg/src/github/jobs.rs`,
    saves the demand row and then the attempt row in one transaction. The
    attempt row has a foreign key to the organization.
  - The first save of a Boat placement runs on the dedicated connection of
    the overflow session. `save` in `overflow/deliveries.rs` updates the
    delivery row. Then `placements::save` inserts the runner row, which
    has a foreign key to the organization.
  - Today the guards and the listener writes use the one shared store
    connection of each process. So the listener cycle needs two pods at
    once. A rolling update runs two pods, because the Helm chart uses
    `RollingUpdate` with `maxSurge: 1`.
  - The overflow session connection is apart from the shared connection, so
    the placement cycle can form inside one pod.
  - On 17 November 2026, commit `5db2e701` changed the workspace lock of
    both loaders to `FOR NO KEY UPDATE`. It broke the same class of cycle
    at the workspace row.
  - Its test `boat_result_and_delivery_cost_save_both_complete`, in
    `crates/ciproxy-store-pg/tests/store/overflow/guard_cost_tests.rs`,
    pauses a delivery save before its second write. It saves the placement
    before the guard runs, so the runner row already exists. So it does not
    cover the placement cycle.
  - The helpers `tagged_store` and `wait_for_blocker`, in
    `crates/ciproxy-store-pg/tests/store/overflow/guard_cost_support.rs`,
    order the statements of two connections in a test.
- **Problem:** Two lock cycles can form:
  - Listener cycle: a guard holds the organization row and waits for a
    demand row. A listener holds that demand row and inserts an attempt.
    The key-share lock of that insert waits for the guard.
  - Placement cycle: the first Boat placement holds the delivery row and
    inserts the runner row. Its key-share lock waits for the organization
    lock of the Boat guard. The guard waits for the delivery row.
- **Impact if no action:** PostgreSQL waits for its deadlock timeout and
  then aborts one transaction of the cycle. Recovery must repeat the
  listener write, the result, or the placement before the job can go on.
- **Root question:** How can result guards and the other writes of the same
  workspace complete without a database lock cycle?
- **Options:**
  - **A.** Change the organization locks in both loaders to
    `FOR NO KEY UPDATE`. Add an ordered PostgreSQL regression case. A also
    removes the placement cycle, but its one regression case covers only
    the listener. Effort: Small.
  - **B.** Share one non-key organization lock read between both loaders.
    Keep the regression case of A. Effort: Small.
  - **C.** Make each listener write take the workspace fence before it
    updates a demand row. Cost: more job traffic waits, and C does not
    remove the placement cycle. A is better than C on every point.
    Effort: Medium.
  - **D.** Use one lock rule for every row of both loaders.
    - Take `FOR NO KEY UPDATE` on each row. Keep `FOR UPDATE` only on a row
      that the guard deletes or whose unique key it changes.
    - These rows are the Boat intent row, which `Retired` deletes, and the
      command row, whose unique `command_id` changes.
    - Share the organization and demand-row lock reads between both
      loaders.
    - Write the rule in one sentence in
      `crates/ciproxy-store-pg/execution.md`.
    - Add two ordered regression tests with `tagged_store` and
      `wait_for_blocker`. The first runs a new listener attempt against each
      guard. The second runs a first Boat placement against the Boat guard.
    - Cost: more locks to check than in B. Each changed row is still
      blocked for every other update and delete.
    - Effort: Medium.
- **Left out:**
  - A retry of deadlock errors: it fails the goal. PostgreSQL still waits
    for its deadlock timeout and aborts one transaction each time.
  - A statement or lock deadline on the connections: it fails the goal. It
    does not remove either cycle. It only changes the error that ends one
    transaction.
  - A listener that takes the key-share lock on the organization before it
    updates the demand row: A is better on every point. A costs the same,
    also removes the placement cycle, and adds no wait to other listener
    writes.
  - A migration that drops the organization foreign key of the attempt
    table: it fails the goal for the placement. The runner row still needs
    its organization key. It also needs a migration.
  - Locking the demand rows before the organization row: not a real choice.
    It breaks the parent-first order of the store. An organization delete
    also cascades from the organization to its demand rows, in the
    opposite order.
  - Locking only the demand rows of the runner: not a real choice. A
    `Started` message could then give a job to the runner between the
    compare and the write. The guard exists to stop that.
  - Running all writers of a workspace in one process: not a real choice. A
    rolling update runs two pods at once, and the placement cycle forms
    inside one pod.
- **Wider fix:** D. B must know about the organization row, so it is a
  direct fix. The rule of D covers each row that a guard locks, also the
  new locks that `plans/follow-ups/late-start-label-policy.md` adds.
- **Previous examples:** Commit `5db2e701` of 17 November 2026 broke a lock
  cycle between a Boat result and a delivery cost save. It changed only the
  workspace lock of these two loaders, so the class came back at the
  organization row.
- **Related:**
  - `plans/follow-ups/late-start-label-policy.md`: the same code, and work
    that waits for the answer. Its fix adds label and binding checks to
    these loaders. Labels and scale sets have foreign-key children, such
    as scale sets, demand rows, and runners. New `FOR UPDATE` locks on them
    can repeat this class. So its new locks must follow the rule of this
    item. Apply this item first, or in the same change.
  - `plans/follow-ups/control-plane-async-store-boundary.md`: its option G
    adds a connection pool. A listener write and a guard can then use two
    connections in one pod, so the listener cycle happens in normal
    running. Apply this item before that pool, or with it.
  - `plans/follow-ups/routing-snapshot-bounded-reads.md`: the same code.
    That item narrows the rows that the guard loads read, and this item
    changes their lock modes. The options do not conflict. Both can go in
    one change, in either order.
- **Recommendation:** D, changed from B. B fixes the second case of a class
  whose first case was at the workspace row. Both loaders still take
  `FOR UPDATE` on five more kinds of rows, and the late-start fix adds
  more locks. D sets the rule once and tests both writers known today. B
  is the right direct fix for the smallest change. C fixes only the
  listener writer.
- **Action:** fix after merge, apply D, because it is Medium and Rare, and
  PostgreSQL ends each deadlock with an error.
