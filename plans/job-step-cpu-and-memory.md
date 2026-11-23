# CPU and memory of each job step

Status: Active. On branch `calummoore/action-cpu-and-memory-usage`, no pull
request yet. The pull request that completes the work deletes this plan, as
the `plans` skill says.

No part of ciproxy measures the CPU or the memory that a job uses. A
customer cannot see which step of a job needs the CPU or the memory, or
which step ran out of memory. So the customer guesses the size of each
label. This change measures each job inside its VM, and the job page shows
the CPU and the memory of each step.

The guest program samples the CPU and the memory of its VM once per second,
from the start of the job to the exit of the runner. It sends the samples
with its runner exit report. On an own machine, the agent forwards them to
the control plane. On Boat, the end report of the sandbox carries them, and
the guest also sends them every 30 seconds while the job runs. The
control plane stores the samples with the job, and it gets the steps of the
job as decision 1 says. The job page shows the peaks of the job, a chart
over time, and the figures of each step.

The change does not cover size hints, figures on the job list, figures
while a job runs, figures for each process, alerts, or an export to the
customer's own monitoring. A job that ran before the change has no figures.

Decision 1 waits for the user. The user answered decisions 2, 3, and 4 on
23 November 2026. Each TODO that needs the answer to decision 1 ends with
"Waits for decision 1". The other TODOs do not wait.

Contract owners:

- `docs/protocol/job-usage.md`, a new protocol doc that Milestone 2 writes.
- [Secrets and job data](../docs/protocol/security.md#secrets-and-job-data)
  and [Data retention](../docs/protocol/security.md#data-retention).
- [What the dashboard shows](../docs/protocol/job-routing.md#what-the-dashboard-shows).
- [Overflow runners](../docs/protocol/overflow.md#overflow-runners), for the
  end report of a sandbox.
- The [architecture doc](../docs/architecture.md) and its
  [data flow](../docs/architecture/data-flow.md).
- The [mockups](../docs/mockups/README.md) of the job page.
- The READMEs of `ciproxy-guest`, `ciproxy-protocol`, `ciproxy-agent`,
  `ciproxy-github`, `ciproxy-control-plane`, `ciproxy-overflow`,
  `ciproxy-api`, `ciproxy-schema`, `ciproxy-store-interface`,
  `ciproxy-store-pg`, and `ts/app`.

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
    [Secrets and job data](../docs/protocol/security.md#secrets-and-job-data).
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
- **Action:** ask the user, because the user chose D for its reuse of the
  timing read, and that read skips most jobs.

History: the first round of the brainstorm asked about each job as a whole
and recommended C. The user then asked for each step, so the second round
left C out and recommended D. The user chose D on 22 November 2026, in the
words "1 = D". The options check of the same day found that the timing
read skips most jobs, so this decision is open again. The research sources
are [CircleCI](https://circleci.com/docs/resource-class-overview/),
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
- **Location:** `docs/protocol/security.md:276` and
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
  not wait for the end report. The cleanup holds the overflow lock of the
  workspace. An end report that waits for that lock is refused after the
  cleanup ends the record of the sandbox. The samples of decision 1
  travel in the end report.
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
- **Previous examples:** `plans/follow-ups/boat-runner-stderr-drain.md`
  records other end-of-job data that the deletion of a sandbox loses.
- **Related:** Decision 1, work that waits for the answer. Only the Boat
  part of the samples waits.
- **Recommendation:** B. It keeps the figures of each Boat job, and Boat
  bills more only when a report is late. Choose D for figures also when a
  report is lost or refused.
- **Action:** The user chose B and C together on 23 November 2026, in the
  words "4 = B + C".

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
  without a start or an end gets no figures. Reason: GitHub gives step
  times in whole seconds, and each point then counts once.
- **Where to compute:** the API computes the step figures when it serves
  the job page. Reason: the samples and the step times can arrive in
  either order.
- **Chart size:** the API sends at most 600 chart points, merged as the
  guest merges them. Reason: the page stays small on a phone.
- **Clock:** the samples carry UTC times from the clock of the VM. The
  runner in the same VM reports the step times to GitHub. Milestone 13
  proves that the two agree within one second.
- **Units on the wire:** the agent protocol gives memory in MiB. The
  dashboard API gives bytes as decimal strings and times in RFC 3339, as
  `AGENTS.md` says.
- **Job page only:** the job list stays as it is. Reason: the job page
  answers the question, and a list column can follow.
- **Running jobs:** the panel says that the figures come when the job ends.
  Reason: the guest sends the samples once, with the exit report.
- **Older jobs:** a job without figures, for example from an older agent
  or image, shows one plain sentence instead of the panel content.
- **Tiers and retention:** every tier gets the figures. They follow the job
  metadata: ciproxy keeps them 90 days, and each tier shows its job
  history window. Reason: every tier includes the dashboard.
- **Open-source boundary:** the guest, protocol, and agent changes stay in
  their Apache-2.0 crates and use no hosted crate. The store, control
  plane, API, and app changes stay in closed crates.
- **Doc split:** `docs/protocol/security.md` has 317 lines. Milestone 2
  moves "Sign-in and access", with "Roles" and "Who gets a role", into a
  new `docs/protocol/access.md`. Reason: the `documentation` skill splits
  a doc over 250 lines when a change touches it, and this is the largest
  section that stands alone.
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

Contract sketch, for the protocol crate:

- `JobUsage`: the platform, the UTC start of the first point, the length
  of a point in seconds, and three arrays of the same length. The arrays
  hold the CPU in thousandths of a vCPU, the peak memory in MiB, and the
  out-of-memory kills.
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

- [ ] Add a CPU and memory panel to the job page screen, in a new component
      beside `docs/mockups/src/screens/jobs/job-detail.tsx`. It shows the
      peak memory and the average CPU of the job, a chart over time with
      the steps marked, and a table of the steps with their figures. Check
      `@firna/ui` and its chart family first.
- [ ] Add usage fixtures to the own-machine and Boat states of the job
      page. Include a step that holds no point.
- [ ] Name the steps of the fixtures with the script names design call.
      Include a script step that shows as "Run script".
- [ ] Add a sub-page for the panel under
      `docs/mockups/src/entries/app/jobs/`, with at most five states: a
      step that ran out of memory, a macOS job, a running job, a job
      without figures, and a job without step times.
- [ ] Link the existing job routing and security docs from the new pages.
- [ ] Run the mockup check named in `AGENTS.md`. Smoke-test the changed
      pages in the mobile and the desktop variants.
- [ ] Update the design and the TODOs of this plan for each problem that
      the mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

The protocol docs state the requirements of the figures before the code.

- [ ] Write `docs/protocol/job-usage.md`: the terms, the figures, the
      window, the steps, what the job page shows, the platforms, the trust
      rule, retention, and decision 1 with its rejected options. Link the
      mockup pages of Milestone 1. Waits for decision 1.
- [ ] Add the CPU and memory samples and the steps of each job to the job
      metadata in "Secrets and job data" of `docs/protocol/security.md`,
      with the script names and rule words design calls. Say in "Key
      custody" that the sandbox token also sends the samples of its own
      job. Link the new doc.
- [ ] Split `docs/protocol/security.md` as the doc split design call says.
      Update every link to the moved sections in `docs/`, `plans/`, the
      READMEs, the mockups, and `ts/site`.
- [ ] Write the out-of-memory rule of decision 2 into the new doc: only
      the stopped process ends, and the job goes on.
- [ ] Write the hold and the sample reports of decision 4 into
      `docs/protocol/overflow.md`. That doc has 267 lines, so split it as
      the `documentation` skill says.
- [ ] Link the new doc from "What the dashboard shows" in
      `docs/protocol/job-routing.md`.
- [ ] Add the samples to the boundaries "GitHub and the control plane" and
      "The agent and its job VMs" in `docs/architecture.md`. Add them to
      the job flows in `docs/architecture/data-flow.md`.
- [ ] Add the new docs to the protocol doc list in `README.md`.
- [ ] Run the Markdown check named in `AGENTS.md`.

## Milestone 3: Usage in the runner exit messages

Depends on: 2

The protocol carries the samples, and the agent forwards them. The control
plane accepts the field and ignores it until Milestone 8.

- [ ] Add `JobUsage` to `ciproxy-protocol`, with the limits of the design.
      Test each limit.
- [ ] Add the optional `usage` field to `GuestToAgent::RunnerExited` and
      `AgentMessage::RunnerExited`. Test that a message without the field
      parses, and that the field round-trips.
- [ ] Add `GuestToAgent::UsageSamples`, as the contract sketch says. Test
      that it round-trips.
- [ ] Forward the usage of a guest exit report in
      `crates/ciproxy-agent/src/observations.rs`. An exit report that the
      agent makes itself carries no usage. Test both cases.
- [ ] Update the READMEs of `ciproxy-protocol` and `ciproxy-agent`.

## Milestone 4: Jobs that run out of memory

Depends on: 2

A Linux guest keeps running when the kernel stops a step process for lack
of memory.

- [ ] Start the runner in its own scope, as the runner scope design call
      says, in `crates/ciproxy-guest/src/platform.rs`.
- [ ] Give the guest and the runner their out-of-memory scores, as the
      guest protection design call says.
- [ ] Add a stand-in for `systemd-run` to the chroot check in
      `image/check-runner-environment.py`.
- [ ] Prove in a Linux job VM and in a Boat sandbox that a step that runs
      out of memory fails alone, and that the later steps and the exit
      report still run. Check whether Boat runs `systemd-oomd`.
- [ ] Update `plans/follow-ups/boat-runner-stderr-drain.md`, because its
      stderr helper leaves the control group of the guest.
- [ ] Update the READMEs of `ciproxy-guest` and `image`.

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
- [ ] Accept the Boat sample reports with the report limits design call.
      Test the rate limit, a report after the record ends, and a newer
      series that replaces an older one.
- [ ] Add the hold of decision 4 to the retirement check in
      `crates/ciproxy-control-plane/src/boat_authority.rs`, not inside the
      cleanup lock. Test a late end report, which keeps the final series,
      and a lost one, which keeps the newest sample report.
- [ ] Apply the checks of the design. Test each check, and test that a
      dropped usage keeps the exit.
- [ ] Update the READMEs of `ciproxy-control-plane` and `ciproxy-overflow`.

## Milestone 9: Step times

Depends on: 7

The control plane gets the steps of each finished job, as decision 1 says.
The TODOs below are for E. If decision 1 chooses F, replace them before
the milestone starts.

- [ ] Fix the follow-up item `plans/follow-ups/timing-repair-throttle.md`,
      and delete it in the same commit. Waits for decision 1.
- [ ] Read the steps of each finished job once, from the reply of the jobs
      API in `crates/ciproxy-github/src/rest/job_times.rs`. Save the steps
      of every finished job of the run that the reply covers. Waits for
      decision 1.
- [ ] Apply the script names design call to the step names before the
      store write. Test each kind of name.
- [ ] Test that one read serves all finished jobs of a run, and that a step
      without times is saved without them. Waits for decision 1.
- [ ] Update the `ciproxy-github` README. Waits for decision 1.

## Milestone 10: Dashboard API

Depends on: 7

The job API gives the figures of the job page.

- [ ] Add the usage of a job to `JobView`: its state, the peaks, at most
      600 chart points, and the figures of each step. Put the new wire
      types in a new module beside `crates/ciproxy-api/src/wire/jobs.rs`,
      which has 264 of its 300 lines.
- [ ] Compute the step figures as the design says. Test a step that holds
      no point, a step outside the samples, an out-of-memory kill, and a
      job without steps.
- [ ] Regenerate the API types with `npm --prefix ts run api:generate`.
- [ ] Update the `ciproxy-api` README.

## Milestone 11: Job page

Tags: ui

Depends on: 1, 10

The dashboard shows the panel of the mockups.

- [ ] Build the CPU and memory panel and each of its states in
      `ts/app/src/screens/jobs/`, as the mockups show them. Check
      `@firna/ui` first, and take each colour from `@ciproxy/tokens`.
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
- [ ] Build the site with `npm --prefix ts run site:build`.

## Milestone 13: Verification, commit, review, and pull request

Depends on: every other milestone

The change works end to end, and the review passes.

- [ ] Build the images from this branch. Run a job with several steps on a
      Linux machine, a Mac, and Boat. Check that the step figures match the
      step times on GitHub within one second. Save the evidence under
      `.context/job-step-cpu-and-memory/`.
- [ ] Run a step whose name uses a secret. Check that the GitHub jobs API
      masks the secret in the name. If it does not, stop and raise a
      decision, because decision 3 relies on it.
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
