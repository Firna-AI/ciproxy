# New jobs wait while the listener is down

- **Severity:** Medium. **Effort:** Small. **Confidence:** High. **Harder
  later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from pull request #1, branch
  `conductor/ciproxy-mvp-foundation-m2-docs`, review finding 1.
- **Location:** `docs/protocol/job-routing.md`, the warm runner and outage
  rules. `docs/architecture.md`, the listener. The design call on warm
  runners in `plans/ciproxy-mvp-foundation.md`.
- **Reason:** The fix narrows a requirement in force, so it waits for the
  user.

- **Terms:**
  - **Listener:** the part of the hosted control plane that keeps a message
    session open with GitHub for each scale set.
  - **Message session:** the long poll through which GitHub learns the free
    capacity of a scale set and hands out its jobs.
  - **Warm runner:** a runner that registers before a job arrives, so that
    a job starts at once.
- **Background:** The docs promise that warm runners on own machines keep
  taking new jobs while the hosted control plane is down. GitHub dropped
  the step in which a listener accepts each job, but it still hands out
  jobs only to a scale set whose listener holds a message session. In ARC
  issue 4432, warm runners stayed online with no listener, and new jobs
  stayed queued. The `actions/scaleset` README says that GitHub uses the
  capacity from each poll to assign jobs.
- **Problem:** The outage promise does not hold. When the listener stops,
  new jobs wait in the GitHub queue, also when warm runners are free.
- **Impact if no action:** The implementation cannot meet the written
  promise, and customers expect CI to continue through an outage when it
  does not.
- **Root question:** What job-start promise does ciproxy make during an
  outage of its hosted service?
- **Options:**
  - **A.** Promise only that jobs that a runner already took finish, and
    that queued jobs start when the service is back. Align
    `job-routing.md`, `architecture.md`, and the plan's design call.
    Effort: Small.
  - **B.** Run job intake on customer hardware during an outage. This needs
    the GitHub App key on customer machines, which `security.md` forbids.
    Effort: Large.
  - **C.** Run the listener as a small separate service with a standby, so
    that most control plane failures do not stop new jobs. GitHub seems to
    allow one session per scale set, so the standby must take over the
    session. The scale set spike checks this. Effort: Medium.
  - **D.** Wider fix: each requirement that depends on GitHub or Boat
    behaviour names its source, a vendor doc or a spike, or carries
    `Open:` until a spike settles it. This is a new repository rule.
    Effort: Medium.
  - **E.** Register warm runners as plain self-hosted runners with the
    `ciproxy` label, outside the scale set. GitHub serves them without a
    listener. This needs a spike, and the agent must report their jobs for
    metering. Effort: Large.
- **Left out:** More warm runners, because the dependency on the listener
  stays. Marking the rule `Open:` until the spike, because public reports
  already show that the rule fails.
- **Wider fix:** D. Finding 2 of the same review is a second case of a doc
  that states a vendor behaviour without a source.
- **Previous examples:** The design call on warm runners in the plan, from
  3 November 2026.
- **Related:** `plans/follow-ups/label-size-no-backend-can-run.md`, which
  shares the wider fix. Also: an idle Boat runner gets no job during the
  outage, and nothing ends its sandbox, so Boat bills for the whole outage
  (`docs/protocol/overflow.md`). The scale set spike must crash the
  listener rather than stop it cleanly, and must check that queued jobs
  start after the outage (ARC issue 4027).
- **Recommendation:** A. It promises only what GitHub is known to do, and
  the spike can widen the promise later. C is the next step if new jobs
  must start through partial outages.
- **Action:** ask the user, because A narrows the outage requirement that
  the plan and the docs state.
