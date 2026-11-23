# Script step links open the wrong GitHub log

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1, finding 202.
- **Location:** `ts/app/src/view/jobs-usage.ts:141`,
  `crates/ciproxy-api/src/handlers/job_views.rs:42`,
  `crates/ciproxy-api/src/handlers/job_usage.rs:129`, and
  `docs/protocol/job-usage.md:89` and `:112`.
- **Reason:** Each exact option adds a GitHub read or a dependence on
  runner log details that no requirement covers, and no option is clearly
  best.

- **Terms:**
  - **Script placeholder:** the name "Run script", "Pre Run script", or
    "Post Run script". The control plane stores it in place of a step name
    that holds a line of job code.
  - **GitHub step number:** the number in the address
    `#step:<number>:<line>` that opens the log of one step on the GitHub
    job page.
  - **Timing read:** the GitHub read of a finished job's times. It runs
    only for some jobs, and only it sets the numeric GitHub job ID.
- **Background:**
  - The CPU and memory panel of the job page lists the steps of a job,
    numbered from 1 in their order. The steps come from the runner's
    Worker log in the VM, by the user's decision of 23 November 2026, so
    that no finished job costs a GitHub read.
  - By the user's decision of 23 November 2026, a script name becomes a
    placeholder, and the page links the placeholder to the log of that
    step on GitHub, so a person with GitHub access can see what ran. The
    same decision rejected names from GitHub for each page view, because
    each view would cost a GitHub read.
  - `stepHref` in `ts/app/src/view/jobs-usage.ts` adds `#step:<row>:1`,
    with the row number, to the `github_url` of the job.
  - `github_url` is the job page only when the timing read set the GitHub
    job ID. Otherwise it is the run page
    (`crates/ciproxy-api/src/handlers/job_views.rs:42`), and that is most
    jobs.
  - In runner `v2.338.2`, `src/Runner.Worker/ExecutionContext.cs:1384`
    reserves the orders N+1 to 2N for post steps when the first post step
    registers. So each post step and "Complete job" has a higher number
    on GitHub than its row. The GitHub job `137541629778` of
    `actions/setup-node` numbers its steps 1, 2, 3, 4, 8, and 9: its fifth
    row, "Post Run Dependabot", is step 8 on GitHub.
- **Problem:**
  - For most jobs, a script link opens the run summary and no step,
    because `github_url` is the run page. No test covers a run address.
  - On a job page address, the link of each post step opens the wrong
    step: the fifth row above gets `#step:5:1`, not `#step:8:1`.
  - `docs/protocol/job-usage.md:89` says that the page numbers the steps
    "as GitHub does". The steps component and the link words say "step N
    on GitHub". Both are wrong after the first post step.
- **Impact if no action:** A person who uses the link to see a hidden
  script must find the step by hand.
- **Root question:** How does ciproxy get the GitHub address of each step,
  which is the numeric job ID and the GitHub step number, so that a script
  placeholder opens the log of that step?
- **Options:**
  - **A.** Link to the job page, or to the run page when the job ID is
    unknown, and label the link that way. No new source, but no direct
    step links. Effort: Small.
  - **B.** Read the numeric step order of the runner from the Worker log,
    and carry it with each stored step. No GitHub read, but it needs proof
    that the stock runner gives enough detail in each case, and it gives
    no job ID. Effort: Large.
  - **C.** Use the step numbers of the finished job's GitHub response:
    reuse the timing read when it ran, and read the job when it did not.
    Keep the row numbers apart from the link targets, and guess no target
    when the match is not clear. Cost: one GitHub read for each finished
    job without a timing read. Effort: Large.
  - **D.** Point each script link at a new dashboard route. On a click,
    the API lists the jobs of the run with the installation token and
    matches the runner, as the timing read does. It then redirects to
    `#step:<GitHub number>:1` of that job, and it keeps no step name. When
    the step counts differ or the read fails, it redirects to the job page
    or the run page. It can keep the job ID and the numbers, so a second
    click needs no read. Cost: a new route, one GitHub read for each
    opened link, and a ciproxy address on hover. Effort: Large.
  - **E.** When the API serves the job page, read the steps of the job
    with the person's own GitHub sign-in. A person without access, a
    failed read, or a step count that differs gets the link of A. Cost:
    one GitHub read for each page view. Effort: Large.
  - **F.** Do B, and also read the numeric job ID from the job message
    that the stock runner writes to the Worker log
    (`src/Runner.Worker/Worker.cs:101`, field `job.check_run_id`). No link
    needs a GitHub read. Cost: two log details that are not a runner
    contract. GitHub sends the ID only when its flag
    `actions_add_check_run_id_to_job_context` is on, so a live job must
    prove it. The step list must also be complete. Effort: Large.
- **Left out:**
  - A fixed offset for post steps, because the gap changes with each job.
  - A rule in the API from the stored names, because the control plane
    replaces script names, and runner post steps such as "Stop
    containers" have no "Post" prefix.
  - An anchor by step name, because the GitHub job page opens a step log
    only from `#step:<number>:<line>`.
  - A patched runner, because Boat runs the stock runner.
- **Wider fix:** D. Like C and E, it takes the address from GitHub, so a
  change in how GitHub numbers steps cannot break it, and the app never
  builds a step address from a row number or a run address.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/job-usage-overlapping-steps.md`, the same
  code: B and F extend the step tracker of the guest that its options
  rewrite.
- **Recommendation:** D. It is the only exact option that keeps the
  reasons of both decisions of 23 November 2026: no GitHub read for each
  finished job, and none for each page view. Its read also gives the job
  ID that most jobs lack. Whatever the answer, correct
  `docs/protocol/job-usage.md:89` and the link words, and add a test with
  a run address and a test where the row number differs from the GitHub
  number.
- **Action:** ask the user, because D adds a route and a GitHub read that
  no requirement covers, and C, D, and F trade GitHub reads against runner
  log details with no clearly best choice.
