# The Mac agent is signed with an Individual Developer ID

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** the GitHub environment `macos-signing`;
  `crates/ciproxy-agent/scripts/macos-release.sh`.
- **Reason:** Decision 1 of the plan: Individual membership now; Cal
  upgrades it to an Organization.

- **Background:** `ciproxy-agent` 0.1.5 for `aarch64-apple-darwin` is
  signed by "Developer ID Application: Calum Moore (UV85J3PKBX)" and
  notarized.
- **Task:** after the Organization upgrade, make a new Developer ID
  Application certificate and App Store Connect key for the company team,
  replace the five secrets and `MACOS_SIGNING_TEAM_ID`, and release. Note
  that macOS ties a Full Disk Access grant to the team, so Macs that
  granted it to the old team must grant it again.
- **Action:** post-merge task, after the upgrade.
