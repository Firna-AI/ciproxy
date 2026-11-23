# macOS "When nothing fits" doc and screen differ

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, the protocol docs work.
- **Location:** `docs/protocol/labels.md`, section "Platform", the line
  "\"When nothing fits\" of a macOS label offers only \"Wait\"", and
  `docs/mockups/src/screens/settings/label-platform.tsx`, the function
  `WhenNothingFitsField`.
- **Reason:** The difference is older than the provider interface change,
  and that change keeps the macOS label behaviour as it is.

- **Source:** Implementation.
- **Category:** Docs or spec.
- **Terms:**
  - **"When nothing fits":** a label setting. It says what the jobs of the
    label do while no place of its order fits their size: wait, or run on
    the largest provider size.
- **Background:** A macOS label runs only on the customer's Macs, because a
  cloud Mac bills 24 hours at least. So its jobs always wait when no Mac
  fits. The protocol doc says that the setting of a macOS label offers only
  "Wait". The approved mockup of the label dialog shows no "When nothing
  fits" field at all for a macOS label.
- **Problem:** The doc and the screen describe the same behaviour in two
  different ways. A reader of the doc expects a field with one choice.
- **Impact if no action:** The next change to the label dialog can build
  the field from the doc, or drop it from the doc, with no rule to choose.
  The behaviour that customers get stays the same.
- **Root question:** How does the dashboard show that the jobs of a macOS
  label wait when no Mac fits?
- **Options:**
  - **A.** Change the doc to say that a macOS label has no "When nothing
    fits" choice, and that its jobs wait. Effort: Small.
  - **B.** Change the mockup and the app to show the field with "Wait" as
    its only choice. Effort: Small.
- **Left out:** None.
- **Wider fix:** None, because one doc line and one component hold the
  rule.
- **Previous examples:** None found.
- **Recommendation:** A. A field with one fixed choice gives the reader
  nothing to do.
- **Action:** ask the user, because the user decides which side of a doc
  and mockup difference is right.
