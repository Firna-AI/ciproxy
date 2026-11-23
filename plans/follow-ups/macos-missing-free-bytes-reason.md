# A Mac with an older agent shows no reason for no jobs

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from PR #141, branch `calummoore/macos-s-disk-real-use`,
  decision 2 of its review.
- **Location:** `crates/ciproxy-agent/src/root_sizing.rs`, `RootFit::Unknown`;
  `crates/ciproxy-control-plane/src/capacity.rs`; the Mac page and the label
  fit of `ts/app`.
- **Reason:** The reason needs a new field from the API to the app, and
  only an agent from before PR #141 meets it.

- **Terms:**
  - **Image free bytes:** `ImageMetadata::free_job_bytes`, the measured
    free space inside a macOS image disk. The agent reports it since
    PR #141.
- **Background:** On 23 November 2026 the parent session chose that a Mac
  whose agent reports no image free bytes gets no new macOS job until it
  does. `RootRule::fit` gives `RootFit::Unknown`, so the scaler places no
  job there, and the label fit of the dashboard shows unknown facts.
- **Problem:** The Mac page and the label fit do not say that the agent
  must be updated.
- **Impact if no action:** A customer with an older agent on a Mac sees no
  macOS jobs start and no reason.
- **Root question:** How does the dashboard tell the customer to update
  the agent of such a Mac?
- **Options:**
  - **A.** Show "Update the agent to run macOS jobs" on the Mac page when
    its selected image has no free bytes. Effort: Medium.
  - **B.** Give the label fit a reason per machine, with this case as one
    reason. Effort: Medium.
- **Left out:** A refusal at the agent: an older agent cannot be changed.
- **Wider fix:** B. One reason field also covers other unknown facts.
- **Previous examples:** Commit `0ade9745` kept an unknown fit apart from a
  known lack of fit in Jobs.
- **Recommendation:** A.
- **Action:** fix after merge, apply A.
