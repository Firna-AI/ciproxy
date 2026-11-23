# A Mac agent loses its control connection with no cause in the log

- **Severity:** Medium. **Effort:** Small to Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/support-mac-os-kq1u`, the end of
  the plan `plans/macos-own-machines.md`.
- **Location:** `crates/ciproxy-agent/src/runtime.rs`,
  `crates/ciproxy-agent/src/gateway.rs`; the gateway of the control plane.
- **Reason:** The cause is not known yet, and the log hides it.

- **Background:**
  - On 23 November 2026, the agent on the test Mac in main preview lost its
    control connection at 15:07, 15:50, and 15:56 UTC. Each time it logged
    `control connection failed; preserve VMs` with only
    `[agent/connection] internal error`, kept its VMs, and reconnected.
  - At least one loss came during a main preview deploy. The others may
    too, but this is not confirmed.
  - The image download continued through the 15:07 loss, so the resumable
    download worked as planned.
- **Problem:** A loss outside a deploy would stop job starts on the Mac
  until it reconnects, and the log does not say why.
- **Impact if no action:** Real network or gateway faults look the same as
  deploys, so nobody sees them.
- **Root question:** Does a Mac agent lose its control connection only at
  deploys, and if not, why?
- **Options:**
  - **A.** Apply `plans/follow-ups/agent-error-log-causes.md` to the
    connection and gateway paths first, then compare the times with the
    deploy runs and the gateway log. Effort: Small.
  - **B.** Add a connection-close reason to the gateway log and the agent
    report. Effort: Medium.
- **Recommendation:** A, then B if the losses come outside deploys.
- **Action:** fix after merge.
