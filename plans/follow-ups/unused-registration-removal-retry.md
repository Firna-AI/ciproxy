# A failed removal of an unused runner registration is not retried

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** `crates/ciproxy-control-plane/src/remote_retirement.rs`;
  `crates/ciproxy-control-plane/tests/runtime/unused_registration_tests.rs`.
- **Reason:** Decision 20 of the plan.

- **Background:** GitHub keeps an unused JIT registration as an offline
  runner, so the control plane removes it when a runner exits without a
  job. The macOS test runs started and retired many warm runners.
- **Problem:** If the GitHub call fails, nothing tries again, so a stale
  offline runner stays in the customer's GitHub settings.
- **Options:**
  - **A.** Accept it. Effort: none.
  - **B.** Record each pending removal and retry it later. Effort: Small.
- **Recommendation:** B. First check whether the retirement path already
  retries a failed removal ("registration retirement remains pending").
- **Action:** fix after merge.
