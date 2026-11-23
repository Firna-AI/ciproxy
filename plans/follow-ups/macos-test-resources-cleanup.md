# Clean up the macOS test resources

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** outside the repository.
- **Reason:** These resources served the macOS tests only.

- **Task, when the macOS testing ends:**
  1. Remove the test Mac from the `firna-test-org` workspace in main
     preview, and uninstall its agent.
  2. Delete the spike repository `calummoore/ciproxy-macos-spike`; decide
     whether `firna-test-org/ciproxy-macos-spike` stays for later tests.
  3. On the test Mac, delete `/etc/sudoers.d/ciproxy-test`, `~/vz`,
     `~/ws-parent`, and `~/ws-i/secrets` (the R2 keys).
  4. Remove the test Mac's Tailscale node and the reusable auth key.
  5. On the external SSD, keep only the build folders that the next image
     build needs.
  6. Delete the R2 tokens that nothing uses any more.
- **Action:** post-merge task.
