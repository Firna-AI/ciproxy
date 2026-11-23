# Script step links open the wrong GitHub log

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 1, finding 202.
  Review round 2 checked it again, added options G and H, and kept D with
  a new reason.
- **Location:** `ts/app/src/view/jobs-usage.ts:140-141`,
  `crates/ciproxy-api/src/handlers/job_views.rs:42`,
  `crates/ciproxy-api/src/handlers/job_usage.rs:130`,
  `docs/protocol/job-usage.md:91-92` and `:114`,
  `docs/mockups/src/fixtures/job-usage.ts:39`,
  `docs/mockups/src/screens/jobs/usage-steps.tsx:3`, `:42`, and `:45`, and
  `ts/app/guides/jobs/README.md:48`.
- **Reason:** Each exact option adds a GitHub read, an App setting, or a
  dependence on runner log details that no requirement covers, and no
  option is clearly best.

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
  - `stepHref` in `ts/app/src/view/jobs-usage.ts:141` adds
    `#step:<row>:1`, with the row number, to the `github_url` of the job.
    The API sets that number to the row position
    (`crates/ciproxy-api/src/handlers/job_usage.rs:130`).
  - `github_url` is the job page only when the timing read set the GitHub
    job ID. Otherwise it is the run page
    (`crates/ciproxy-api/src/handlers/job_views.rs:42`), and that is most
    jobs.
  - The images pin runner `v2.338.2` (`image/pins.env:11`). The paths that
    start with `src/` are in its repository `actions/runner` at that tag.
  - In that runner, `src/Runner.Worker/ExecutionContext.cs:1388`
    reserves the orders N+1 to 2N for post steps when the first post step
    registers. So each post step and "Complete job" has a higher number
    on GitHub than its row. The GitHub job `137541629778` of
    `actions/setup-node` numbers its steps 1, 2, 3, 4, 8, and 9: its fifth
    row, "Post Run Dependabot", is step 8 on GitHub. The control plane
    stores that name as "Post Run script"
    (`crates/ciproxy-overflow/src/job_usage_names.rs:8`), so its row links
    to `#step:5:1`.
  - The step link uses the shared `TextLink`. Its caller `StepTitle` passes
    the address, the title, and the screen-reader words
    (`ts/app/src/screens/jobs/usage-steps.tsx:50-58`). So a fix changes
    only the view model and those words.
  - The page keeps its own row numbers from 1, in order. GitHub shows no
    step numbers, and a gap would look like a lost step. So each option
    keeps the rows and corrects only the claim that they are GitHub
    numbers.
  - The guest keeps every step of new Worker log pages, so a dropped early
    step no longer moves the later rows. The cap of 1,000 steps
    (`crates/ciproxy-protocol/src/usage.rs:16`) can still cut the list.
  - ciproxy stores no step name from GitHub, by the user's decisions of 23
    November 2026. So an option that receives names in a reply or an event
    drops them at once.
  - The checks pin the current link: the `steps` browser check expects
    "opens step 4 on GitHub" and `#step:4:1` on a job address
    (`ts/app/scripts/browser-pages-job-usage.ts:105-113`), and so does
    `ts/app/_tests_/jobs-usage-view.test.ts:111`.
- **Problem:**
  - For most jobs, a script link opens the run summary and no step,
    because `github_url` is the run page. No test covers a run address.
  - On a job page address, the link of each post step opens the wrong
    step: the fifth row above gets `#step:5:1`, not `#step:8:1`.
  - `docs/protocol/job-usage.md:91-92` says that the page numbers the
    steps "as GitHub does". The `title` and `accessibilityLabel` of the
    link in `StepTitle` say "step N on GitHub"
    (`ts/app/src/screens/jobs/usage-steps.tsx:56-57`). Both are wrong after
    the first post step.
  - The mockup says that each row number is "the number of the step on
    GitHub" (`docs/mockups/src/fixtures/job-usage.ts:39`), and its words
    say "step N on GitHub" (`docs/mockups/src/screens/jobs/usage-steps.tsx:3`,
    `:42`, and `:45`). The mockups are the spec of the app.
  - The comment at `ts/app/src/view/jobs-usage.ts:140` says that the link
    opens "the page of the job, open at the step", and
    `ts/app/guides/jobs/README.md:48` says that it links "to that step on
    GitHub". For most jobs, the address is the run page.
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
    that the stock runner gives enough detail in each case. Alone, it
    gives no job ID, so for most jobs the link still opens the run page. F
    completes it. Make the new step field optional, so that an older guest
    still decodes. Effort: Large.
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
    needs a GitHub read.
    - Carry the job ID inside the steps. The decode of `UsageReport`
      (`crates/ciproxy-protocol/src/usage/wire.rs:65`) drops every figure
      for a bad value outside the steps.
    - Cost: two log details that are not a runner contract, and the step
      list must be complete. GitHub sends the ID only when its flag
      `actions_add_check_run_id_to_job_context` is on. The runner test
      `src/Test/L0/Worker/ExecutionContextL0.cs:1319` says that the server
      now always sends it, but a live job must still prove it.
    - Effort: Large.
  - **G.** Subscribe the App to the `workflow_job` webhook event.
    - The App already has the "Actions" read permission that the event
      needs (`docs/protocol/security.md:139`).
    - On `completed`, match the job by `run_id` and `runner_id`, as the
      timing read does (`crates/ciproxy-github/src/rest/job_times.rs:55`).
      Keep the numeric job ID and the GitHub step numbers in order, with
      no names.
    - The API pairs rows with numbers only when the counts match.
      Otherwise it gives the link of A. No link needs a GitHub read.
    - Cost: the App owner must turn the event on in the App settings on
      GitHub, so the agent cannot apply G alone. GitHub then sends three
      or four events for every job of every organization, also for jobs
      that ciproxy does not run.
    - Cost: G needs a table, a migration, and the 90-day cleanup. A lost
      delivery gives the link of A, because ciproxy redelivers only
      installation and repository events
      (`crates/ciproxy-github/src/redelivery.rs:80`). Each event holds step
      names, so the handler must drop them before it logs or stores
      anything.
    - Effort: Large.
  - **H.** Do E with the installation token, once for each job.
    - When a person first opens a finished job page with a script row,
      the API lists the jobs of the run and matches the runner. It keeps
      the job ID and the numbers, with no names, and later views use the
      kept values.
    - A failed read or a count that differs gives the link of A, and a
      later view tries again. The link is a plain GitHub address, with no
      route.
    - Cost: one GitHub read for each finished job that a person opens,
      which is more reads than D. The first view is slower, or the page
      needs a second request. H needs a table.
    - Effort: Large.
- **Left out:**
  - A fixed offset for post steps, because the gap changes with each job.
    The runner reserves the orders N+1 to 2N for post steps
    (`src/Runner.Worker/ExecutionContext.cs:1388`).
  - A rule in the API from the stored names, because names cannot count
    the post steps:
    - The runner adds post steps with no "Post" prefix, such as "Create
      custom image" (`src/Runner.Worker/JobExtension.cs:565`) and "Stop
      containers".
    - An author can name a main step "Post comment".
    - The rule also gives no job ID.
  - An anchor by step name, because the GitHub job page opens a step log
    only from `#step:<number>:<line>`.
  - A read in the browser with the person's GitHub token, because E makes
    the same read in the API, with no token in the browser.
  - The scale-set messages, because they carry a job UUID, not the
    numeric job ID, and no step numbers
    (`crates/ciproxy-github/src/listener/messages.rs:65` and `:69`).
  - A patched runner, because Boat runs the stock runner. It comes back if
    the user takes the runner record of option C of
    `plans/follow-ups/job-usage-step-name-expressions.md`.
- **Wider fix:** D. Like C, E, G, and H, it takes the address from GitHub,
  so a change in how GitHub numbers steps cannot break it, and the app
  never builds a step address from a row number or a run address. D stays
  the wider fix, because it needs no App setting and makes the fewest
  reads.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/job-usage-overlapping-steps.md`, the same code: B and
    F extend the step tracker of the guest that its options rewrite.
  - `plans/follow-ups/job-usage-step-name-expressions.md`, the same cause
    in part, and a shared read. Its option B matches GitHub steps by
    position, because the row number is not the GitHub number, so it needs
    a count check. If the user takes that option, C can share its read. Its
    option C adds a runner record with the step number, which must then
    also carry the job ID.
  - `plans/follow-ups/job-page-refresh.md`, options that conflict, for E
    only. E reads GitHub each time the API serves the job page, and the
    refresh of that item reads the page again until it is final. If the
    user picks E, its read must not run on each refresh.
  - `plans/follow-ups/job-usage-container-step-names.md`, the same rows:
    its option A keeps each unnamed container step behind this link.
- **Recommendation:** D. It makes the fewest GitHub reads, one for each
  opened link, and it needs no App setting, no event traffic, and no
  runner log detail. D is no longer the only exact option that keeps the
  reasons of both decisions of 23 November 2026. G keeps them too, and H
  keeps them except on the first view of each job. G makes no read, but it
  needs the App owner to change the App, and it adds traffic and a table.
  F makes no read, but it must know how the runner numbers post steps.
  - If the user takes the runner record of
    `plans/follow-ups/job-usage-step-name-expressions.md`, take the number
    and the job ID from that record instead.
  - Whatever the answer, correct `docs/protocol/job-usage.md:91-92`, the
    link words in the app and the mockup, the comment at
    `ts/app/src/view/jobs-usage.ts:140`, and
    `ts/app/guides/jobs/README.md:48`.
  - Add a test with a run address, and a test where the row number differs
    from the GitHub number. Update the browser check and the view test that
    pin the current link.
- **Action:** ask the user, because D adds a route and a GitHub read that
  no requirement covers, and C, D, and F trade GitHub reads against runner
  log details with no clearly best choice.
