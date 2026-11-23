# An older Mac agent can keep idle runners for new jobs

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from PR #141, branch `calummoore/macos-s-disk-real-use`,
  review finding 12.
- **Location:** `crates/ciproxy-agent/src/root_sizing.rs`, `RootFit::Unknown`;
  `crates/ciproxy-scaler/src/lifecycle.rs`, `retire_obsolete` and
  `claim_existing`.
- **Reason:** It needs an agent from before PR #141 with an idle runner
  that registered before the update of the control plane.

- **Terms:**
  - **Image free bytes:** `ImageMetadata::free_job_bytes`, the measured
    free space inside a macOS image disk. The agent reports it since
    PR #141.
  - **Idle runner:** a runner in the state Starting or Ready, which GitHub
    can give a job.
- **Background:** On 23 November 2026 the parent session decided that a
  Mac whose agent reports no image free bytes gets no new macOS job until
  it does, and that its running jobs go on. `RootRule::fit` gives
  `RootFit::Unknown` there, so the scaler starts no new VM on that Mac. But
  `claim_existing` still lets a waiting job claim an idle runner of that
  Mac, and no rule retires the idle runner.
- **Problem:** An idle runner of such a Mac can take a new job.
- **Impact if no action:** During an agent update, an older Mac can run one
  more job per idle runner. That job gets the disk of the old count.
- **Root question:** How does a Mac without image free bytes finish its
  running jobs and take no new ones?
- **Options:**
  - **A.** Retire the idle runners of a Mac whose selected image has no
    free bytes, through the existing retirement. Keep busy runners.
    Effort: Small.
  - **B.** Use one machine rule for new starts and idle runners under the
    missing-fact policy, in `retire_obsolete` and `claim_existing`. Test a
    Mac with one ready and one busy runner. Effort: Medium.
  - **C.** Stop the GitHub registrations of such runners until the Mac
    reports the facts. Effort: Large.
- **Left out:** A skip in the claim loop alone: the runner stays registered
  and GitHub can still give it a job.
- **Wider fix:** B.
- **Previous examples:** Commit `1c486f90` reconciles idle supply with
  routing eligibility when a destination changes.
- **Recommendation:** B.
- **Action:** fix after merge, apply B.
