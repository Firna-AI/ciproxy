# The savings report prices macOS jobs at Linux rates

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** the savings calculation and its GitHub price list
  (`crates/ciproxy-control-plane/README.md` names the source; the
  configuration in `crates/ciproxy-api/pages-and-billing.md`).
- **Reason:** Decision 19 of the plan.

- **Background:** GitHub charges more per minute for macOS runners than for
  Linux runners. The savings report has one GitHub price list for Linux.
- **Problem:** Savings from macOS jobs show too low.
- **Options:**
  - **A.** Add GitHub's macOS prices, and choose the list by the label's
    platform. Effort: Small.
  - **B.** Leave macOS jobs out of the report. Effort: Small.
- **Recommendation:** A.
- **Action:** fix after merge.
