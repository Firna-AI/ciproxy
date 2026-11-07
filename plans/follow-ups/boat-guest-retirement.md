# Idle Boat sandboxes keep running after an App removal

- **Severity:** Medium. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-07, from plan item 89, option B, which the user asked
  to review later.
- **Source:** Review, Milestone 15, pull request #35, its finding 2.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/boat_authority.rs`, after
  pull request #35.
- **Reason:** The user chose A for the MVP: after an App removal, idle Boat
  sandboxes end at their lifetime limit. This item keeps B for a later
  review.

- **Terms:**
  - **Boat sandbox:** a rented Boat VM that runs one overflow job.
  - **Handover:** the moment, just before a job's steps start, when the
    guest in the VM receives the job.
- **Background:** ciproxy finds idle Boat sandboxes through GitHub. An App
  removal ends that access, and the guest reports only when its runner
  exits, so ciproxy cannot tell idle from busy.
- **Problem:** Idle sandboxes bill the customer's Boat account until their
  lifetime ends: six hours, or two hours on a trial.
- **Impact if no action:** Each App removal can cost the customer some idle
  Boat time, within the spend limit.
- **Root question:** How does ciproxy stop idle Boat cost after an App
  removal, without stopping running jobs?
- **Options:**
  - **A.** Keep the lifetime exception of item 89. Effort: Small.
  - **B.** Ask each sandbox's guest to retire itself through Boat's trusted
    controller. A busy guest keeps its job, and a job taken before the
    handover can fail, as item 79 = A allows on own machines. Effort: Large.
  - **C.** Add a confirmed handover receipt before the steps start, with a
    new callback contract and image. Effort: Large.
  - **D.** Patch the runner so that it stops taking jobs and keeps accepted
    work, with a live GitHub proof. Effort: Large.
- **Left out:** Deleting every sandbox, because it stops running jobs.
  Reading the status and then deleting, because a job can start in between.
- **Wider fix:** B. Retirement and handover share one protection point.
- **Previous examples:** Plan item 79, the same problem on own machines.
- **Recommendation:** B, when App removals or idle Boat cost become common.
- **Action:** ask the user, because the user asked to review B later.
