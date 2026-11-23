# CPU and memory of each job step

Status: Active. On branch `calummoore/action-cpu-and-memory-usage`, no pull
request yet. The user approved this plan on 23 November 2026, in the words
"impl the plan". The pull request that completes the work deletes this
plan, as the `plans` skill says.

No part of ciproxy measures the CPU or the memory that a job uses. A
customer cannot see which step of a job needs the CPU or the memory, or
which step ran out of memory. So the customer guesses the size of each
label. This change measures each job inside its VM, and the job page shows
the CPU and the memory of each step.

The guest program samples the CPU and the memory of its VM once per second,
from the start of the job to the exit of the runner. It sends the samples
with its runner exit report. On an own machine, the agent forwards them to
the control plane. On Boat, the end report of the sandbox carries them, and
the guest also sends them every 30 seconds while the job runs. The guest
also reads the name, the start, and the end of each step from the runner's
Worker log, and it sends them with the samples. The control plane stores
the samples and the steps with the job. The job page shows the peaks of
the job, a chart over time, and the figures of each step.

The change does not cover size hints, figures on the job list, figures
while a job runs, figures for each process, alerts, or an export to the
customer's own monitoring. A job that ran before the change has no figures.

Decisions 6 and 7 wait for the user. Each TODO that needs an answer ends
with "Waits for decision N". The other TODOs do not wait.

Contract owners:

- `docs/protocol/job-usage.md`, a new protocol doc that Milestone 2 writes.
- [Secrets and job data](../docs/protocol/security-data.md#secrets-and-job-data)
  and [Data retention](../docs/protocol/security-data.md#data-retention).
- [Key custody](../docs/protocol/security.md#key-custody), for the sandbox
  token.
- [What the dashboard shows](../docs/protocol/job-routing.md#what-the-dashboard-shows).
- [The exit report](../docs/protocol/overflow-sandbox-end.md#the-exit-report)
  and [The end of a job](../docs/protocol/overflow-sandbox-end.md#the-end-of-a-job),
  for the Boat reports and the hold.
- The [architecture doc](../docs/architecture.md) and its
  [data flow](../docs/architecture/data-flow.md).
- The [mockups](../docs/mockups/README.md) of the job page.
- The READMEs of `ciproxy-guest`, `ciproxy-protocol`, `ciproxy-agent`,
  `ciproxy-control-plane`, `ciproxy-overflow`, `ciproxy-api`,
  `ciproxy-schema`, `ciproxy-store-interface`, `ciproxy-store-pg`, `image`,
  and `ts/app`, and the image checks in `image/checks.md`.

## Decision 2. An out-of-memory kill ends the whole job

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
- **Source:** Review. The options check of decision 1 found it on
  22 November 2026. The options check of decisions 2 to 4 added D and
  changed the recommendation from A to B.
- **Category:** Product bug.
- **Location:** `image/ciproxy-guest.service`,
  `image/ciproxy-boat-guest.service`, and
  `crates/ciproxy-guest/src/platform.rs:84`.
- **Terms:**
  - **OOM policy:** the systemd setting `OOMPolicy=`. It says what happens
    to a service when the kernel stops one of its processes for lack of
    memory. The default is `stop`, which stops the whole service.
  - **Control group:** the Linux group of processes that systemd manages
    as one service.
  - **systemd-oomd:** a systemd service that stops a whole control group
    under memory pressure, before the kernel acts.
- **Background:** The guest program runs as a systemd service in each
  Linux job VM and each Boat sandbox. It starts the runner with `runuser`
  and without a login, so the runner and each step process stay in the
  control group of that service. Docker containers run in control groups
  of their own. Neither service file sets an OOM policy, so the default
  `stop` applies. A macOS job VM runs the guest under launchd, so this
  item does not apply to it.
- **Problem:** When the kernel stops one step process for lack of memory,
  systemd stops the whole guest service. That ends the runner, so the job
  ends at once. The guest has no stop handler, so it sends no exit report.
  This comes from the service files and the systemd defaults. Nobody
  reproduced it in a job VM.
- **Impact if no action:** A job that runs out of memory outside Docker
  loses its cleanup steps and its uploads. GitHub then most likely shows
  "The runner has received a shutdown signal" instead of the failed step.
  The figures of decision 1 miss exactly these jobs.
- **Root question:** What happens to a job, and to the guest's exit report,
  when the kernel stops one process of the job for lack of memory?
- **Assumptions:**
  - Only the kernel stops job processes for lack of memory. Source: the
    code. No file of the repository sets up `systemd-oomd`. Nobody checked
    Boat, which keeps its own services and swap. B does not need this.
  - A Linux job VM has no swap. Source: the code. Kept. The follow-up item
    `plans/follow-ups/job-vm-swap.md` records it.
- **Options:**
  - **A.** Set `OOMPolicy=continue` in both guest service files. The guest
    program also protects itself from the kernel's out-of-memory killer,
    and it gives the runner the normal priority. Only the killed process
    ends, and the guest still reports. Cost: when something stops the
    whole control group, such as `systemd-oomd`, the guest still ends.
    Effort: Medium.
  - **B.** Start the runner in a systemd scope of its own, with
    `OOMPolicy=continue`, apart from the guest service. The guest keeps
    the protection of A. Cost: the chroot check of the image needs a
    stand-in for `systemd-run`, and the follow-up item
    `plans/follow-ups/boat-runner-stderr-drain.md` must change, because
    its helper leaves the control group of the guest. Effort: Medium.
  - **C.** Keep the current behaviour. Cost: the job page shows no figures
    for the jobs that ran out of memory outside Docker. Effort: Small.
  - **D.** Keep the `stop` policy, and make the guest report on a stop. The
    guest handles the stop signal, waits a few seconds for the runner to
    end, and sends its exit report with the samples. Cost: the job still
    ends at once, as today. A kill of the whole group still loses the
    report. Effort: Medium.
- **Left out:** `OOMPolicy=kill`, because it ends every process of the
  service, also the guest. A memory limit for the runner, because alone it
  still stops the service, and with A or B it only picks the process that
  the kernel stops, which the protection of the guest already does.
- **Wider fix:** B, because it moves the job out of the control group of
  the guest. The guest then outlives any action on the job's group.
- **Previous examples:** None found.
- **Recommendation:** B. The first recommendation was A. Public reports
  show that a GitHub-hosted runner also ends such a job, so A gains no
  likeness to GitHub. B also keeps the guest alive when something stops
  the whole job group. A is enough if Milestone 4 finds no `systemd-oomd`
  on Boat. Choose D for the job behaviour of today with figures.
- **Action:** The user chose B on 23 November 2026, in the words "2 = B".

## Decision 1. Customers cannot see step CPU and memory

- **Severity:** Medium. **Effort:** Large.
- **Source:** Design. The options check of 22 November 2026 added E and F.
- **Category:** Docs or spec.
- **Location:** `crates/ciproxy-store-pg/src/github/timing.rs:61` and
  `crates/ciproxy-github/src/listener/messages.rs:78`.
- **Terms:**
  - **Guest program:** `ciproxy-guest`, the ciproxy program inside each job
    VM and each Boat sandbox. It starts the runner and reports its exit.
  - **Timing read:** the read of a job from the GitHub Actions jobs API
    that gives the control plane the real start and end of the job.
  - **Scale-set message:** the message that GitHub sends the control plane
    when a job is assigned, starts, or ends.
  - **Worker log:** the runner's own log file in the VM. It names each step
    with its start and end.
  - **Job summary:** the Markdown panel that GitHub shows on the page of a
    job run.
- **Background:** The guest program runs in every job VM and every Boat
  sandbox, so one sampler there covers Linux machines, Macs, and Boat. The
  reply of the timing read lists each step with its name, start, and end.
  But the timing read runs only when the scale-set messages gave no start
  or no end, and they usually give both.
- **Problem:** No part of ciproxy measures the CPU or the memory that a job
  uses. A customer cannot see which step needs the CPU or the memory, or
  which step ran out of memory.
- **Impact if no action:** Customers guess label sizes and cannot find the
  step that makes a job slow or fail. A guess that is too large sends more
  jobs to paid Boat sandboxes.
- **Root question:** How does a customer learn how much CPU and memory each
  step of a job used, including the step that ran out of memory?
- **Assumptions:**
  - Each figure belongs to one step of one job. Source: the user's question
    of 22 November 2026. Kept.
  - The figures come from inside the VM. Source: the code. The host
    measures nothing, and Boat's API gives only the cost of a sandbox.
    Kept.
  - A step's figures cover the whole VM while the step runs. Source: this
    plan. Kept, because the label limits the whole VM.
  - No figure names a process or a command line. Source:
    [Secrets and job data](../docs/protocol/security-data.md#secrets-and-job-data).
    Kept. Decision 3 covers the step names.
  - The figures change no VM size. Source:
    [Labels](../docs/protocol/labels.md). Kept.
  - The program that reports the figures lives until the job ends. Source:
    the code. It fails on Linux when a step outside Docker runs out of
    memory. Decision 2 covers it.
  - The timing read already reads each job. Source: the text of D. It is
    wrong, as the background says. E and F drop it.
- **Options:**
  - **A.** Add a docs guide. It tells customers to add
    `step-security/workflow-telemetry-action` as the first step of each
    job. The action shows a step timeline beside CPU and memory charts in
    the job summary. Cost: each workflow needs an edit, and a third-party
    action runs in each job. Effort: Small.
  - **B.** Add a sampler to the guest program and a job-completed hook to
    the images. The hook writes a table of the steps with their CPU and
    memory into the job summary. It reads the step times from the Worker
    log. Cost: the Worker log is not a runner contract. Each GitHub job log
    gets a new "Complete runner" step, and the dashboard shows nothing.
    Effort: Medium.
  - **D.** Add the same sampler, and send its samples to the control plane
    with the exit report. Take the step times from the timing read. The
    job page shows each step with its CPU and memory, and a chart over
    time. Cost: the timing read skips most jobs, so most jobs show no
    steps. Effort: Large.
  - **E.** Do D, but read the steps of each finished job from the GitHub
    jobs API. One read covers all jobs of a run that have ended. Cost: one
    more GitHub call for each finished job, against the rate limit of the
    installation. The timing read must first obey GitHub's throttle
    replies, as the follow-up item `timing-repair-throttle.md` asks.
    Effort: Large.
  - **F.** Do D, but the guest reads the name, the start, and the end of
    each step from the Worker log. It sends them with the samples, so the
    control plane makes no GitHub call. Cost: each runner update needs an
    image check of the log format, and job code can change the step names.
    Effort: Large.
- **Left out:**
  - **C.** One summary for the whole job on the job page. It shows no
    steps. D, E, and F include it, because their job page also shows the
    peaks of the whole job.
  - Host figures from each Linux job VM. Macs and Boat get no figures.
  - Step events from ciproxy's patched runner. Boat runs the stock runner,
    so Boat jobs would get no steps.
  - Figures for each process in the VM. A process name or a command line
    can hold a secret.
  - A ciproxy action that a customer puts around chosen steps. It needs
    workflow edits, and E and F cover every step without them.
  - Machine-level use. It shows no job or step.
  - Automatic size changes. They change the label rules, and job code can
    fake the figures.
  - An export to the customer's own monitoring. It reaches few customers.
    An export from the VM would also put its credential where job code can
    read it.
- **Wider fix:** None. This is a new feature, not a class of defect.
- **Previous examples:** None in the plans, the follow-up items, or the
  pull requests. The workspace "CPU Sharing for Self-Hosted" has an open
  decision about sharing CPU between jobs.
- **Recommendation:** E. The first recommendation was D. E reads the steps
  from GitHub's documented jobs API, a safer source than the Worker log of
  F.
- **Action:** The user chose F on 23 November 2026, in the words "F", after
  a more detailed comparison of D, E, and F.

History: the first round of the brainstorm asked about each job as a whole
and recommended C. The user then asked for each step, so the second round
left C out and recommended D. The user chose D on 22 November 2026, in the
words "1 = D". The options check of the same day found that the timing
read skips most jobs, so this decision was open again. The user then asked
whether E could be less exact than F because of clocks. F is more exact:
its steps and its samples come from the same clock of the VM, while the
steps of E come from GitHub. The research sources are
[CircleCI](https://circleci.com/docs/resource-class-overview/),
[WarpBuild](https://www.warpbuild.com/docs/ci/features/observability),
[Blacksmith](https://docs.blacksmith.sh/blacksmith-cli/jobs),
[workflow-telemetry-action](https://github.com/step-security/workflow-telemetry-action),
and the
[GitHub jobs API](https://docs.github.com/en/rest/actions/workflow-jobs#list-jobs-for-a-workflow-run).

## Decision 3. Step names can hold a line of job code

- **Severity:** Medium. **Effort:** Small.
- **Source:** Review. The options check of decision 1 found it on
  22 November 2026. The options check of decisions 2 to 4 added E.
- **Category:** Security.
- **Location:** `docs/protocol/security-data.md:10` and
  `ts/site/src/content/docs/docs/security/job-isolation.md:86`.
- **Background:** The job page of decision 1 shows each step by its name.
  If a step has no `name:`, GitHub names it with "Run" and the text of its
  `run` command, and a step name can also use the `secrets` context. Both
  docs say that the control plane never receives, stores, or logs the code
  of a job. But the timing read already receives the jobs reply of GitHub,
  which names each step, and drops the steps. A person who can see the
  jobs sees the jobs of all repositories of the workspace, also without
  access to a repository on GitHub.
- **Problem:** A stored step name can hold a line of job code, and that
  line can hold a secret. The word "receives" in the rule is already not
  true.
- **Impact if no action:** The control plane stores job code against its
  security rule. People without access to the repository can read it.
- **Root question:** How does the job page let a reader tell its steps
  apart, without ciproxy keeping job code or showing it to a person who
  cannot read the repository?
- **Assumptions:**
  - The control plane may receive step names in a GitHub reply when it
    drops them at once. Source: the code. Each option keeps this, so each
    option changes "never receives, stores, or logs" to "never stores or
    logs" in both docs.
  - A person with a role sees every job of the workspace. Source: the
    user's decision under "Roles" in the security doc. Kept.
- **Options:**
  - **A.** Store each name as GitHub gives it. The security doc then says
    that a step name can hold a line of job code. Cost: that line reaches
    the control plane and each person who sees the jobs. Effort: Small.
  - **B.** Store no names. The job page shows each step by its number, with
    a link to that step on GitHub. Cost: the reader opens GitHub to see
    what a step does. Effort: Small.
  - **C.** Store each name, except a name that GitHub can make from a
    `run` command: a name that starts with "Run" and is not
    "Run" with a remote action, such as `actions/checkout@v4`. Such a step
    shows as "Run script" with its number and a link to GitHub. Cost: a
    name that the author chose and that starts with "Run", and the name of
    a local action such as "Run ./build", also show as "Run script".
    Before C ships, a test must show that GitHub masks a secret in a step
    name. Effort: Small.
  - **D.** The browser reads the names from GitHub with a GitHub token of
    the person, so the names never reach the control plane. Cost: a GitHub
    token in the browser, and one more GitHub read for each page view.
    Effort: Large.
  - **E.** The API reads the names from GitHub when it serves the job page,
    with the person's own GitHub App sign-in, which ciproxy already keeps
    for the membership check. GitHub gives names only for repositories that
    the person can read. The API never stores or logs them. A person
    without access, or a failed read, gets the numbers and links of B.
    Cost: one GitHub read for each page view, and a page state for each
    person. Effort: Medium.
- **Left out:** Masking the secrets in a name, because the control plane
  does not know the secrets of a job, and a line of code stays code.
- **Wider fix:** None. The rule in force is the wider fix.
- **Previous examples:** None found.
- **Related:** Decision 1, work that waits for the answer. The step table
  of E and F needs this rule.
- **Recommendation:** C. It keeps the stores-or-logs rule, and it keeps the
  names of named steps and of remote action steps. Choose E if readers
  need the names of script steps.
- **Action:** The user chose C on 23 November 2026, in the words "3 = C".

## Decision 4. Boat can delete a sandbox before its report

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Review. The options check of decision 1 found it on
  22 November 2026. The options check of decisions 2 to 4 added D and E.
- **Category:** Docs or spec.
- **Location:** `crates/ciproxy-control-plane/src/boat_authority.rs:94`,
  `crates/ciproxy-control-plane/src/boat_jobs.rs:45`, and
  `crates/ciproxy-store-pg/src/overflow/completion.rs:28`.
- **Terms:**
  - **End report:** the message in which the guest program of a Boat
    sandbox tells the control plane that its runner exited.
- **Background:** The guest program sends the end report after its runner
  exits. When GitHub reports that a job completed, the Boat cleanup
  removes the runner of that job and deletes its sandbox at once. It does
  not wait for the end report. When this decision was made, an end report
  that waited for the overflow lock was refused after the cleanup ended
  the record of the sandbox. Since the claim-first change of main on
  23 November 2026, a report that arrives first is saved without the lock.
  The samples of decision 1 travel in the end report.
- **Problem:** The cleanup can delete the sandbox before the guest sends
  its end report, or refuse a report that arrives during the cleanup. The
  samples of that job are then lost.
- **Impact if no action:** Many Boat jobs show no figures, because the
  cleanup and the end report race at the end of each job.
- **Root question:** How does the control plane get the samples of each
  Boat job, whatever ends its sandbox or refuses its report first?
- **Assumptions:**
  - The guest sends the samples, and the control plane never fetches them.
    Source: the "Transport" design call. D drops it.
  - A lost report can lose the figures. Source: the "Lost reports" design
    call. Kept, because the figures never decide.
- **Options:**
  - **A.** Accept the loss. The job page of such a Boat job says that it
    has no figures. Effort: Small.
  - **B.** The retirement check of the cleanup skips a sandbox while its
    job ended less than 15 seconds ago and no end report has arrived. It
    does not wait inside the cleanup, which holds the lock. The overflow
    doc then says so. Cost: the cleanup runs every 10 seconds, so a late
    or lost report keeps the sandbox up to about 25 seconds more, and Boat
    bills that time. Effort: Medium.
  - **C.** The guest program also sends its samples every 30 seconds while
    the job runs. Cost: a race still loses the last 30 seconds, and the
    sandbox token, which the store now accepts once, must accept several
    reports. Effort: Large.
  - **D.** Read the samples before each deletion. The guest keeps its
    latest samples in a root-only file. Before it deletes a sandbox, the
    overflow service reads that file through Boat's controller, with a
    time and a size limit. Cost: one more Boat call and more lock time for
    each deletion. Effort: Large.
  - **E.** The job-completed hook of the runner asks the guest to send its
    samples before the runner reports the result to GitHub. Cost: a
    "Complete runner" step in each Boat job log, a second report for the
    sandbox token, and job code can remove the hook. Effort: Large.
- **Left out:** Sending the samples when the worker process of the job
  exits, because GitHub learns the result first. Saving the usage of a
  report before it takes the lock, because it does not help when the
  sandbox is deleted before the guest sends.
- **Wider fix:** D, because every deletion passes the read, also a cleanup
  after a failed or refused report.
- **Previous examples:**
  - `plans/follow-ups/boat-runner-stderr-drain.md` records other
    end-of-job data that the deletion of a sandbox loses.
  - `plans/follow-ups/boat-live-completion-acceptance.md` records the same
    race between the job-end cleanup and the exit report. On 23 November
    2026, main moved the acceptance of the exit report out of the cleanup
    lock, in `crates/ciproxy-overflow/src/service/cleanup.rs`. So a report
    that arrives during a cleanup is no longer refused for the lock, but
    the deletion can still come first.
- **Related:** Decision 1, work that waits for the answer. Only the Boat
  part of the samples waits.
- **Recommendation:** B. It keeps the figures of each Boat job, and Boat
  bills more only when a report is late. Choose D for figures also when a
  report is lost or refused.
- **Action:** The user chose B and C together on 23 November 2026, in the
  words "4 = B + C".

## Decision 5. Who reads the CPU and memory figures

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design. The user wrote on 23 November 2026 that the figures
  are "only for internal use by the team".
- **Category:** Docs or spec.
- **Location:** Milestones 1, 2, 11, and 12 of this plan.
- **Background:** The plan shows the figures on the job page, which every
  person with a role in the workspace sees. "The team" could mean the team
  of the customer or the team of ciproxy.
- **Problem:** A wrong reading builds a customer page, its mockups, and a
  docs page that nobody asked for.
- **Impact if no action:** The plan can build the wrong scope.
- **Root question:** Who reads the CPU and memory figures of each step?
- **Options:**
  - **A.** The customer's own team, on the job page. Effort: Small.
  - **B.** Only the ciproxy team, through a query or an internal report.
    Effort: Medium.
  - **C.** Both, with an internal view across all workspaces. Effort:
    Large.
- **Left out:** None.
- **Wider fix:** None. This is a scope question, not a defect.
- **Previous examples:** None found.
- **Recommendation:** A. It matches the job page of decision 1.
- **Action:** The user chose A on 23 November 2026, in the words "5 = A".
  The user also said that faked figures do not matter, because only the
  jobs of a customer can change the figures of that customer. The size and
  rate limits of the design stay, because they also protect the control
  plane from a broken guest.

## Decision 6. The hold reverses an earlier choice

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation. The orchestrator found it on 23 November
  2026, while it prepared Milestone 2. The options check of the same day
  added D and E, and it changed the reason of the recommendation.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/overflow-sandbox-end.md:46` and
  `crates/ciproxy-control-plane/src/boat_authority.rs:94`.
- **Terms:**
  - **Exit report:** the one-time message in which the guest program of a
    Boat sandbox tells the control plane that its runner exited. This plan
    adds the samples and the steps of the job to it.
  - **Job-end delete:** the delete of a Boat sandbox that starts when
    GitHub reports that its job completed.
  - **Claim-first:** the user's choice of 22 November 2026. The control
    plane saves an exit report before it waits for other overflow work.
- **Background:**
  - With claim-first, a report that arrives before the job-end delete is
    saved. The samples are lost only when the delete comes first.
  - The Decision line of "The exit report" in `overflow-sandbox-end.md`
    still rejects "a delay of the job-end delete".
  - That rejection had two reasons: a report could still wait behind other
    overflow work, and the delay bills each sandbox whose report never
    comes. Claim-first removed the first reason.
  - On 23 November 2026, the user chose B and C for decision 4. B is such a
    delay. B states the second reason as its cost.
- **Problem:** The protocol doc rejects the hold that decision 4 adds.
- **Impact if no action:** Milestone 2 cannot write the Boat rules, and
  Milestone 8 cannot build the hold.
- **Root question:** How does the control plane keep the final samples of
  a Boat job without breaking a reason of the claim-first choice that
  still holds?
- **Options:**
  - **A.** Keep B and C. The doc keeps the Decision line for claim-first,
    takes the delay out of its rejected list, and adds a Decision line for
    the hold. Cost: Boat bills up to about 25 seconds more, but only when a
    report is late or lost. Effort: Small.
  - **B.** Keep only C. The job-end delete and the doc stay as they are.
    Cost: when the delete comes first, a Boat job loses up to its last 30
    seconds of samples and steps, which often hold the step that failed.
    Effort: Small.
  - **C.** Keep only C, but send the samples every 10 seconds, with a rate
    limit of 5 seconds. Cost: about 100 KB every 10 seconds for a long job,
    and a Boat job still loses up to its last 10 seconds. Effort: Small.
  - **D.** Keep only C, and add the runner's job-completed hook on Boat.
    The hook asks the guest to send a sample report, and it waits a few
    seconds for the answer. The runner runs the hook after the last step,
    before it reports the result to GitHub. Cost: each Boat job log gets a
    "Complete runner" step, and each Boat job runs up to the hook limit
    longer. Effort: Medium.
  - **E.** Keep only C, and also send a sample report when the guest sees a
    step end in the Worker log, inside the rate limit. Cost: nothing makes
    sure that the report of the last step arrives first. Effort: Small.
- **Left out:** The hold without a doc change, because the protocol doc
  must state the behaviour. Saving a report that arrives after the recorded
  end, because it keeps only a report that left the VM during the delete.
  The exit report as the only trigger of the delete, because a lost report
  would then bill up to the lifetime of 6 hours.
- **Wider fix:** A, because it keeps each fact of the exit report, not only
  the samples.
- **Previous examples:** `plans/follow-ups/boat-live-completion-acceptance.md`
  and `plans/follow-ups/overflow-lifecycle-canonical-writes.md` record the
  same race between the cleanup and the exit report.
- **Related:** Decision 4, the same code. This decision settles whether B
  of decision 4 applies.
- **Recommendation:** A. The reason against a delay was a report that
  waits behind other overflow work, and claim-first removed it. So A
  narrows one rejected list and keeps claim-first. Choose D to keep the
  text of 22 November 2026 as it is.
- **Action:** ask the user, because A changes the rejected list of a
  decision in force that the user made on 22 November 2026.

## Decision 7. Partial Boat figures look complete

- **Severity:** Low. **Effort:** Medium.
- **Source:** Implementation. The docs work of Milestone 2 found it on
  23 November 2026. The options check of the same day added D, E, and F,
  and it moved the recommendation from A to D.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/job-usage.md`, "How the figures reach the
  control plane" and "Steps".
- **Background:**
  - On Boat, the guest sends sample reports every 30 seconds, and the final
    figures in the exit report.
  - When the exit report is lost or refused, the job page shows the newest
    sample report. It can miss up to the last 30 seconds of the job.
  - The step that runs at the end of such a series has a start and no end.
    A step without an end shows no figures today, so that step, often the
    one that failed, shows none.
- **Problem:** The page shows partial figures as if they covered the whole
  job, and it shows no figures for the step that ran at the end.
- **Impact if no action:** A reader can trust figures that miss the end of
  the job. A kill in the last seconds can show on no step row.
- **Root question:** How does a reader of the job page know which part of a
  job its figures cover, for the job and for each step?
- **Options:**
  - **A.** Show the figures with a note, such as "These figures end at
    6:31, before the job ended." The API marks figures that come from a
    sample report. Cost: a new mockup state, an API field, and a UI note.
    Effort: Medium.
  - **B.** Show no figures, as for a job without figures. Cost: the reader
    loses the figures that did arrive. Effort: Small.
  - **C.** Show the figures as they are. Cost: partial figures look
    complete. Effort: Small.
  - **D.** Do A, and also: only an accepted exit report marks figures as
    final. A last step with a start and no end runs to the end of the
    samples and shows its figures so far, with no "Complete job" row after
    it. Cost: a store mark, a step rule with its tests, and one more state
    of a step row. Effort: Medium.
  - **E.** Do B, and drop the sample reports of decision 4. Cost: it
    reverses the user's choice of C in decision 4, and each Boat job whose
    delete comes first shows no figures. Effort: Small.
  - **F.** Do D, and give the length of the missing end, such as "These
    figures miss the last 25 seconds of the job". The control plane
    measures it with its own clock, and the note shows only past a few
    seconds. Cost: a threshold for the note. Effort: Medium.
- **Left out:** A note on every Boat job, because the reader still cannot
  tell which job misses its end.
- **Wider fix:** D, because its final mark is a correct default for each
  series that no exit report finished.
- **Previous examples:** None found.
- **Related:** Decision 6. With its option A, this case becomes rare. With
  its option D or E, the gap is a second or two, and F fits better.
- **Recommendation:** D. The first recommendation was A. D keeps the
  partial figures that the sample reports exist to save, and it gives the
  step that ran at the end its figures.
- **Action:** ask the user, because D adds a state to the job page that no
  requirement covers.

## Design

Design calls, my calls, 22 November 2026:

- **Whole VM:** a step's figures cover all of the VM while the step runs.
  That includes service containers and processes that earlier steps
  started. Reason: the label limits the whole VM, and a process name or a
  command line can hold a secret.
- **Window:** on an own machine, the samples start at the job handoff of
  the patched runner. On Boat, the stock runner has no handoff, so they
  start when the runner starts the worker process of its job. They end
  when the runner exits. Reason: a warm runner can wait for hours, and
  that wait says nothing about the job.
- **CPU:** a sample gives the busy vCPU time of the last second, in
  thousandths of a vCPU. Busy time is user, nice, system, and interrupt
  time. Idle, I/O wait, and steal time are not busy.
- **Memory:** on Linux, a sample gives the total memory minus the
  available memory. On macOS, it gives app, wired, and compressed memory,
  as Activity Monitor counts them. Reason: the file cache fills all free
  memory, so it does not show the size that a job needs.
- **Out of memory:** on Linux, each point counts the processes that the
  kernel stopped for lack of memory. macOS gives no such count.
- **Points:** the samples hold at most 3,600 points. When they would pass
  that, each two neighbouring points merge, and the length of a point
  doubles. A point keeps the average CPU, the peak memory, and the sum of
  the out-of-memory kills. Reason: a 6-hour job then stays under about
  100 KB, inside the 1 MiB line limit of the agent's guest reader.
- **Transport:** the existing runner exit messages get an optional `usage`
  field: guest to agent, agent to control plane, and the end report of a
  Boat sandbox. Reason: no new message and no version change. Older guests
  send no field, and older readers ignore it. On Boat, decision 4 adds one
  new message, the sample report.
- **Control plane queue:** the queue of agent events holds only small
  events of a fixed size. The usage takes the byte-bounded worker payloads
  beside the queue, or a store write before the queue.
- **Lost reports:** a lost exit report loses the figures, and the job page
  says that the job has no figures. No queue retries them. Reason: the
  figures inform and never decide. On Boat, the sample reports of decision
  4 keep all but the last 30 seconds.
- **Checks:** the control plane drops a `usage` value with more than 3,600
  points, arrays of different lengths, CPU above the vCPUs of the VM that
  ran the job, or memory above its memory. On Boat, these limits come
  from the Boat size that ran the job, not from the label. It also drops a
  value whose times lie more than 5 minutes outside the runner's life,
  because the clocks of the VM and the control plane can differ. It keeps
  the exit, and its log holds no figure. Reason: job code has passwordless
  sudo in its VM, so a job can send any figures.
- **Trust:** nothing bills, routes, or sizes jobs by the figures. Reason:
  the same as for the checks.
- **Step figures:** a point belongs to the step that holds the middle of
  the point. A step shows its average CPU, its peak memory, and its
  out-of-memory kills. A step that holds no point shows no figures. A step
  without a start or an end gets no figures. Reason: the Worker log gives
  step times in whole seconds, and each point then counts once.
- **Where to compute:** the API computes the step figures when it serves
  the job page. Reason: the store keeps the raw points, so a later change
  to the rule needs no migration.
- **Chart size:** the API sends at most 600 chart points, merged as the
  guest merges them. Reason: the page stays small on a phone.
- **Clock:** the samples and the steps carry UTC times from the same clock
  of the VM. Reason: a VM clock that is off moves both by the same amount,
  so the steps and the samples still line up. Milestone 13 compares the
  steps with the steps on GitHub, to prove the Worker log reader.
- **Units on the wire:** the agent protocol gives memory in MiB. The
  dashboard API gives bytes as decimal strings and times in RFC 3339, as
  `AGENTS.md` says.
- **Job page only:** the job list stays as it is. Reason: the job page
  answers the question, and a list column can follow.
- **Running jobs:** the panel says that the figures come when the job ends.
  Reason: own machines send the samples once, with the exit report, and the
  Boat sample reports hold only part of a job.
- **Older jobs:** a job without figures, for example from an older agent
  or image, shows one plain sentence instead of the panel content.
- **Tiers and retention:** every tier gets the figures. They follow the job
  metadata: ciproxy keeps them 90 days, and each tier shows its job
  history window. Reason: every tier includes the dashboard.
- **Open-source boundary:** the guest, protocol, and agent changes stay in
  their Apache-2.0 crates and use no hosted crate. The store, control
  plane, API, and app changes stay in closed crates.
- **Doc split:** none. On 23 November 2026, main split `security.md` and
  `overflow.md`. "Secrets and job data" now lives in `security-data.md`,
  and the Boat exit report in `overflow-sandbox-end.md`. Each doc that
  this plan touches is under 250 lines.
- **Brainstorm:** the widen and narrow steps ran in the orchestrator
  session. That session runs the agent, the model, and the effort of the
  `brainstorming` action.

Design calls, my calls, 23 November 2026, for the answers:

- **Runner scope:** the guest starts the runner through `systemd-run
  --scope` with `OOMPolicy=continue`. The runner stays a child process of
  the guest, but it leaves the control group of the guest service.
  Reason: the guest keeps its supervision and the exit code of the runner.
- **Guest protection:** the guest gives itself the lowest out-of-memory
  score, and the runner scope gets the normal score. Reason: the kernel
  then stops a job process, not the guest.
- **Out-of-memory rule home:** `docs/protocol/job-usage.md` states what a
  job does when the kernel stops one of its processes. Reason: the rule
  and the out-of-memory figure belong together, and `agent.md` already
  passes 250 lines.
- **Script names:** a step name that starts with "Run", "Pre Run", or
  "Post Run" keeps its text only when the rest is a remote action, such as
  `actions/checkout@v4` or `docker://alpine:3.20`. Each other such name
  becomes "Run script". Reason: only a remote action reference is safe to
  keep, as decision 3 says.
- **Rule words:** "Secrets and job data" and the site page then say that
  the control plane never stores or logs the code of a job. Reason: the
  timing read already receives step names, as decision 3 says.
- **Hold:** the hold of decision 4 runs on the existing 10-second cleanup
  tick, with no timer of its own. Reason: up to about 25 seconds of hold
  costs less than a second timer path.
- **Sample reports:** on Boat, the guest posts its samples every 30 seconds
  to the existing callback URL, with the sandbox token. Each report holds
  the whole series so far. The end report also holds the final series.
  Reason: a report then never depends on an earlier one.
- **Report limits:** the control plane accepts sample reports only while
  the sandbox record is live, at most one each 10 seconds for a sandbox.
  It keeps only the newest series. The end report stays one-time. Reason:
  job code can read the token from the memory of the guest.
- **Report traffic:** a long job sends up to about 100 KB every 30 seconds.
  Reason: whole-series reports are simpler than deltas, because the
  points merge as the series grows.

Design calls, my calls, 23 November 2026, for the answer to decision 1:

- **Worker log steps:** the guest reads four lines of the runner's
  `StepsRunner` in the Worker log. "Processing step: DisplayName=" gives
  the name, "Starting the step." the start, and "Step result:" the end.
  "Skipping step due to condition evaluation." marks a skipped step.
  Reason: the pinned runner writes these lines for each step.
- **Set up and complete:** the time before the first step shows as "Set up
  job", and the time after the last step as "Complete job". Reason: the
  job page of GitHub shows the same two steps.
- **Step limits:** the guest sends at most 1,000 steps, with at most 256
  characters for each name. The control plane drops the steps of a report
  that passes a limit, and keeps its samples. Reason: job code can write
  the Worker log.
- **Name rule place:** the control plane applies the script names design
  call before it stores a name. Reason: the names come from the VM, which
  job code controls.
- **Log format check:** a unit test reads a Worker log fixture of the
  pinned runner. An image check runs a job and compares the steps that the
  guest reads with the steps of the workflow. Each update of the runner pin
  reruns both. Reason: the log format is not a contract of the runner.

Design calls, my calls, 23 November 2026, from the mockups of Milestone 1:

- **Label mark:** a job whose VM is larger than the size of its label, as
  on Boat, shows the label size as a reference mark on both plots and both
  meters. Reason: a reader sizes the label from these figures.
- **Run prefix:** the script names design call applies only to a name that
  starts with "Run ", "Pre Run ", or "Post Run ", with the space. Reason:
  a name such as "Running tests" is a name that the author chose.
- **Display units:** measured figures use SI GB and vCPU with one decimal.
  VM sizes and label sizes keep the format of the Details panel, such as
  "2 vCPU · 8 GB". Reason: one value has one format on a page.
- **Meter top:** the top of the memory meter is the memory of the VM. The
  kernel of the VM keeps a part of it, so a job can run out of memory below
  the top. The new protocol doc says so.
- **macOS memory:** macOS counts app, wired, and compressed memory, so its
  figures do not compare with Linux figures. The new protocol doc says so.
- **Running state:** the panel keeps its state for a running job, because a
  direct link can open the page of a running job. No new link leads to it.
- **Whole VM note:** the panel says that each figure covers the whole VM,
  also the services and processes that earlier steps started.
- **App chart:** `@firna/ui/chart` cannot mark steps, and it draws at a
  fixed width in a static render. So Milestone 11 builds the chart in the
  app as the mockup draws it, with token colours. Previous example of a
  chart that drifted from its mockup:
  `plans/follow-ups/savings-chart-firna-marks.md`.

Design calls, my calls, 23 November 2026, from the docs of Milestone 2:

- **Pre and post script names:** a replaced "Pre Run" or "Post Run" name
  becomes "Pre Run script" or "Post Run script". Reason: the mockups show
  them so, and the fixed prefix holds no job code.
- **Sample report clock:** for the time check of a sample report, the life
  of the runner ends at the arrival of the report. Reason: the runner of a
  sample report still runs.
- **Guest step limits:** the guest cuts a name to 256 characters and keeps
  the first 1,000 steps. The control plane still drops the steps of a
  report that passes a limit. Reason: the guest then sends a valid report
  for nearly every job.
- **Architecture words:** `docs/architecture.md` says that the control
  plane never stores or logs the code, the secrets, the logs, and the
  artifacts of a job. Reason: a step name from the VM can reach the
  control plane before the name rule replaces it.

Design calls, my calls, 23 November 2026, from milestone 2a:

- **Privacy mockup:** the privacy mockup shows placeholder lines for its
  sections, so it gets the new title and a new retention row, but no
  paragraph about stored data. Milestone 12 writes the script names rule
  into the privacy draft. Reason: the user writes and approves the legal
  text, and the mockup holds only its frame.
- **Boat key in the privacy draft:** the stored Boat key moves out of the
  section "What ciproxy never stores or logs". Reason: a stored fact under
  that heading contradicts it.

Design calls, my calls, 23 November 2026, from the options check of
decision 7:

- **Final series stays:** no sample report replaces the samples and steps
  of an accepted exit report, also when it arrives after that report.
  Reason: the exit report holds the final series.
- **Failed exit samples:** when the samples of an exit report fail a check,
  the control plane keeps the samples of the newest sample report. Reason:
  those samples passed the checks.

Contract sketch, for the protocol crate:

- Times in the agent protocol are whole milliseconds since the Unix epoch,
  in UTC. Reason, my call of 23 November 2026: the open protocol crate has
  no time dependency, and the dashboard API still gives RFC 3339 times.
- `JobUsage`: the platform, the UTC start of the first point, the length
  of a point in seconds, and three arrays of the same length. The arrays
  hold the CPU in thousandths of a vCPU, the peak memory in MiB, and the
  out-of-memory kills. It also holds the steps, each with its name, its
  UTC start, and its UTC end, or a mark that the runner skipped it.
- `GuestToAgent::RunnerExited` and `AgentMessage::RunnerExited` get
  `usage: Option<JobUsage>`, which serializes only when present.
- `GuestToAgent::UsageSamples` holds the runner and its `JobUsage`. Only a
  Boat guest sends it, to the callback URL. A guest in `vsock` mode never
  sends it, so older agents never see it.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The job page gets a CPU and memory panel. The job page already renders
five screens, so a new sub-page holds the other states of the panel.

- [x] Add a CPU and memory panel to the job page screen, in a new component
      beside `docs/mockups/src/screens/jobs/job-detail.tsx`. It shows the
      peak memory and the average CPU of the job, a chart over time with
      the steps marked, and a table of the steps with their figures. Check
      `@firna/ui` and its chart family first.
- [x] Add usage fixtures to the own-machine and Boat states of the job
      page. Include a step that holds no point.
- [x] Name the steps of the fixtures with the script names design call.
      Include a script step that shows as "Run script".
- [x] Add a sub-page for the panel under
      `docs/mockups/src/entries/app/jobs/`, with at most five states: a
      step that ran out of memory, a macOS job, a running job, a job
      without figures, and a job without step times.
- [x] Link the job routing and security docs, and the new job usage doc,
      from the new pages.
- [x] Run the mockup check named in `AGENTS.md`. Smoke-test the changed
      pages in the mobile and the desktop variants.
- [x] Update the design and the TODOs of this plan for each problem that
      the mockups show.

Evidence: the screenshots are in `.context/job-step-cpu-and-memory/mockups/`.

## Milestone 2: Documentation and protocol

Depends on: 1

The protocol docs state the requirements of the figures before the code.

- [x] Write `docs/protocol/job-usage.md`: the terms, the figures, the
      window, the steps, what the job page shows, the platforms, the trust
      rule, retention, and decision 1 with its rejected options. Say that
      the steps come from the runner in the VM, and that job code can
      change them. State the meter top, the display units, and the macOS
      memory design calls.
- [x] Add the CPU and memory samples and the steps of each job to the job
      metadata in "Secrets and job data" of `docs/protocol/security-data.md`,
      with the script names and rule words design calls. Link the new doc.
- [x] Say in "Key custody" of `docs/protocol/security.md` that the sandbox
      token also sends the samples of its own job.
- [x] Write the out-of-memory rule of decision 2 into the new doc: only
      the stopped process ends, and the job goes on.
- [x] Write the sample reports of decision 4 into "The exit report" of
      `docs/protocol/overflow-sandbox-end.md`. Align the token sentence of
      `docs/protocol/overflow.md`.
- [ ] Write the answer to decision 6 into "The end of a job" and into the
      Decision line of "The exit report" of
      `docs/protocol/overflow-sandbox-end.md`. Waits for decision 6.
- [x] Link the new doc from "What the dashboard shows" in
      `docs/protocol/job-routing.md`.
- [x] Add the samples to the boundary "The agent and its job VMs" in
      `docs/architecture.md`, and keep that doc under 250 lines. Add them
      to the job flows in `docs/architecture/data-flow.md`.
- [x] Add the new docs to the protocol doc list in `README.md`.
- [x] Run the Markdown check named in `AGENTS.md`.

## Milestone 2a: Privacy mockup and partial figures

Tags: mockup

Depends on: 2

The docs of Milestone 2 found two mockup gaps.

- [x] Change the title "What ciproxy never receives" in
      `docs/mockups/src/fixtures/legal.ts` to the rule words, and add the
      samples and the steps to the data that the privacy mockup stores.
- [ ] Add the state of figures that end before the job ended, if decision 7
      chooses A, D, or F. The `cpu-memory` sub-page already has five
      screens, so add a nested sub-page for it. Waits for decision 7.
- [ ] Run the mockup check named in `AGENTS.md`. Smoke-test the changed
      pages in the mobile and the desktop variants.

## Milestone 3: Usage in the runner exit messages

Depends on: 2

The protocol carries the samples, and the agent forwards them. The control
plane accepts the field and ignores it until Milestone 8.

- [x] Add `JobUsage` to `ciproxy-protocol`, with the limits of the design.
      Test each limit.
- [x] Add the optional `usage` field to `GuestToAgent::RunnerExited` and
      `AgentMessage::RunnerExited`. Test that a message without the field
      parses, and that the field round-trips.
- [x] Add `GuestToAgent::UsageSamples`, as the contract sketch says. Test
      that it round-trips.
- [x] Forward the usage of a guest exit report in
      `crates/ciproxy-agent/src/observations.rs`. An exit report that the
      agent makes itself carries no usage. Test both cases.
- [x] Update the READMEs of `ciproxy-protocol` and `ciproxy-agent`.

## Milestone 4: Jobs that run out of memory

Depends on: 2

A Linux guest keeps running when the kernel stops a step process for lack
of memory.

- [x] Start the runner in its own scope, as the runner scope design call
      says, in `crates/ciproxy-guest/src/platform.rs`.
- [x] Give the guest and the runner their out-of-memory scores, as the
      guest protection design call says.
- [x] Add a stand-in for `systemd-run` to the chroot check in
      `image/check-runner-environment.py`.
- [ ] Prove in a Linux job VM and in a Boat sandbox that a step that runs
      out of memory fails alone, and that the later steps and the exit
      report still run. Check whether Boat runs `systemd-oomd`.
- [x] Update `plans/follow-ups/boat-runner-stderr-drain.md`, because its
      stderr helper leaves the control group of the guest.
- [x] Update the READMEs of `ciproxy-guest` and `image`.

## Milestone 5: Guest sampler on Linux

Depends on: 3

The guest program measures Linux job VMs and Boat sandboxes.

- [ ] Add the points to `ciproxy-guest`, without platform code: one sample
      per second, and the merge at 3,600 points. Test the merge with a fake
      clock.
- [ ] Read the CPU from `/proc/stat`, the memory from `/proc/meminfo`, and
      the kills from `/proc/vmstat`. Test each reader with fixture files.
- [ ] Start the samples at the job handoff in `vsock` mode. In `boat` mode,
      start them when the stock runner starts the worker process of its
      job. End them when the runner exits.
- [ ] Send the samples with the exit report in `vsock` mode and in `boat`
      mode.
- [ ] In `boat` mode, also post the samples every 30 seconds, as the sample
      reports design call says. A failed post logs a warning and changes
      nothing else.
- [ ] Prove that a failed sample changes neither the runner nor its exit
      report. Log a warning, as the guest does for other failed
      observations.
- [ ] Update the `ciproxy-guest` README.

## Milestone 6: Guest sampler on macOS

Depends on: 5

The guest program measures macOS job VMs. A Mac is an own machine, so its
samples start at the job handoff.

- [ ] Read the CPU ticks and the app, wired, and compressed memory of
      macOS. Report no out-of-memory kills.
- [ ] Add `ciproxy-guest` to `MACOS_CRATES` in
      `xtask/src/check/phases.rs`, as the gate README says for a crate with
      macOS code. Test the conversions that need no macOS call on Linux.
- [ ] Update the `ciproxy-guest` README.

## Milestone 7: Store

Depends on: 3

The store keeps the samples and the steps of each job.

- [ ] Add a migration under `crates/ciproxy-schema/migrations` with one
      table for the samples of a job and one for its steps. Delete their
      rows with the job, also in the 90-day purge and in an App removal.
- [ ] Add the types to `ciproxy-store-interface` and the reads and writes
      to `ciproxy-store-pg`. Test them against PostgreSQL, as `AGENTS.md`
      says.
- [ ] Update the READMEs of `ciproxy-schema`, `ciproxy-store-interface`,
      and `ciproxy-store-pg`.

## Milestone 8: Save the samples

Depends on: 7

The control plane saves the samples from both exit reports and from the
Boat sample reports.

- [ ] Save the usage of `AgentMessage::RunnerExited` for the job of its
      runner. Pass it as the control plane queue design call says, not in
      the fixed-size event of
      `crates/ciproxy-control-plane/src/runtime_agent_event.rs`.
- [ ] Save the usage of the Boat end report that
      `crates/ciproxy-control-plane/src/boat_callback.rs` receives.
- [ ] Save the steps of each report with its samples, after the script
      names design call. Apply the step limits of the design. Test each
      kind of name and each limit. Pin with a test that a one-word command
      shaped like an action, such as "Run foo/bar@baz", keeps its text.
- [ ] Accept the Boat sample reports with the report limits design call.
      Test the rate limit, a report after the record ends, and a newer
      series that replaces an older one.
- [ ] Add the hold of decision 4 to the retirement check in
      `crates/ciproxy-control-plane/src/boat_authority.rs`, if decision 6
      keeps it. The check skips the sandbox and never waits. Start the 15
      seconds at the completion time that the control plane records for
      the job. Test a late end report, which keeps the final series, and a
      lost one, which keeps the newest sample report. Waits for decision 6.
- [ ] Apply the checks of the design. Test each check, and test that a
      dropped usage keeps the exit.
- [ ] Update the READMEs of `ciproxy-control-plane` and `ciproxy-overflow`.

## Milestone 9: Steps from the Worker log

Depends on: 5

The guest program reads the steps of its job from the Worker log and sends
them with the samples.

- [ ] Add a Worker log reader to `ciproxy-guest`, as the Worker log steps
      design call says. Test it with a Worker log fixture of the pinned
      runner, with a skipped step, and with a log that ends before a step
      result.
- [ ] Add the steps to the samples of each report, with the step limits of
      the design. Test the limits.
- [ ] Add the image check of the log format check design call, and list it
      in `image/checks.md` among the checks that each runner pin update
      reruns.
- [ ] Update the READMEs of `ciproxy-guest` and `image`.

## Milestone 10: Dashboard API

Depends on: 7

The job API gives the figures of the job page.

- [ ] Add the usage of a job to `JobView`: its state, the peaks, at most
      600 chart points, and the figures of each step. Put the new wire
      types in a new module beside `crates/ciproxy-api/src/wire/jobs.rs`,
      which has 264 of its 300 lines.
- [ ] Compute the step figures as the design says, with the "Set up job"
      and "Complete job" steps. Test a step that holds no point, a step
      outside the samples, a skipped step, an out-of-memory kill, and a job
      without steps.
- [ ] Regenerate the API types with `npm --prefix ts run api:generate`.
- [ ] Update the `ciproxy-api` README.

## Milestone 11: Job page

Tags: ui

Depends on: 1, 10

The dashboard shows the panel of the mockups.

- [ ] Build the CPU and memory panel and each of its states in
      `ts/app/src/screens/jobs/`, as the mockups show them. Check
      `@firna/ui` first, and take each colour from `@ciproxy/tokens`.
- [ ] Build the chart as the app chart design call says. Draw the label
      line in the context colour of `chartColors`, not in a status colour.
      Draw the meter tick over `ProgressBar`, not with `BulletChart`,
      whose screen reader text speaks of a target.
- [ ] Add formatters for one decimal of vCPU and GB. The app formatters
      round to whole numbers today.
- [ ] Test the formatting of the figures under `ts/app/_tests_/`.
- [ ] Smoke-test the job page with the dev server, on a phone width and on
      a desktop width.
- [ ] Update the `ts/app` README.

## Milestone 12: Customer docs

Depends on: 11

The public docs explain the figures.

- [ ] Add the samples and the steps to the stored job details in
      `ts/site/src/content/docs/docs/security/job-isolation.md`, with the
      rule words design call. Say that the sandbox token also sends the
      samples of its own job.
- [ ] Add a docs page about the CPU and memory panel: what each figure
      means, and how to use the figures to choose a label size.
- [ ] Update the privacy policy draft in `ts/site/src/data/legal-privacy.ts`:
      add the samples and the steps to the job metadata with the script
      names rule, and use the rule words in the heading "What ciproxy never
      stores or logs". Keep the stored Boat key fact only under "What
      ciproxy stores". The legal pages stay drafts until the user approves
      them, as `plans/follow-ups/legal-pages-approval.md` says.
- [ ] Build the site with `npm --prefix ts run site:build`.

## Milestone 13: Verification, commit, review, and pull request

Depends on: every other milestone

The change works end to end, and the review passes.

- [ ] Build the images from this branch. Run a job with several steps on a
      Linux machine, a Mac, and Boat. Check that the steps of the Worker log
      match the steps on GitHub, with times within one second. Save the
      evidence under `.context/job-step-cpu-and-memory/`.
- [ ] Run a step whose name uses a secret. Check that the Worker log masks
      the secret in the name. If it does not, stop and raise a decision,
      because decision 3 relies on it.
- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record
      follow-up items under `plans/follow-ups/` for the rest, re-review
      once, and report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.

## Post-merge follow-up (non-blocking)

- [ ] Publish the new Linux, macOS, and Boat images, so that jobs on
      customer machines get figures.
