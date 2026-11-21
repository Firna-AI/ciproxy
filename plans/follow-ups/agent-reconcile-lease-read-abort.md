# A failed lease read stops the agent's reconcile

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-16, from the options check of
  `machine-statistics-availability.md`.
- **Location:** `crates/ciproxy-agent/src/controller.rs:135`, the call of
  `cache_budget` inside `reconcile`.
- **Reason:** The options check found this outside the decision that it
  checked, so a separate branch fixes it.

- **Terms:**
  - **Reconcile:** the periodic agent step that refreshes the cache budget,
    enforces the machine mode, and retries the network cleanup.
  - **Lease:** a Docker disk reservation of a running job, read from disk.
- **Background:** `reconcile` computes the job disk budget, then calls
  `self.cache.refresh_budget(self.cache_budget(budget)?)`. `cache_budget`
  reads the leases. After it, `reconcile` calls `enforce_mode` and
  `retry_network_cleanup`.
- **Problem:** The `?` on `cache_budget` returns from `reconcile` when the
  lease read fails, so `enforce_mode` and `retry_network_cleanup` do not
  run. A lasting lease read error stops both.
- **Impact if no action:** A mode change, such as a drain, and the retry of
  a failed network cleanup wait as long as the lease read fails.
- **Root question:** How does one failed step of the reconcile leave the
  other steps running?
- **Options:**
  - **A.** Log a failed `cache_budget` like a failed `refresh_budget`, and
    go on to `enforce_mode` and `retry_network_cleanup`. Add a regression.
    Effort: Small.
  - **B.** Run each reconcile step on its own, collect the errors, and
    return the first one after all steps ran. Add a regression. Effort:
    Small.
- **Left out:** Retrying the lease read inside `reconcile`, because a
  lasting error still stops the later steps.
- **Wider fix:** B. No later step then depends on an earlier one that can
  fail for its own reason.
- **Previous examples:** The same function already logs a failed
  `refresh_budget` and goes on.
- **Related:** `docs/protocol/docker-disks.md#unavailable-figures`: the same
  lease read. Reports now mark unread figures as unavailable. This item
  concerns the separate reconcile steps that the failed read still stops.
- **Recommendation:** B. It keeps each step independent, at the same cost.
- **Action:** fix after merge, apply B, because it is separate from the
  branch that found it.
