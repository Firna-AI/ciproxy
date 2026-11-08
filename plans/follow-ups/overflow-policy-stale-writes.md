# A Boat settings save can overwrite newer values

- **Severity:** Medium. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from Milestone 22, pull request #51, its handoff.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** the Overflow settings route of `crates/ciproxy-api`, and
  the Boat settings of `ts/app`.
- **Reason:** The fix needs backend work outside the Milestone 22 scope.

- **Background:** The Overflow settings save the whole policy: the wait, the
  spend limit, and the Boat state. Two tabs or two Admins can edit them.
- **Problem:** A save from an older page writes back the values that it
  read, also those that another Admin changed since.
- **Impact if no action:** A newer spend limit or wait can be lost
  silently.
- **Root question:** How does a save change only what its person changed?
- **Options:**
  - **A.** Send only the changed fields, and refuse a save whose base
    version is older than the stored one, with a clear reload message.
    Effort: Medium.
- **Left out:** Last write wins, because a lost spend limit costs money.
- **Wider fix:** A.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, because it needs a backend change
  outside the Milestone 22 scope.
