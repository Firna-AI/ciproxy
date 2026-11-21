# The App keeps the wide Actions read permission

- **Severity:** Low. **Effort:** Large. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from the branch
  `calummoore/ci-proxy-load-balancer`, the user's answer on the source of
  the job facts. Rewritten on 2026-11-18, from round 2 of the final review
  of 17 November 2026 of that branch, finding 1207.
- **Location:** `docs/protocol/security.md`, "GitHub App permissions";
  `docs/protocol/cache.md`, "Restore order" and "Trusted write-back";
  `crates/ciproxy-github/src/rest/http.rs:124`,
  `crates/ciproxy-github/src/rest/cache_metadata.rs:51`,
  `crates/ciproxy-github/src/rest/job_times.rs:41`, and
  `image/runner-patch/CiproxyAssignment.cs:21`.
- **Reason:** On 4 November 2026 the user chose the "Actions" read
  permission for the launch, and kept the hook as "a possible follow up".

- **Terms:**
  - **"Actions" read:** a repository permission of the GitHub App. It
    reads the runs, the jobs, and the cache entries of GitHub Actions. It
    can also read the logs and the artifacts of every run, although
    ciproxy reads neither.
  - **Trusted write-back:** only jobs on the default branch save cache
    entries that every branch can restore. It stops a pull request from
    poisoning the cache.
  - **Checked token:** the runtime token of a job on an own machine. The
    machine checks its signature, issuer, validity, and assignment outside
    the job VM. Its signed write scope gives the full ref of the job.
  - **Entry facts:** the ID, ref, key, version, and creation time of an
    entry of the GitHub cache, as GitHub reports them.
  - **Worker hook:** the code that the image builds into the Worker of the
    GitHub runner, in `image/runner-patch/CiproxyAssignment.cs`. On an own
    machine, it reads the original job request before action setup. It
    hands the cache URLs, the runtime token, and the cache rights of the
    job to the host, and it holds the steps until the host decides. The
    host trusts only its first handoff, the guest message `JobStarting`.
    Boat runs the original runner, with no Worker hook.
  - **Scale-set message:** a message about a job that GitHub sends to the
    control plane through the session of a scale set.
- **Background:**
  - The App asks for four permissions, as `docs/protocol/security.md`
    says: "Self-hosted runners", read and write; "Metadata", read;
    "Members", read; and "Actions", read. Without "Actions", three remain.
    "Members" read checks the person's own organization memberships and
    ownership, so the App already asks for more than runner access.
  - The full ref of a job comes from the checked token, not from the
    GitHub API. `docs/protocol/security.md` and "Trusted write-back" in
    `docs/protocol/cache.md` reject a hook as the source of the ref,
    because the checked token needs no trust in the order of the runner
    setup. The user chose the checked token on 5 November 2026.
  - The control plane uses "Actions" read for three reads.
    `crates/ciproxy-control-plane/src/composition.rs` connects them in
    production: the entry list at line 108, the job-time recovery at line
    161, and the run facts at line 205, for the listeners in
    `crates/ciproxy-control-plane/src/listeners.rs:155`:
    1. The run facts, `run_facts` at
       `crates/ciproxy-github/src/rest/http.rs:124`. The listener reads the
       event, the head repository, and the branch name of each new run. The
       control plane sends the event and the head repository to the agent,
       for trusted write-back and the Docker disks. The job history shows
       the branch name, and never trusts it as a ref.
    2. The entry list of the GitHub cache, at
       `crates/ciproxy-github/src/rest/cache_metadata.rs:51`. "Restore
       order" in `docs/protocol/cache.md` needs these entry facts, so that
       one order covers the machine cache and the GitHub cache. It decides
       that the entry facts come from GitHub through the control plane. It
       rejects a ref from the job, because job code can choose it.
    3. The job-time recovery, at
       `crates/ciproxy-github/src/rest/job_times.rs:41`, which `RepairTiming`
       in `crates/ciproxy-github/src/timing.rs` runs. It reads the numeric
       Actions job ID, the start and end times, and the result of a job
       attempt, when ciproxy lacks them. Metering and the job history match
       a job by that ID, and the job page links to the job on GitHub with
       it.
  - The scale-set message carries the event, as `eventName`, and the
    repository of the run. It carries no head repository, no branch name,
    and no numeric Actions job ID. Its `jobWorkflowRef` names the ref of
    the workflow file, which differs from the ref of the job in a reusable
    workflow.
  - On 4 November 2026, the plan left `jobWorkflowRef` and `eventName` out
    as sources of the job facts, with one reason: the ref in the message
    is the ref of the workflow file. That reason does not apply to the
    event.
  - A spike on 4 November 2026 ran 31 real jobs on runner 2.338.2 with a
    job-started script hook. That script ran in the job VM before the first
    step, and it sent one first report of the job facts to a receiver
    outside the VM. The first report matched the GitHub API in each case:
    pushes, a pull request, a dispatch, reusable workflows,
    `pull_request_target`, and attempts to override the values in the
    workflow.
  - The script hook ran before job containers, service containers, action
    pre-steps, steps, and composite actions. No tamper attempt changed a
    first report. A fork was not tested, because the test repository was
    private.
  - The spike named these conditions for a hook source: a fresh trusted VM
    for each job, a receiver outside the VM, one first report for each
    runner, a pinned runner version, and a test that repeats the proof
    before each runner update.
  - The spike tested the script hook, not the Worker hook. The Worker hook
    reads the original job request before action setup, so it is a
    candidate source. Its proof must run again.
- **Problem:** The App holds a permission that also reads the logs and the
  artifacts of every run. A new source of the job facts alone cannot drop
  it, because the entry list of the GitHub cache and the job-time recovery
  also need it.
- **Impact if no action:** Each customer grants read access to the logs
  and the artifacts of every run, although ciproxy reads neither.
- **Root question:** Can the App drop the "Actions" read permission, and
  which source replaces each read that needs it?
- **Options:**
  - **A.** Keep the "Actions" read permission for all three reads, and
    close this item. Effort: Small.
  - **B.** Replace each read, then drop the permission. Effort: Large.
    - The event: `eventName` of the scale-set message, which GitHub sends
      for each job. On an own machine, the Worker hook can also add the
      event from the original job request to `JobStarting`.
    - The head repository: the Worker hook adds it from the original job
      request to `JobStarting`. Only own machines need it, because the
      GitHub cache applies its own scopes on Boat.
    - Repeat the proof of the spike for the Worker hook, with a test that
      repeats it before each runner update.
    - The branch name of the job history: a blocker. No source gives it
      for each job. The checked token gives the full ref on own machines
      only. `jobWorkflowRef` is wrong for a job in a reusable workflow.
    - The entry list of the GitHub cache: the main blocker. No other
      source gives the entry facts. The decision of "Restore order" must
      change first, and the user decides it. Without the entry facts, one
      order cannot cover both caches.
    - The job-time recovery: a blocker. The scale-set messages give the
      assignment time, the finish time, and the result, but not the
      numeric Actions job ID.
    - Update `docs/protocol/security.md`, `docs/protocol/cache.md`, the
      site docs, and each copy of the App permission list.
    - GitHub lets an App drop a permission with no owner approval, so the
      last step needs nothing from the customers.
- **Left out:**
  - An organization webhook, because each owner must add it by hand, and
    jobs get no trusted writes until then.
  - A hook as the source of the full ref, because the checked token gives
    it with no trust in the order of the runner setup.
  - A ref or entry facts from the job, because job code can choose them.
  - The hook on own machines, with the GitHub API as the source when the
    hook report is missing, as the earlier option C proposed: it keeps the
    permission, so it cannot answer the root question.
- **Wider fix:** None. B removes the permission itself.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/cache-sync-github-deletions.md`. Its option B syncs
    deletions through the entry list of the GitHub cache, which needs
    "Actions" read. B of this item would block that option. Neither item
    waits for the other, but their answers must agree.
  - `plans/follow-ups/shared-app-permission-list.md`. B of this item
    changes each copy of the App permission list. Neither item waits for
    the other.
- **Recommendation:** A, changed from B. B now needs more than a new
  source of the job facts: it needs new sources or new decisions for three
  reads, and its main blocker changes "Restore order". It gains only a
  narrower permission, and ciproxy already reads no logs or artifacts with
  the current one.
- **Action:** ask the user, because the user chose A for the launch, and B
  changes the App permissions and the decision of "Restore order".
