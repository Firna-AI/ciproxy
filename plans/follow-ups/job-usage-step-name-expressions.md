# Step names can show unresolved expressions

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch
  `calummoore/action-cpu-and-memory-usage`, review round 2, finding 304.
- **Location:** `crates/ciproxy-guest/src/usage/worker_log/steps.rs:65`,
  the `Processing step` event in `Steps::observe`, and
  `docs/protocol/job-usage.md:79-82`.
- **Reason:** The recommended option adds a GitHub read that the Steps
  decision of `docs/protocol/job-usage.md` rejects, and the other options
  have real trade-offs.

- **Terms:**
  - **Worker log:** the log that the GitHub runner writes for each job in
    `_diag/Worker_*.log`. The guest, the ciproxy program in the job VM,
    reads the steps from it.
  - **Expression name:** a step name that uses an expression, such as
    `Publish ${{ steps.build.outputs.target }}`, which the runner resolves
    while the job runs.
  - **Trace level:** the level of the lines in the Worker log. The release
    runner writes `Info` lines, and `Verbose` lines only when
    `GITHUB_ACTIONS_RUNNER_TRACE` is set.
  - **Script names rule:** the rule that replaces a generated "Run" name
    with "Run script" unless the rest names a remote action, such as
    `actions/checkout@v4`.
- **Background:**
  - The images pin runner `v2.338.2` (`image/pins.env:11`). The paths that
    start with `src/` are in its repository `actions/runner` at that tag.
  - A step name can use the output of an earlier step, or a context such as
    `runner`. The runner writes the `Processing step: DisplayName=` line
    before it resolves such a name (`src/Runner.Worker/StepsRunner.cs:81`
    and `:215`).
  - The runner first evaluates each name when it builds the job, with only
    the contexts of the job message (`src/Runner.Worker/JobExtension.cs:345-354`).
    A name that needs `hashFiles`, or a context that the job message does
    not carry, such as `steps` or `runner`, fails that check
    (`src/Sdk/DTObjectTemplating/ObjectTemplating/Tokens/TemplateTokenExtensions.cs:116-147`).
    The runner then keeps the expression text as the name
    (`src/Runner.Worker/ActionRunner.cs:444`).
  - `StepsRunner.cs:215` resolves the name later, just before the
    condition, and updates the timeline record that GitHub shows. The
    `secrets` context comes only at that time
    (`src/Runner.Worker/ExecutionContext.cs:1009`). The runner masks known
    secrets in a fully evaluated name (`ActionRunner.cs:346`).
  - So the Worker log holds the expression of a main step name that uses a
    secret, not its value. Only a post step name can hold a resolved and
    masked secret. The runner builds `Post {DisplayName}` at the main
    stage, after the name resolves (`ActionRunner.cs:115`).
  - At the default `Info` level, no Worker log line holds the resolved name
    of a main step. It reaches only the `Record:` line at
    `src/Runner.Common/JobServerQueue.cs:861`, which is `Verbose`
    (`src/Runner.Common/TraceSetting.cs:13-21`). The guest passes only
    `ACTIONS_RUNNER_INPUT_JITCONFIG` and `CIPROXY_ASSIGNMENT_SOCKET` to the
    runner (`crates/ciproxy-guest/src/platform.rs:22-25` and `:128`).
  - The steps come from the Worker log in the VM, by the user's decision of
    23 November 2026, option F. The Steps decision at
    `docs/protocol/job-usage.md:79-82` rejects the jobs API of GitHub,
    because each job then costs a GitHub call, and the times come from
    another clock.
  - A row must show the name that GitHub shows, so that a reader can
    compare the row with GitHub. The planned end-to-end check of this
    branch compares the steps of the Worker log with the steps on GitHub.
    It has not run yet, because it needs real machines.
  - By the user's decision of 23 November 2026, option C, the control
    plane applies the script names rule before it stores a name. That
    choice relies on the masking of secrets in step names, so each option
    that stores a resolved name must check that masking on its own source.
- **Problem:** The guest keeps the early name, because a default Worker log
  does not name the step again. A trace replay reported
  `Publish ${{ steps.build.outputs.target }}`, but the completed record
  named the step `Publish linux`. That replay used a `Verbose` log, which
  a default runner does not write, so the guest gets only the early name.
- **Impact if no action:** The row does not match the step name on GitHub,
  or the name of its own post row. Readers must work out which step the
  figures describe.
- **Root question:** Which name should a row show for a step whose name the
  stock runner resolves only after it logs the step? From which source can
  ciproxy get that name, with the VM times kept?
- **Options:**
  - **A.** Read the final names from the completed records of the Worker
    log when they match one step, and keep the early name when no record
    matches. It cannot work as written: the record that it reads is the
    `Verbose` `Record:` line, which a default runner log does not have.
    Effort: Medium.
  - **B.** Read the final names from the GitHub jobs API, only for a job
    whose logged name contains `${{`. Keep the VM times and the script
    names rule.
    - Match by position with a count check and the literal text around
      each `${{ }}`, not by the row number alone. A row carries its
      position, not its GitHub number, and the two differ after the first
      post step.
    - First make the timing read obey the throttle replies of GitHub, as
      `plans/follow-ups/timing-repair-throttle.md` asks.
    - Carry the source of each step from the guest, so that a resolved
      "Run" name of a script step still becomes "Run script". Before its
      merge, this branch adds a guest check that keeps a "Run" name only
      when its rest matches a remote action of the job. That check sees
      only the logged names.
    - Cost: one GitHub read for each job with such a name, which the Steps
      decision rejects.
    - Effort: Large.
  - **C.** Write one structured record from the runner code that reports
    the step to GitHub, with the final name, the step number, and the
    times, on every image. Cost: a runner patch, also on Boat, which runs
    the stock runner. The Steps decision rejects step events from the
    runner patch. Effort: Large.
  - **D.** Make the record of A exist.
    - Start the runner with `GITHUB_ACTIONS_RUNNER_TRACE=1` on Linux and
      macOS, and read the `Record:` lines. Give a step the name of the
      `t=Task` record that matches it by start time and order, and keep the
      early name when no record matches. Add a fixture, and a step with an
      expression name in `image/worker-steps-smoke/workflow.yml`.
    - Cost, log size: the Worker log then also holds each line of job
      output (`JobServerQueue.cs:244`). The own-machine guest reads the
      whole log once, at runner exit
      (`crates/ciproxy-guest/src/usage/mod.rs:196`). Boat reads it every
      30 seconds.
    - Cost, lost steps: each read has a budget of 64 MiB and a line limit
      of 1 MiB (`crates/ciproxy-guest/src/usage/worker_log/mod.rs:16-17`).
      A read error drops every step of the job
      (`crates/ciproxy-guest/src/usage/mod.rs:222-231`). So a job with much
      output loses all its steps unless those limits change, and they guard
      against job code that writes the log.
    - Cost, format: the record line has no record ID. Its name ends at
      `, s=`, which a step name can also hold. Its `st=` time uses the
      culture date format of the job. A `Verbose` line is a weaker contract
      than the `Info` lines that the runner pin check covers.
    - A match by start time is not unique for steps that start in the same
      second, which background and `parallel` steps make normal.
    - Effort: Large.
  - **E.** Keep the name that the runner logs, and document it.
    - The Step names section of `docs/protocol/job-usage.md`, the guide
      `ts/site/src/content/docs/docs/guides/choose-a-label-size.md`, and the
      end-to-end name check then say this: a name shows its expression when
      the expression needs `hashFiles` or a context that the job message
      does not carry, such as `steps` or `runner`.
    - Cost: such a row still differs from GitHub and from its own post row.
      The requirement that a row shows the GitHub name drops for these
      names, so E needs the user.
    - Effort: Small.
  - **F.** Store the logged name, and resolve a name that contains `${{`
    only when the API serves the page.
    - The API reads the steps of the job with the GitHub App sign-in of the
      person. It shows the GitHub name, and it never stores or logs it.
    - The person sees the logged name in three cases: no access to the
      repository, a failed read, or a step count that differs.
    - Cost: one GitHub read for each view of such a job, which the user
      rejected for all names on 23 November 2026, and a page state for each
      person.
    - Gain: no resolved runtime text reaches storage, or a person without
      access to the repository.
    - Effort: Large.
- **Left out:**
  - The timing reads that already run, because most jobs get no timing
    read, so they fail the goal.
  - The name of the post step, `Post <resolved name>`, because only an
    action with a post step has one, and a script step never does.
  - The debug line `Set step '…' display name to` (`ActionRunner.cs:349`),
    because `Debug` writes to the step log, not to the Worker log
    (`ExecutionContext.cs:1099-1120`). It also needs step debug, which adds
    debug lines to the GitHub log of the customer.
  - A runner patch on own machines only, because Boat jobs keep the
    defect, and C covers both.
  - An evaluation of the expression in the guest, because the guest has
    neither the step outputs nor the `env` and `runner` values of the step,
    and it would have to copy the expression engine of the runner.
- **Wider fix:** C. One runner record gives the final name and the GitHub
  times of each step, also of a step that fails before it runs. D also
  reads the records that the runner sends to GitHub, but it depends on a
  `Verbose` line with no record ID, and its log volume can drop every step.
- **Previous examples:** None found for unresolved step names.
- **Related:**
  - `plans/follow-ups/job-usage-step-links.md`, a shared GitHub read in
    part. Its options C, D, and E read the GitHub job, whose steps give
    both the GitHub number and the resolved name. If the user picks its C,
    B takes the names from that same read and matches by its verified
    numbers. If the user picks its E, F uses that same read.
  - `plans/follow-ups/job-usage-overlapping-steps.md`, the same code. Its
    option B tracks open steps by identity in the same `Steps` tracker. A
    background step also resolves its name at `StepsRunner.cs:215`, before
    `:248` queues it.
  - `plans/follow-ups/job-usage-container-step-names.md`, the same rule. B
    takes names from GitHub in the control plane, where the guest name
    check does not run, so B needs the source of each step from the guest.
    The step mark of option B of that item can carry that source.
  - `plans/follow-ups/job-usage-unstarted-step-times.md`, the same wider
    fix: the runner record of C would also give its steps their times.
- **Recommendation:** B. It is the only option that gives the exact GitHub
  name with the stock runner on every platform and keeps the VM times. It
  reads GitHub only for a job whose logged name contains `${{`, which the
  runner writes for an expression that it cannot evaluate yet. Choose F if
  no resolved runtime text may reach storage. C stays the wider fix.
- **Action:** ask the user, because B adds a GitHub read that the Steps
  decision at `docs/protocol/job-usage.md:79-82` rejects, and D, E, and F
  have real trade-offs.
