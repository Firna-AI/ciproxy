# A label size that no backend can run

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High. **Harder
  later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from pull request #1, branch
  `conductor/ciproxy-mvp-foundation-m2-docs`, review finding 2.
- **Location:** `docs/protocol/job-routing.md`, the label and fit rules.
  `docs/protocol/overflow.md`, the sandbox size rule.
  `docs/mockups/src/screens/settings/shape-dialog.tsx`.
- **Reason:** The recommended option adds behaviour at the boundary between
  ciproxy and Boat, so it waits for the user.

- **Terms:**
  - **Label:** the `runs-on` value of a job. It sets the vCPUs, the memory,
    and the disk of the job's VM.
  - **Sandbox:** one Boat VM, which runs one job.
- **Background:** The docs send a label that fits no own machine to Boat.
  The label dialog mockup shows a 32 vCPU, 128 GB label and says that its
  jobs run on Boat. Boat's largest sandbox has 16 vCPU and 32 GB. The Boat
  plan also limits the sizes: a trial cannot start `large`, and the $20
  plan cannot start `xlarge`. Boat's `GET /limits` returns the sizes that
  an account can start now.
- **Problem:** ciproxy can accept a label that no own machine and no Boat
  size can run. The default labels need Boat's `large` or `xlarge`, so a
  trial account overflows no default label, and the $20 plan cannot
  overflow `ciproxy-8vcpu`. The docs also do not say whether a label's disk
  is the whole VM disk or the free space, and Boat gives only free space.
- **Impact if no action:** Jobs of such a label wait in the GitHub queue
  until GitHub cancels them after 24 hours, while the dashboard promised a
  run.
- **Root question:** What should ciproxy promise and do for a label whose
  size fits no own machine and no size that the customer's Boat account
  can start?
- **Options:**
  - **A.** Document a waiting state, and correct the oversized example in
    the mockup. Jobs wait until a machine that fits connects. Effort: Small.
  - **B.** Reject a label that no connected backend can run, and show the
    supported sizes before save. This is a new rejection. Effort: Small.
  - **C.** Add an overflow provider with larger VMs. Effort: Large.
  - **D.** A, plus a backend function that reports the sizes that the
    customer's account can start now, from Boat's `GET /limits`. One rule
    then decides: own machines, Boat, or wait. The dialog, the label list,
    and the live queue show its result. The Boat spike reads the sizes of a
    $20 account. Effort: Medium.
  - **E.** Fail each job of such a label at once, with the reason in the
    GitHub log. This is a new rejection, and the job's secrets reach the
    runner. Effort: Large.
- **Left out:** Giving the job fewer resources, because the job can fail.
  Repeated Boat requests. Cancelling the job through the GitHub API,
  because it needs the Actions write permission, which breaks the
  "runner access only" promise.
- **Wider fix:** D. Every promise and every Boat start then follows the
  sizes that the backend reports.
- **Previous examples:** The dialog promise came in `d790413`, the docs
  labels page in `661d1f8`, and the plan design call in `aa5d2d9`.
- **Related:** The dialog locks "Your machines" off for such a label, but
  `job-routing.md` moves the label back to own machines when a machine
  that fits connects.
- **Recommendation:** D. It covers the oversized example and the default
  labels on small Boat plans, and B or C would still need its size list.
- **Action:** ask the user, because D adds behaviour at the boundary with
  Boat, and it changes a mockup promise.
