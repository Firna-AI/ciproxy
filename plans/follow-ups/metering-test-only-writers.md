# Metering keeps cost writers that only tests call

- **Severity:** Low. **Effort:** Large. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`: option C of the check of
  finding 1007, with step 4 of the producer list that the same check
  found.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-store-interface/src/metering/store.rs`,
  `MeteringStore::save_machine_cost`, `save_plan_cost`, and
  `save_boat_cost`, their code in
  `crates/ciproxy-store-pg/src/metering/adapter.rs`, and
  `crates/ciproxy-metering/README.md`, step 4 of the list "Trusted fact
  producers must supply these inputs".
- **Reason:** The change moves 20 test calls in 12 files across four
  crates, and the severity is Low.

- **Terms:**
  - **`MeteringStore`:** the store trait of the metering facts, in
    `crates/ciproxy-store-interface/src/metering/store.rs`.
  - **Production writer:** the store code that saves a metering fact in the
    transaction that owns the fact.
  - **Producer list:** the numbered list "Trusted fact producers must
    supply these inputs" in `crates/ciproxy-metering/README.md`.
  - **Cost facts:** the price facts of metering: the monthly price of an own
    machine, of the tier and add-ons, and the prices of one Boat runner.
- **Background:**
  - `MeteringStore` has three cost writers that no production code calls:
    `save_machine_cost`, `save_plan_cost`, and `save_boat_cost`.
  - 20 test calls in 12 files use them, in the crates `ciproxy-api`,
    `ciproxy-control-plane`, `ciproxy-overflow`, and `ciproxy-store-pg`.
  - Production writes each of these facts in the transaction that owns it:
    - Machine costs: `cost` in
      `crates/ciproxy-store-pg/src/dashboard/machines.rs`, the save of a
      machine price from the dashboard.
    - Plan costs: `save` in `crates/ciproxy-store-pg/src/billing/costs.rs`,
      which `BillingSession::commit` calls.
    - Boat costs: `save` in
      `crates/ciproxy-store-pg/src/overflow/placements.rs`. It writes
      `metering_boat_costs` from the delivery record.
  - Step 4 of the producer list says: "Save each ciproxy Boat runner's
    prices with save_boat_cost." No production code calls
    `save_boat_cost`.
  - Several store tests rewrite a Boat cost to imitate later readings. The
    production writer takes that cost from the delivery record.
  - On 17 November 2026 the final review chose to remove the unused period
    storage of metering in the branch. It also chose to delete
    `record_machine_registration` and step 2 of the list, because each
    machine registration now saves its own time. Neither change removes
    these three cost writers.
  - "Replacing an old way" in the `plans` skill says that when a change
    replaces a way, every consumer moves, and the old way goes.
- **Problem:** Each cost fact has two writers: the production writer in the
  owning transaction, and a store method that only tests call. Step 4 of
  the producer list names the method that only tests call.
- **Impact if no action:** Tests can build a fact in a way that production
  never uses, so they can pass while the production writer is wrong. A
  reader of the README calls a method that production does not use.
- **Root question:** How does each cost fact keep one writer, in the
  transaction that owns it, while the tests still build that fact?
- **Options:**
  - **A.** Correct step 4 of the producer list only. Name the Boat price
    write of `overflow/placements.rs`, and keep the three methods for the
    tests. Cost: two writers stay for each fact. Effort: Small.
  - **B.** Move the three methods out of `MeteringStore` into one
    test-support module, as typed helpers that write the same rows. Correct
    step 4. Cost: the helpers still bypass the production writers.
    Effort: Medium.
  - **C.** Delete `save_machine_cost`, `save_plan_cost`, and
    `save_boat_cost` with their adapter code. Each cost fact then has one
    writer, in the transaction that owns it.
    - The tests build these facts through the production writers, or
      through typed helpers in one test-support module.
    - Correct step 4 of the producer list.
    - C drops the assumption that a metering fact can keep a second writer
      on `MeteringStore` that only tests call. The source is the code. Its
      only reason is test convenience, not a requirement.
    - Cost: 20 test calls in 12 files across four crates. The store tests
      that rewrite a Boat cost need another way to imitate later readings.
    - Effort: Large.
- **Left out:** A lint for store methods with no production caller: a new
  lint needs the user. The rule "Replacing an old way" already covers the
  class.
- **Wider fix:** None. The rule "Replacing an old way" of the `plans` skill
  already covers the class. Each option must name every leftover, so each
  option is a direct fix.
- **Previous examples:** Round 1 of the same review found that paid plans
  stayed free in savings. A plan-cost writer that no product code called
  stood beside the billing commit. Commit `3b082ee` of 17 November 2026
  fixed it.
- **Related:** `plans/follow-ups/metering-refresh-reversal.md`: the same
  code. Both items change
  `crates/ciproxy-store-pg/tests/store/metering/concurrent/boat_tests.rs`.
  The options do not conflict, and neither item waits for the other.
- **Recommendation:** C. Each cost fact then has one writer, in the
  transaction that owns it. So no test can pass on a write path that
  production never takes.
- **Action:** fix after merge, apply C, because it moves 20 test calls in 12
  files across four crates, and the severity is Low.
