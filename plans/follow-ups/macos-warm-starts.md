# macOS VMs boot cold; saved-state starts are an opt-in for later

- **Severity:** Low. **Effort:** Large. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** `crates/ciproxy-agent/src/platform/macos/`
  (`SAVED_STATES` is off); `crates/ciproxy-vz`.
- **Reason:** Decision 18 of the plan: cold boots now, warm starts later
  only if customers ask.

- **Background:** Each macOS job VM boots macOS from scratch, about 25 s
  until the runner is ready on the test Mac. Warm runners still boot before
  a job. Restoring a saved VM state works only in a logged-in user's
  desktop session; it fails as root in the system domain. That needs
  automatic login, which macOS does not allow with FileVault on.
- **Options:**
  - **A.** Keep cold boots. Effort: none.
  - **B.** An opt-in mode: a dedicated user stays logged in, FileVault is
    off, and the agent restores saved states. Effort: Large.
- **Recommendation:** A until customers ask.
- **Action:** decide when demand appears.
