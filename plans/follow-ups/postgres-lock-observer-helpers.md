# Two test files copy the PostgreSQL lock observer

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from `fix/boat-exit-report-claim-first`, review
  round 1. The check of finding 3 found it as a problem outside the scope.
- **Source:** Review.
- **Category:** Code structure.
- **Location:**
  `crates/ciproxy-store-pg/tests/store/overflow/guard_cost_support.rs:14-68`,
  `crates/ciproxy-store-pg/tests/store/metering/concurrent/support.rs`,
  the line `mod guard_cost_support;` in
  `crates/ciproxy-store-pg/tests/store/overflow/mod.rs`, and the line
  `pub(in crate::store) mod support;` in
  `crates/ciproxy-store-pg/tests/store/metering/concurrent/mod.rs`.
- **Reason:** Both copies are correct test helpers that predate the
  branch, and their merge changes no product code and no test result.

- **Terms:**
  - **Store tests:** the PostgreSQL integration tests of `ciproxy-store-pg`.
    They are one test target, `crates/ciproxy-store-pg/tests/store_tests.rs`,
    with modules under `crates/ciproxy-store-pg/tests/store/`.
  - **Lock observer:** test code that waits until PostgreSQL reports that
    one session waits for a lock that another session holds.
  - **`TaggedStore`:** a store whose connection has a unique
    `application_name`, so that `tagged_store` can find its backend process
    ID in `pg_stat_activity`.
  - **`backend_pid`:** a helper that returns the process ID of the current
    PostgreSQL session, from `pg_backend_pid()`.
  - **`wait_for_blocker`:** a helper that polls `pg_blocking_pids` until
    the waiter session waits for the blocker session.
- **Background:**
  - The concurrency cases of the store tests make two database sessions
    overlap in a known order. A lock observer shows that a session waits
    before the test releases the lock.
  - Two files define the same lock observer:
    - `crates/ciproxy-store-pg/tests/store/overflow/guard_cost_support.rs`
    - `crates/ciproxy-store-pg/tests/store/metering/concurrent/support.rs`
  - Each file defines `TaggedStore`, `tagged_store`, `backend_pid`, and
    `wait_for_blocker`. Each also defines the private row types `Backend`
    and `Blocking`.
  - At commit `bb71c61`, the two files matched line for line, except in
    four places:
    - the module doc comment
    - the visibility, `pub(super)` in the overflow file and
      `pub(in crate::store)` in the metering file
    - the `application_name` prefix before a new UUID,
      `guard-cost-operation-` or `metering-operation-`
    - the panic text of `wait_for_blocker`, "the tagged guard-cost worker
      did not reach its expected blocker" or "the tagged metering worker
      did not reach its expected blocker"
  - A later test can add a helper to one copy only. So the copies can
    differ in more places than these four.
  - Both copies have the same readiness checks:
    - `tagged_store` adds the `application_name` and a 20-second statement
      timeout option to the connection URL. The store connector applies
      its existing timeout policy. The helper then finds the backend
      process ID of that name in `pg_stat_activity`.
    - `wait_for_blocker` polls with `thread::yield_now`. Its 10-second
      deadline detects hung setup only, never the timing of a product
      outcome.
  - Each short path below is under `crates/ciproxy-store-pg/tests/store/`.
  - These test files use the copies:
    - `overflow/mod.rs` declares the overflow copy as a private module.
      Only `overflow/guard_cost_tests.rs` uses it.
    - `metering/concurrent/mod.rs` declares the metering copy with
      `pub(in crate::store)`. The four test files beside it use it.
    - `crates/ciproxy-store-pg/tests/store/github/timing_support.rs`
      imports the metering copy instead of adding its own.
  - No production code uses either copy.
  - No test reads the `application_name` prefix. The UUID alone keeps each
    tag unique.
  - The branch that added this item adds overlap tests for the overflow
    delivery writes. They use the metering copy, so the branch adds no
    third copy.
  - Three other test files have their own inline observers, with other
    names, arguments, and panic texts. They are not copies of the two
    files:
    - `crates/ciproxy-store-pg/tests/store/install_revocation_tests.rs`
      has `blocked`, `await_machine_insert`, and a `tagged_store` that
      takes a tag and returns a tuple.
    - `crates/ciproxy-store-pg/tests/store/key_erasure_tests.rs` has
      `wait_for_insert_blocker`.
    - `crates/ciproxy-store-pg/tests/store/machine_snapshot_tests.rs` has
      `wait_for_reader`.
- **Problem:** The store tests keep two copies of one lock observer. A fix
  to one copy, such as a change to the readiness query or to the deadline
  for hung setup, can miss the other copy.
- **Impact if no action:** The copies can drift apart, so two test areas
  can check the same database wait in different ways. A new concurrency
  test can also add a third copy.
- **Root question:** How do the store tests share one PostgreSQL lock
  observer?
- **Options:**
  - **A.** Keep the metering copy as the only copy. Delete
    `overflow/guard_cost_support.rs` and its module line. Point
    `overflow/guard_cost_tests.rs` at
    `crate::store::metering::concurrent::support`, as
    `github/timing_support.rs` already does. Cost: overflow and GitHub
    tests keep a dependency on a metering module, which is a misleading
    home. Effort: Small.
  - **B.** Move the lock observer to one shared module of the store test
    support:
    - Use a new file such as
      `crates/ciproxy-store-pg/tests/store/support/locks.rs`, and declare
      it in `support/mod.rs`.
    - Give its items `pub(in crate::store)` visibility.
    - Take the tag prefix as a parameter, or use one neutral prefix.
    - Keep every helper of both copies, also a helper that only one copy
      has.
    - Point each current user at it, and delete both old copies.
    - Keep the statement timeout, the process ID lookup, the blocker query,
      the 10-second deadline, every readiness check, and every assertion.
    - Effort: Small.
  - **C.** Do B, and also move the three inline observers to the shared
    module. Each caller keeps its own panic text through a parameter. Cost:
    three more files with other signatures change, and each failure message
    needs care. Effort: Medium.
- **Left out:**
  - Change nothing: it fails the goal, because the two copies stay.
  - A new crate for test helpers: B is better on every point, because the
    helpers serve only this test target.
  - The helpers in the product crate behind a test feature: B is better on
    every point, because it shares the helpers without product code.
  - A macro that writes each copy: B is better on every point, because the
    macro still compiles two copies and hides them.
- **Wider fix:** B. One shared module in the store test support gives each
  later concurrency test one place to find the lock observer.
- **Previous examples:** None found of a merge of these helpers.
  `github/timing_support.rs` already imports the metering copy instead of
  adding its own.
- **Related:**
  - `plans/follow-ups/result-guard-lock-rule.md`: its option D adds two
    ordered regression tests with `tagged_store` and `wait_for_blocker`.
    Neither item waits for the other. The later change points those tests
    at the shared module.
  - `plans/follow-ups/overflow-lifecycle-canonical-writes.md`: the overlap
    tests of the overflow delivery writes use the metering copy. Neither
    item waits for the other. B moves those tests to the shared module.
- **Recommendation:** B. A neutral home with store-wide test visibility
  removes the copy, and it sits beside the shared test fixtures. A is
  smaller, but it keeps overflow and GitHub tests on a metering module.
- **Action:** fix after merge, apply B, because the copies predate the
  branch, sit outside its diff, and cause no wrong test result.
