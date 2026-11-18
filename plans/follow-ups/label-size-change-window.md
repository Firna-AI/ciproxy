# Label doc omits the old-size runner window

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 1001 found.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/labels.md`, "Custom labels", the rule on a
  size change, `crates/ciproxy-scaler/src/lifecycle.rs`, `retire_obsolete`,
  and `crates/ciproxy-control-plane/runners.md`.
- **Reason:** The window lasts at most one pass of 10 seconds, and a change
  of the rule needs the user.

- **Terms:**
  - **Idle runner:** a registered runner with no job. A warm runner is an
    idle runner that the scaler keeps ready for the next job.
  - **Pass:** one reconciliation of a workspace. It runs every 10 seconds.
- **Background:**
  - "Custom labels" in `docs/protocol/labels.md` says: "After the customer
    changes the size of a label, each job of the label that a runner takes
    gets the new size."
  - It goes on: "So ciproxy stops the runners of the old size that have no
    job, and it starts runners of the new size." And: "A job that runs
    during the change keeps its VM."
  - The scaler stops an idle or warm runner of the old size at its next
    pass. `obsolete` in `crates/ciproxy-scaler/src/lifecycle.rs` returns
    `LabelChanged` when the size of the label differs from the requested
    size of the runner.
  - `crates/ciproxy-control-plane/runners.md` says that reconciliation runs
    every ten seconds.
  - GitHub gives a queued job to any idle runner of the scale set at once.
  - So a job that GitHub queues between the label save and the next pass
    can still run on an idle runner of the old size.
  - No option can close this window fully. GitHub can give a queued job to
    an idle runner at any moment before the stop reaches that runner.
  - `docs/protocol/job-routing.md` already states a short window of the
    same kind for Docker disks: "a Docker disk reserves only spare disk, so
    it stops no start, except for a short time after a label or image
    change."
- **Problem:** The protocol doc says that each job that a runner takes after
  a size change gets the new size. But an idle or warm runner of the old
  size can still take a job until the next pass stops it. The doc does not
  say so.
- **Impact if no action:** A customer who reads the doc expects the new size
  at once. A job that GitHub queues within about 10 seconds of the save can
  run with the old size. A rerun of the job is the workaround.
- **Root question:** What should the label rules promise about the jobs that
  runners take just after a size change?
- **Options:** Only two approaches are real. "Change nothing" fails, as
  **Left out** says.
  - **A.** State the window in "Custom labels" of `docs/protocol/labels.md`.
    An idle or warm runner of the old size stops at the next pass, within
    about 10 seconds. So a job that GitHub queues in that time can still
    run with the old size. Cost: it changes a requirement, so it needs the
    user. Effort: Small.
  - **B.** Stop the idle runners of the old size from the label save
    itself, before the save returns, and keep the doc. Cost: the label
    writers then need the runtime and GitHub calls. A job can still reach
    an idle runner between the save and the stop, so the doc still promises
    more than the code does. Effort: Medium.
- **Left out:** Change nothing: it fails the goal, because the doc keeps a
  promise that the code does not keep.
- **Wider fix:** None. The `documentation` skill is in force for docs that
  do not match the code.
- **Previous examples:** The rule for Docker disks in
  `docs/protocol/job-routing.md` states the same kind of window after a
  label change.
- **Related:** `plans/follow-ups/late-start-label-policy.md`: the same rule
  of `docs/protocol/labels.md`. That item closes another path to the old
  size: a runner whose registration wait spans the label save. Neither
  item waits for the other.
- **Recommendation:** A. It states the behaviour that no option can remove
  fully, as the doc already does for Docker disks. B costs more and still
  cannot keep the full promise.
- **Action:** ask the user, because A changes a requirement of
  `docs/protocol/labels.md`, and no recorded user decision covers that
  change.
