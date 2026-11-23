# Job usage

This doc states the CPU and memory figures of each job and of each step. It
states what ciproxy measures, how the figures reach the control plane, and
what the job page shows.

## Terms

- [Agent images](agent-images.md#terms) defines the guest, the ciproxy
  program in each job VM and each Boat sandbox.
- **sample:** one measure of the CPU and the memory of a job VM or a sandbox.
- **point:** one entry of the samples of a job. It holds one sample, or the
  merge of several neighbouring samples.
- **runner log:** the log that the [runner](job-routing.md#terms) writes in
  its VM, which the runner names the Worker log.
- **out-of-memory kill:** the stop of one process by the kernel of the VM
  for lack of memory.
- **figures:** the CPU, the memory, and the out-of-memory kills of a job or
  of one of its steps.

## What ciproxy measures

- The guest samples the VM of its job once per second.
- A sample covers the whole VM, also the service containers and the
  processes that earlier steps started.
- No sample names a process or a command line.
- **Decision:** the figures cover the whole VM. Rejected: figures for each
  process, because a process name or a command line can hold a secret.
  Reason: the label limits the whole VM.
- The CPU of a sample is the busy vCPU time of the last second.
- All CPU time is busy time, except idle, I/O wait, and steal time.
- On Linux, the memory of a sample is the total memory minus the available
  memory.
- On macOS, the memory of a sample is the app, wired, and compressed
  memory, as Activity Monitor counts them.
- So the memory of a macOS job does not compare with Linux memory.
- Neither platform counts the file cache, because it fills all free memory
  and does not show the size that a job needs.
- On Linux, a sample also counts the out-of-memory kills of its second.
- macOS gives no such count, so a macOS job has no out-of-memory figures.
- **Decision:** the guest measures each job inside its VM, and the job page
  shows the figures. Rejected: figures in the job summary of GitHub, because
  they need an action in each workflow or a new step in each job log.
  Rejected: figures from the host, because Macs and Boat then get none.

### The window

- On an own machine, the samples start when the runner hands its job to
  the guest, just before the first step.
- On Boat, the stock runner has no such handoff. There, the samples start
  when the runner starts the worker process of its job.
- The samples end when the runner exits.
- So the samples leave out the wait of a warm runner, which can last hours
  and says nothing about the job.

### Points

- The samples of a job hold at most 3,600 points.
- When the samples would pass 3,600 points, each two neighbouring points
  merge into one point. The length of a point then doubles.
- A merged point keeps the average CPU, the peak memory, and the sum of the
  out-of-memory kills of its two points.
- So the samples of a job of 6 hours stay under about 100 KB.
- A report gives the length of a point in seconds, the CPU of each point in
  thousandths of a vCPU, and its memory in MiB.
- It gives each time in whole milliseconds since the Unix epoch.
- The samples and the steps carry UTC times from the same clock of the VM.
- So a VM clock that is off moves both by the same amount, and the steps
  still line up with the samples.

## Steps

- The guest reads the name, the start, and the end of each step of its job
  from the runner log. It sends the steps with the samples.
- The guest excludes log pages that exist before it starts the runner.
  It keeps steps on new pages even if they ended before the first sample.
- So the steps come from the runner in the VM, and job code can change
  them, as [Trust](#trust) says.
- **Decision:** the guest reads the steps from the runner log in the VM.
  Rejected: the steps from the jobs API of GitHub, because each job then
  costs a GitHub call, and the times come from another clock. Rejected:
  step events from the runner patch, because Boat runs the stock runner.
- A skipped step has no start and no end.
- When the guest cannot read the steps of a job, the job keeps its samples
  and has no steps.
- The runner log is not a contract of the runner. So before an image takes
  a new runner version, ciproxy checks that the guest still reads the steps.
- The time before the first step shows as the step "Set up job", and the
  time after the last step as "Complete job".
- So the steps match the job page of GitHub, which shows the same two steps.
- The job page numbers the steps in their order, from 1 for "Set up job",
  as GitHub does.
- **Decision:** a point belongs to the step that holds the middle of the
  point. Reason: the runner log gives step times in whole seconds, and each
  point then counts once.
- A step shows the average CPU, the peak memory, and the sum of the
  out-of-memory kills of its points.
- A step that holds no point, or lacks a start or an end, shows no figures.
- The figures of the job are the average CPU, the peak memory, and the sum
  of the out-of-memory kills of all its points.

### Step names

- GitHub names a step without a name from its `run` command or its action, as
  in "Run npm test".
- So the control plane treats each name that starts with "Run", "Pre Run",
  or "Post Run" and a space as a script name.
- A script name keeps its text only when the rest names a remote action,
  such as `actions/checkout@v4` or `docker://alpine:3.20`.
- A local action, such as `./build`, is not a remote action.
- Each other script name becomes "Run script", "Pre Run script", or "Post
  Run script", to match its start.
- So a name such as "Running tests" keeps its text.
- The job page links each step with a replaced name to that step on GitHub.
- The control plane applies this rule before it stores a name, because the
  names come from the VM, which job code controls.
- **Decision:** a script name keeps its text only when it names a remote
  action. Rejected: each name as the runner gives it, because the control
  plane then stores lines of job code. Rejected: no names, or names from
  GitHub for each page view, because the reader then opens GitHub, or each
  view costs a GitHub read.

## How the figures reach the control plane

- On an own machine, the guest sends the samples and the steps to the agent
  when it reports the exit of its runner.
- The agent forwards them to the control plane with that exit.
- An exit that the agent reports by itself, without the guest, has no
  figures.
- On Boat, the [exit report](overflow-sandbox-end.md#the-exit-report) of
  the sandbox carries the final samples and steps.
- While a Boat job runs, the guest also sends its samples and steps so far
  in [sample reports](overflow-sandbox-end.md#sample-reports).
- **Decision:** a Boat guest also sends its samples and steps while the job
  runs. Rejected: the exit report alone, because a sandbox delete before
  that report then loses every figure. Rejected: a read of the samples
  through Boat before each delete, because it adds a Boat call to each one.
- Samples and steps are optional in an exit report. A report without them
  still reports the exit of the runner.
- A failed sample or a failed sample report never stops the runner or its
  exit report.
- A lost or refused exit report loses its figures. On Boat, the newest
  sample report then gives the figures of the job.
- **Decision:** nothing sends the figures of a lost report again. Rejected:
  a queue that retries them, because the figures inform and never decide.

### Checks

- Usage that cannot be decoded follows the same drop rules as failed checks.
- The control plane drops the samples and the steps of a report when its
  samples pass one of these limits:
  - more than 3,600 points
  - lists of CPU, memory, and kills of different lengths
  - a CPU above the vCPU of the VM that ran the job
  - a memory above the memory of that VM
  - a time more than 5 minutes outside the life of the runner, which ends
    at the arrival of a sample report
- On Boat, these limits use the Boat size that ran the job, not the label.
- The 5 minutes allow for VM clocks that differ from the control plane clock.
- The guest sends at most 1,000 steps, each with a name of at most 256
  characters.
- The control plane drops only the steps of a report that passes a step
  limit. It keeps the samples of that report.
- When the samples of an exit report fail a check, the control plane keeps
  the samples of the newest sample report.
- A drop never changes how the control plane handles the exit.
- A drop log gives the reason without values from the report.

## Out of memory

- On Linux, an out-of-memory kill ends only the killed process.
- The job goes on, and the guest still sends its figures.
- So GitHub shows the result of each step, not a stop of the runner.
- When the VM runs out of memory, the kernel kills a process of the job,
  not the guest.
- **Decision:** an out-of-memory kill ends only the killed process, and no
  stop of the processes of the job ends the guest. Rejected: an end of the
  whole job, also with a report at that end, because the job then loses its
  later steps. Rejected: the job inside the service of the guest, because a
  stop of that service still ends the guest.

## Trust

- The samples and the steps come from inside the VM, where job code can
  run as root.
- So job code can change the figures and the steps of its own job, but not
  those of another job.
- Nothing bills, routes, or sizes jobs by the figures.
- **Decision:** the figures only inform. Rejected: automatic size changes,
  because they change the label rules, and job code can fake the figures.
- The checks still apply, because they also protect the control plane from
  a broken guest.

## The job page

- Each job that a runner took has a "CPU and memory" panel on its job page.
- **Decision:** each person with a role in the workspace sees the figures,
  as [Roles](security.md#roles) says. Rejected: figures only for the ciproxy
  team, and a ciproxy view across workspaces, because the figures serve the
  team that owns the jobs.
- For a finished job with figures, the panel shows the average CPU and the
  peak memory of the job against the VM that ran it.
- On Linux, it also shows the out-of-memory kills of the job, or "None".
- A chart shows the CPU and the memory over the time of the samples, with
  a mark for each step and each out-of-memory kill.
- The chart gets at most 600 points, merged as the guest merges them, so
  the page stays small on a phone.
- The panel lists each step with its number, its name, its duration, its
  average CPU, and its peak memory.
- It shows the out-of-memory kills of each step only when one ran out of memory.
- A skipped step shows as "Skipped", with no figures.
- Under the steps, a note says that each figure covers the whole VM, also
  the services and the processes that earlier steps started.
- A job without steps keeps its figures and its chart, without step marks,
  and says "Figures for each step are not available for this job."
- A running job says "The figures show when the job ends."
- **Decision:** a running job shows no figures. Reason: an own machine sends
  them at the end, and a sample report holds only part of a job.
- A finished job without figures says "Figures are not available for this
  job." The panel names no cause.
- The job list shows no figures.

### Meters, plots, and units

- The top of each meter and each plot is the vCPU or the memory of the VM
  that ran the job.
- The kernel of the VM keeps a part of its memory, so a job can run out of
  memory below the top of the meter.
- **Decision:** when the VM has more vCPU or memory than the label gave the
  job, that meter and that plot mark the label size. Reason: a reader
  chooses the size of the label from these figures.
- The page shows each figure in vCPU or in GB, with one decimal. GB is the
  SI unit, as [Custom labels](labels.md#custom-labels) says.
- A VM size or a label size keeps the format of the rest of the page, such as
  "2 vCPU · 8 GB". One value then has one format.
- The [dashboard API](../architecture.md#control-plane) gives the memory in
  bytes as a decimal string, and each time in UTC RFC 3339.

## Tiers and retention

- Every [tier](billing.md#tiers) shows the figures, because every tier
  includes the dashboard.
- The samples and the steps are job metadata, as
  [Secrets and job data](security-data.md#secrets-and-job-data) says.
- So ciproxy keeps them and deletes them with the job, as
  [Data retention](security-data.md#data-retention) says.
- Each tier shows the figures of the jobs in its job history.

## Related docs

- [Job routing](job-routing.md#what-the-dashboard-shows): the job page.
- [Sandbox end](overflow-sandbox-end.md): the Boat exit and sample reports.
- [Data and retention](security-data.md): job metadata and its retention.
- [Security](security.md#key-custody): the token of each sandbox.
- [Agent images](agent-images.md): the guest and the runner in each image.
- [Architecture](../architecture.md): the components and the data flow.
