# The billing lock can outlive its session

- **Severity:** Medium. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from the complete gate on the merge of pull
  request #27. The test is outside that diff.
- **Source:** Implementation.
- **Category:** Test.
- **Location:** `crates/ciproxy-store-pg/src/billing/queries.rs`, the
  function `try_lock_billing`, and
  `crates/ciproxy-billing/tests/_tests_/checkout_tests.rs:53`.
- **Reason:** The flake is rare, and the fix belongs to the billing branch
  of Milestone 17, not to the disk change that showed it.

- **Terms:**
  - **Billing lock:** the PostgreSQL session advisory lock that serializes
    the billing commands of one workspace.
  - **Billing session:** the object that holds the billing lock on its own
    database connection.
- **Background:** `try_lock_billing` opens a new connection and takes
  `pg_try_advisory_lock`. The billing session keeps that connection. When
  the session is dropped, the connection closes, and PostgreSQL releases
  the lock when the server process of that connection ends.
- **Problem:** The release happens after the drop returns, not during it.
  A billing command that starts at once after another can still see the
  lock and fail with `Busy`. The test
  `checkout_lost_reply_and_concurrent_callers_keep_one_session_until_confirmation`
  drops the lock and opens a checkout at once. It failed once with
  "called `Result::unwrap()` on an `Err` value: Busy" under the load of the
  complete gate. It then passed 10 times alone and 6 times in its full test
  file.
- **Impact if no action:** The gate fails now and then for no real defect.
  A person who starts two billing actions quickly can get a needless
  "busy" answer.
- **Root question:** How does the end of one billing command free the lock
  before the next command can ask for it?
- **Options:**
  - **A.** Release the lock with `pg_advisory_unlock` before the session
    closes its connection, so the release is done when the drop returns.
    Add the regression with the test above under parallel load. Effort:
    Small.
  - **B.** A, and have each command wait a short, bounded time for the
    lock. This adds a wait and a time limit. Effort: Small.
  - **C.** Use a transaction-level advisory lock, which the commit or the
    rollback frees. This changes the lock scope of each command. Effort:
    Medium.
- **Left out:** A retry or a sleep in the test, because it hides the
  release race. Marking the test as ignored, because it covers live code.
- **Wider fix:** A. An explicit release makes every caller safe.
- **Previous examples:** None found.
- **Recommendation:** A. It removes the race with no new wait.
- **Action:** fix after merge, apply A, because the defect is in the
  billing store, outside the disk change that showed it. Reproduce the
  flake first, and record the pass counts before and after.
