# No production code writes the routing reason

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the Jobs pages.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/wire/jobs.rs:176`,
  `crates/ciproxy-api/src/handlers/job_views.rs:80`,
  `crates/ciproxy-store-interface/src/jobs.rs:109`,
  `crates/ciproxy-store-pg/src/jobs.rs:65`,
  `crates/ciproxy-store-pg/src/job_views.rs:34`,
  `crates/ciproxy-scaler/src/actions.rs:62`,
  `crates/ciproxy-overflow/src/service/provider_preparation.rs:56`, and
  `docs/protocol/job-routing.md:169`.
- **Reason:** The fix is in the routing and runtime code, which the UI
  branch does not own.

- **Terms:**
  - **Routing reason:** the sentence that says why a job ran where it ran,
    such as "A warm runner was ready, so the job started at once."
- **Background:**
  - `docs/protocol/job-routing.md` says that for each job, the dashboard
    shows where it ran, how long it waited for a runner, and why it ran
    there. It gives two examples: "A warm runner was ready, so the job
    started at once." and "Your machines were full, so the job started on
    Boat after 30 seconds."
  - `JobView.routing_reason` at `wire/jobs.rs:176` is an optional string,
    "Saved routing explanation when supplied". `job_views.rs:80` copies it
    from the store.
  - The `jobs` table has a `routing_reason` column, and the store maps it
    at `store-pg/src/jobs.rs:65`. The job view query at
    `store-pg/src/job_views.rs:34` merges these rows with the GitHub job
    attempts, whose bodies carry no reason.
  - A search for `routing_reason` finds no production writer. Only tests
    set a value: `crates/ciproxy-api/tests/http/m18_jobs_support.rs:69` and
    `crates/ciproxy-store-pg/tests/store/jobs_tests.rs:43`.
  - The scaler already gives each runner start a typed `StartReason`, at
    `crates/ciproxy-scaler/src/actions.rs:62`: free capacity on an own
    machine, a vacancy in the warm pool, the overflow wait elapsed with the
    wait, no eligible own machine, or the largest Boat size. That is the
    intent of the start. No store keeps it. The overflow start keeps only
    `own_machine_fits`, which it derives from the reason, at
    `crates/ciproxy-overflow/src/service/start.rs:129-132`.
  - The execution can differ from that intent. The overflow service makes
    the Boat size choice again when it prepares the sandbox, with the
    current catalog, the current label setting, and the current size
    refusals, at
    `crates/ciproxy-overflow/src/service/provider_preparation.rs:31-59`.
    It drops the kind of that choice, a fitting size or the largest-size
    fallback, at `:56`. The runner of a job, and its place, are known only
    when a runner takes the job.
  - The job page says "Why the job ran here is not available." when the
    field is absent, and never builds a reason in the browser.
- **Problem:** Every real job has no routing reason, so the job page never
  says why a job ran where it ran.
- **Impact if no action:** The requirement of `job-routing.md` stays unmet.
  Each job that runs before a writer exists keeps no reason for its whole
  retention of 90 days, because later facts cannot rebuild it.
- **Root question:** Where does the reason of each placement come from, so
  that later changes cannot rewrite it?
- **Options:**
  - **A.** Record each placement fact where it becomes known, and keep the
    intent apart from the execution. The runtime saves the `StartReason` of
    each start as the intent of that start. The overflow service saves the
    kind of its size choice next to the chosen size, at the preparation.
    When a runner takes a job, the job gets the runner, its place, whether
    that runner was already idle and warm, and the start that made the
    runner. The API words the reason from the execution facts, and uses
    the intent only for why a runner started, such as full own machines
    and the overflow wait. This covers a ready warm runner, a fresh runner
    on an own machine, Boat after the overflow wait, Boat at once because
    no own machine can run the label, and the largest-size fallback.
    Effort: Large.
  - **B.** Derive the reason in the backend from retained execution facts
    only, such as a warm or a fresh runner, the place, and the wait, and
    give no reason when those facts do not decide it. Effort: Medium.
- **Left out:**
  - A reason built from current labels, settings, or machines, because an
    Admin can change them after a job ran.
  - A reason built in the browser, for the same cause.
  - Removing the reason from `job-routing.md`, and keeping only the place
    and the wait, because then no job says why it ran where it ran. That
    change of a requirement is a choice for the user.
  - Saving the `StartReason` of the scaler as the execution reason,
    because it is the intent of a start, and the size choice at the
    preparation can differ from it. One field must not hold both the
    intent and the execution.
- **Wider fix:** A. Each fact is saved where it becomes known, so a later
  change cannot rewrite it, and an intent cannot pass for an execution.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/jobs-notice-facts.md` D saves the kind of
  the size choice at the preparation, next to the chosen size. A of this
  item saves the same fact, so both changes should share one stored field.
  Neither item waits for the other.
- **Recommendation:** A. It saves the facts of each moment where they
  become known, so a later label or machine change cannot rewrite them.
- **Action:** fix after merge, apply A, because the change is backend work
  outside the UI branch, and the protocol already requires the reason.
