# A failed metrics load erases the cache counts

- **Severity:** High. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-16, from the options check of
  `machine-statistics-availability.md`.
- **Location:** `crates/ciproxy-cache-sidecar/src/metrics.rs:72-95`, the
  functions `load` and `persist`.
- **Reason:** The options check found this outside the decision that it
  checked, so a separate branch fixes it.

- **Terms:**
  - **Cache service:** the program on a machine that serves the machine
    cache and counts restores and hits for seven days.
- **Background:** `load` reads the saved counts. A failed read or decode
  gives empty records through `unwrap_or_default` and the catch-all arm.
  `persist` later writes the records over the same file.
- **Problem:** After one failed read or decode, the next save writes the
  empty records over the file, so the counts of the last seven days are
  lost.
- **Impact if no action:** The machine page and the cache hit share lose up
  to seven days of counts after a rare disk or decode error.
- **Root question:** How does the cache service keep its saved counts when
  it cannot read them?
- **Options:**
  - **A.** Do not save while the last load failed, and retry the load
    before the next save. New counts wait in memory. Effort: Small.
  - **B.** Move an unreadable file aside with a time in its name, log it,
    and start new counts. The old counts stay on disk for a manual repair.
    Effort: Small.
  - **C.** Merge on save: read the file again and add the new counts to it.
    A lasting decode error still blocks the merge. Effort: Medium.
- **Left out:** Stopping the cache service on a failed load, because the
  machine cache then stops serving jobs for a statistics error.
- **Wider fix:** B. No save can then overwrite counts that the service
  did not read.
- **Previous examples:** None found.
- **Related:** `machine-statistics-availability.md`: the same counts, and
  how the report marks them as unknown.
- **Recommendation:** B. It never loses the saved counts and keeps the
  service counting.
- **Action:** fix after merge, apply B, because it is separate from the
  branch that found it.
