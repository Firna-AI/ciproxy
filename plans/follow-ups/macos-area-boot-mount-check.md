# The area volume mount after a reboot is not tested on a Mac

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** `crates/ciproxy-agent/src/platform/macos/area.rs`
  (the fstab line by UUID).
- **Reason:** Decision 22 of the plan: Cal reboots the test Mac when he is
  at it, because FileVault can stop the Mac at the password screen.

- **Task:** reboot the test Mac, then check that `/var/db/ciproxy/area`
  is mounted, that the agent starts with no hand step, and that a job runs.
  Repeat with the area on an external disk.
- **Impact if no action:** A customer's Mac might take no jobs after a
  restart until someone acts.
- **Action:** a check on the test Mac.
