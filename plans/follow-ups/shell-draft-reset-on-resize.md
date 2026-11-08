# Crossing 640 pixels clears form drafts

- **Severity:** Low. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from Milestone 22, pull request #51, its handoff.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** the dashboard shell of `ts/app/src/components/shell/`.
- **Reason:** The shell comes from Milestone 19, outside the Milestone 22
  scope.

- **Background:** The shell switches its layout at a width of 640 pixels.
- **Problem:** The switch mounts the page again, so an unsaved form loses
  its draft.
- **Impact if no action:** A person who rotates a tablet or resizes a
  window loses typed values.
- **Root question:** How does a layout switch keep the page state?
- **Options:**
  - **A.** Keep one page tree, and change only the shell's layout around
    it. Effort: Small.
- **Left out:** Saving each draft in storage, because it adds state for a
  layout problem.
- **Wider fix:** A.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, because the shell belongs to
  Milestone 19, which is merged.
