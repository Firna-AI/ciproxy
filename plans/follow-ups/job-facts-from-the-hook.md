# Read job facts from the hook, not the API

- **Severity:** Low. **Effort:** Medium. **Confidence:** High. **Harder
  later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from the branch
  `calummoore/ci-proxy-load-balancer`, the user's answer on the source of
  the job facts.
- **Location:** `docs/protocol/security.md`, "GitHub App permissions", and
  `docs/protocol/cache.md`, "Trusted write-back".
- **Reason:** On 4 November 2026 the user chose the "Actions" read
  permission for the launch, and kept the hook as "a possible follow up".

- **Terms:**
  - **Trusted write-back:** only jobs on the default branch save cache
    entries that every branch can restore. It stops a pull request from
    poisoning the cache.
  - **Job-started hook:** a script that the GitHub runner runs in the job
    VM before the first step of the job.
  - **First report:** the one report of the job facts that the hook sends
    to the agent, which runs outside the VM. Later reports are refused.
- **Background:**
  - Trusted write-back needs the ref, the event, and the head repository of
    each job from a source that job code and its author cannot change.
  - At launch, the control plane reads these facts from the GitHub API,
    with the "Actions" read permission. This permission can also read the
    logs and the artifacts of every run, although ciproxy reads neither.
  - A spike on 4 November 2026 ran 31 real jobs on runner 2.338.2. The
    first report of the hook matched the GitHub API in each case: pushes,
    a pull request, a dispatch, reusable workflows, `pull_request_target`,
    and attempts to override the values in the workflow.
  - The hook ran before job containers, service containers, action
    pre-steps, steps, and composite actions. No tamper attempt changed a
    first report. A fork was not tested, because the test repository was
    private.
  - The hook path needs these conditions: a fresh trusted VM for each job,
    a receiver outside the VM, one first report for each runner, a pinned
    runner version, and a test that repeats the proof before each runner
    update.
- **Problem:** The App holds a permission that is wider than the facts that
  ciproxy needs.
- **Impact if no action:** Each customer grants read access to the logs and
  the artifacts of every run, and the site cannot promise runner access
  only.
- **Root question:** Can ciproxy learn the facts of each job with the two
  narrow permissions only?
- **Options:**
  - **A.** Keep the "Actions" read permission. Effort: Small.
  - **B.** Take the facts from the first report of the hook, as the spike
    proved, and drop the "Actions" permission. Add the test that repeats
    the proof for each runner update. Effort: Medium.
  - **C.** B on own machines, and keep A as the source when the hook report
    is missing. Effort: Medium.
- **Left out:** An organization webhook, because each owner must add it by
  hand, and jobs get no trusted writes until then.
- **Wider fix:** B, because it removes the wide permission.
- **Previous examples:** None found.
- **Recommendation:** B. GitHub lets an App drop a permission with no owner
  approval, so the move is cheap once the hook test exists.
- **Action:** ask the user, because the user chose A for the launch, and B
  changes the App permissions again.
