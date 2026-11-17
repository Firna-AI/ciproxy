# Guest observations lack JSON checks

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the guest and release review check.
- **Location:** `crates/ciproxy-protocol/src/_tests_/wire/guest_tests.rs`
  and `crates/ciproxy-protocol/README.md`.
- **Reason:** These missing checks predate the guest command cleanup.

**Source:** Review. **Category:** Test.

**Terms:**

- **Guest observation:** A JSON message that reports a runner event from
  the job VM to the machine agent.
- **Wire snapshot:** A saved JSON value that checks the format exchanged
  between processes.

**Background:** The protocol crate defines both guest message directions.
Its README says each message variant has a file-based JSON snapshot.
The wire helper checks the message tag, JSON round trip, and snapshot.

**Problem:** `GuestToAgent::JobStarting`, `JobPrepared`, and
`RunnerConfigured` have no JSON check in the wire tests.
Existing agent and guest tests exchange typed Rust values through mocks.
They cannot detect a JSON field rename confined to these variants.

**Impact if no action:** An incompatible JSON change can pass the tests.
The agent and guest image can run different versions.

**Root question:** How should these guest observations retain their JSON
contracts?

**Options:**

- **A.** Add each missing shape to the existing wire test group.
  Use dummy credentials and the current snapshot helper. Effort: Small.
- **B.** Add explicit JSON and round-trip assertions for these shapes.
  This checks the same boundary without snapshot files. Effort: Small.

**Left out:** A schema-only check retains no example or JSON round trip.
A transport test adds infrastructure without changing the serde boundary.
The two useful approaches check the message values directly.

**Wider fix:** The existing protocol README and testing skill already
require this coverage. No new gate is needed.

**Previous examples:** The removed Docker command held live attachment
assertions that needed to move to `PrepareJob`.

**Recommendation:** A. It follows the protocol crate's existing pattern
and uses the production message types.

**Action:** Fix after merge, apply A, because the missing observation
checks predate this branch and need a separate change.
