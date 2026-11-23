# Delivery writes rely on two copies and locks

- **Severity:** Low. **Effort:** Large. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from `fix/boat-exit-report-claim-first`, review
  round 1, findings 2 and 3. It records the shared wider fix of finding 2,
  option C, and of finding 3, option B. Round 2 added the placement case
  of finding 6, option C, to the same work.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-store-pg/src/overflow/deliveries.rs:17-99`,
  `crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`,
  `crates/ciproxy-store-pg/src/overflow/dispatch.rs:34-45`,
  `crates/ciproxy-schema/migrations/202611051400_overflow/up.sql:6-21`,
  `crates/ciproxy-store-pg/src/overflow/starts.rs:107-117`,
  `crates/ciproxy-store-pg/src/boat.rs:144-169`,
  the Boat registration ownership view migration,
  `crates/ciproxy-store-pg/src/overflow/rows.rs:88-96`, and
  `crates/ciproxy-store-pg/src/boat_results.rs:169-179`.

  ```text
  crates/ciproxy-schema/migrations/202611082137_boat_registration_ownership/up.sql:29-41
  ```

- **Reason:** The current code is correct, and this redesign changes
  correct readers and the saved data format beyond the test gaps that the
  branch fixed.

- **Terms:**
  - **Delivery:** one row of the table `overflow_starts`, which saves the
    start of one Boat sandbox, its runner, and its end.
  - **`StartRecord`:** the Rust type of a delivery, in
    `crates/ciproxy-store-interface/src/overflow/record.rs`. The row keeps
    it as JSON in the `record_json` column.
  - **Lifecycle facts:** the phase of a delivery, such as `Deleting` or
    `Ended`, the first exit report time `reported_at`, and the first end
    time `ends_at`.
  - **Exit claim:** the saved exit report of a sandbox. It sets
    `reported_at` and the phase `Deleting`, and it uses up the one-use
    token of that sandbox.
  - **Row lock:** a PostgreSQL `SELECT ... FOR UPDATE`. Each other writer
    of that row then waits until the locking transaction ends.
- **Background:**
  - ciproxy rents a Boat sandbox, a VM, for a job when the customer's own
    machines are busy. When the runner exits, the guest program in the
    sandbox sends one exit report with a one-use token.
  - The workspace owner chose option B for the exit report on 22 November
    2026:
    - The control plane saves the exit claim first, without the workspace
      lock. That lock is the PostgreSQL advisory lock that lets one
      operation at a time manage the Boat work of a workspace.
    - It then waits up to the existing 5-second lock limit and deletes the
      sandbox in the same request.
    - Any failure after the saved claim answers 204, and a later cleanup
      pass deletes the sandbox. A busy lock and a failed provider delete
      are such failures.
    - Before the save, a bad message or a runner ID that is not a UUID
      gets 400. A bad, used, or foreign token, or an ended delivery, gets
      401. A store failure gets 503.
    - One rule covers every save from any copy. The first `reported_at`
      and the first `ends_at` stay. The phase never moves back from
      `Deleting` or `Ended`.
    - The owner rejected a longer lock wait, a grace time before the
      job-end delete, retries from the guest, and shorter lock holds. The
      owner also rejected a check at arrival, a separate received fact, and
      a claim that never waits for the lock.
  - These docs hold the requirements in force:
    - `docs/protocol/overflow-sandbox-end.md`, section "The exit report",
      states the one claim for each sandbox, the save before the delete,
      the 5-second wait, the later cleanup, and the first report and end.
    - `crates/ciproxy-control-plane/overflow.md`, section "Exit callback",
      gives each answer. It gives 204 after the save, and 503 when the
      request fails before the save.
    - `crates/ciproxy-store-interface/src/overflow/README.md` gives the
      save rules of the store, also for a save from an older copy.
  - The current storage is in
    `crates/ciproxy-schema/migrations/202611051400_overflow/up.sql`:
    - Each delivery row keeps its `StartRecord` as JSON in `record_json`.
      The JSON holds all three lifecycle facts.
    - The row also has the typed columns `reported_at` and `ends_at`. So
      the store keeps two copies of each time. The phase is only in the
      JSON.
    - The partial index `overflow_starts_live_key` covers the rows whose
      `ends_at` is empty.
  - Each short path below is under `crates/ciproxy-store-pg/src/`.
  - These readers use the JSON copy:
    - The overflow snapshot, in `overflow/rows.rs`, gives the service each
      `StartRecord`.
    - The Boat result guard, in `boat_results.rs`, builds its
      `BoatDeliveryGuard` from the JSON.
  - These readers use the typed columns:
    - The exit claim accepts a row only when both columns are empty, in
      `overflow/completion.rs`.
    - The 90-day collection deletes rows by `ends_at`, in
      `overflow/starts.rs`.
    - Key erasure keeps a retired Boat key while a delivery of that key has
      no `ends_at`, in `boat.rs`.
    - The `boat_registration_ownership` view, in its migration under
      `crates/ciproxy-schema/migrations/`, counts a registration as terminal
      only when each matching delivery has an `ends_at`, among other
      conditions.
  - The current writes are correct:
    - `reserve` in `overflow/deliveries.rs` inserts a new `Unsent` delivery
      with both columns empty.
    - `save` in the same file serves the exit claim, dispatch, refusals,
      and each session save. It reads the saved JSON with a row lock.
    - `retain_lifecycle` then merges the saved lifecycle facts into the new
      record. `save` writes the JSON and both columns from that one merged
      record.
    - In the same transaction, `save` clears the sealed launch input when
      the phase no longer needs it. It also saves the runner placement and
      its cost facts.
    - The exit claim, `accept` in `overflow/completion.rs`, reads its row
      with a row lock and its filter, and then calls `save`.
    - Dispatch, in `overflow/dispatch.rs`, also reads the row with a row
      lock before it calls `save`.
    - Each transaction runs at the default READ COMMITTED level, through
      `transaction` in `crates/ciproxy-store-pg/src/gateway_transaction.rs`.
      So the row locks order the overlapping writes.
  - `overflow/placements.rs` also uses the merged phase for its proposed
    runner state. A claim moves a `starting` runner to `stopping`.
    Passing the older `Running` copy would move it back to `starting`.
    The upsert still keeps `ready`, `busy`, and `stopped` against that
    proposal. The ordered stale-save test checks this placement copy too.
  - The store tests under `crates/ciproxy-store-pg/tests/store/overflow/`
    check these rules. The branch that added this item adds three kinds of
    cases:
    - a retry of a used token after an older save
    - an older save of an ended delivery
    - overlapping writes that wait for a row that the test holds
  - This item covers the problems of two review findings of that branch:
    - Finding 2: the tests of older saves read only the JSON copy. So a
      write of either column from the caller's older copy passed every
      test.
    - Finding 3: the store tests did not make two writes overlap. So the
      removal of a row lock could pass.
  - The working agent of that branch made a design call on 22 November
    2026. The branch keeps the current product code and adds the missing
    regression tests. This item holds the shared redesign for later.
  - This item is follow-up work that the agent selected. The owner approved
    option B, not this redesign.
  - Each option below keeps these rules:
    - the first `reported_at` and `ends_at`, and no phase move back from
      `Deleting` or `Ended`, also for a save from an older copy
    - one accepted exit claim for each token, and only before the recorded
      end
    - 503 when the request fails before the claim commits, and 204 after
      the claim commits
    - the `StartRecord` values that the snapshot and the result guard give
      today
    - the runner placement and cost writes in the same transaction as
      their delivery write
    - the claim before the workspace lock, the wait of up to 5 seconds,
      the delete in the request, and the later cleanup of option B
- **Problem:** Each delivery writer must follow two rules by hand, and no
  type or statement enforces them:
  - Two stored copies. Each write must set the JSON and both typed columns
    from one merged record. A column that comes from another record splits
    the copies.
  - A separate locked read. Each write reads the row with a row lock, and
    a later statement writes it. Under READ COMMITTED, only that lock stops
    an overlapping write from acting on an old row.
- **Impact if no action:** No product bug exists today. But a later change
  to one delivery writer can break a rule while the other copy still looks
  right:
  - A `reported_at` column from an older copy lets a used token claim
    again, with a 204 and a second delete.
  - An `ends_at` column from an older copy allows a first claim after the
    end. It also keeps a retired Boat key and a registration identity, and
    it stops the 90-day collection.
  - A removed row lock lets two overlapping claims both succeed, or lets an
    older save erase a claim.
  - Only the regression tests catch these mistakes.
- **Root question:** How should lifecycle writes keep saved reports and
  ends correct during concurrent work?
- **Options:**
  - **A.** Keep the current locked merge, with the stronger regression
    tests. Each writer still follows both rules by hand. Effort: Small.
  - **B.** Build the JSON and both columns from one changeset type that
    only the merged record can create. Use the same type for the insert in
    `reserve`. A column then cannot come from another record. The two
    copies and the separate locked read stay. Make the placement writer
    take the private merged type too, so it cannot take the older input.
    Effort: Small.
  - **C.** Give the store one locked change function for a delivery row.
    It locks the row, passes the current record to a change, and saves the
    result with the lifecycle merge. The exit claim, dispatch, and every
    delivery save use it. A new writer then cannot act on an unlocked read.
    The lock still sits in one place that an edit can remove, and the two
    copies stay. Effort: Medium.
  - **D.** Make the JSON the single source of the lifecycle facts. The
    exit claim, the collection, key erasure, the registration view, and the
    live-key index then read the JSON. A forward migration replaces the
    view and the index, and then drops both columns. Cost: the SQL depends
    on serde field names that the compiler does not check. An atomic write
    must then merge JSON in SQL. The column drop must pass the dead-code
    rule of the `git-commit` skill. Effort: Medium.
  - **E.** Run the exit claim and each delivery save at REPEATABLE READ,
    and retry after a serialization failure. PostgreSQL then rejects a
    write to a row that changed after the snapshot. Cost: a retry path and
    an error class that no other store write has. A save inside a READ
    COMMITTED caller loses the protection, and the two copies stay.
    Effort: Large.
  - **F.** Make typed columns the single source of the lifecycle facts, and
    give each write one atomic statement:
    - Keep the `reported_at` and `ends_at` columns. Add a typed phase
      column, because the merge rule reads the saved phase.
    - A forward migration fills the phase column from the saved JSON.
    - Each reader builds the lifecycle facts of its `StartRecord` from the
      columns. The snapshot and the result guard then give the same values
      as today.
    - The exit claim becomes one `UPDATE` whose `WHERE` clause holds its
      filter. It returns the claimed record, or no row.
    - Each save becomes one `UPDATE` that keeps the first `reported_at` and
      `ends_at`, for example with `COALESCE`, and applies the phase rule to
      the current row.
    - The runner placement and cost writes stay in the transaction of
      their delivery write. The placement uses the canonical phase that
      the statement returns, with the current state-preservation rule.
    - Cost: it changes correct readers, the saved data format, and every
      delivery writer. Effort: Large.
- **Left out:**
  - More assertions on the JSON copy only: it fails the goal, because it
    cannot see a change to one column.
  - A `COALESCE` or a trigger on each column alone: it fails the goal.
    Each column then follows its own rule, so the copies become more
    independent.
  - A CHECK constraint or a trigger that rejects a changed column, or a
    test helper that compares the columns with the JSON: each is a
    detection fix, and structural fixes exist. A new rejection also needs
    the user.
  - A claim fact apart from the delivery row, such as a received-at column
    that only the exit route writes: it breaks option B, which the owner
    chose over that approach on 22 November 2026.
  - A lint: it fails the goal, because no text rule can tell which read
    feeds a write.
- **Wider fix:** F. It keeps one copy of each lifecycle fact, and each
  write applies its rule to the current row in one statement. So no copy
  can drift, and no lock can go missing in those writes. If F is declined,
  B is the Small structural fix for the copies, and C for the locked read.
- **Previous examples:** Commit `3b082ee` of 17 November 2026 keeps newer
  job timing facts when a save of other facts comes from older input.
  Commit `5db2e70` of 17 November 2026 forces a write and a retirement
  check to overlap in a store test.
- **Related:**
  - `plans/follow-ups/result-guard-lock-rule.md`: the same code. That item
    changes the lock modes of the Boat result guard, which also locks the
    delivery row. F changes how each delivery write reads and locks that
    row. The options do not conflict. The later change keeps the ordered
    regression tests of the earlier one.
  - `plans/follow-ups/postgres-lock-observer-helpers.md`: the overlap
    tests of these writes use the PostgreSQL lock observer that that item
    merges into one helper. Neither item waits for the other.
- **Recommendation:** F. It removes both rules that each writer must
  follow by hand, and it keeps every rule above.
- **Action:** fix after merge, apply F, because the current code is
  correct, and F changes correct readers and the saved data format beyond
  the test gaps of the branch.
