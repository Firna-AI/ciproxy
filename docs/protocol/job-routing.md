# Job routing

This doc states the slots of an own machine, where each job runs, and the
warm runners. It also states what happens when a part fails.
[Labels](labels.md) states which jobs use ciproxy and the size of each job.

## Terms

- **slot:** 2 vCPU and 8 GB of an [own machine](agent.md#terms).
- **host reserve:** the threads, memory, and disk that the
  [agent](agent.md#terms) keeps for the host of an own machine.
- **image budget:** the separate disk space for images and their downloads.
- **job disk:** the disk that the image budget and host reserve leave for
  the [job VMs](agent.md#job-vms) and the [machine cache](cache.md#terms).
- **runner:** the GitHub Actions runner program that takes one job, in a
  [job VM](agent.md#job-vms) or in a [sandbox](overflow.md#terms) on
  [Boat](overflow.md#terms).
- **warm runner:** a runner on an own machine that is registered with
  GitHub before a job arrives.
- **overflow wait:** how long a job waits for an own machine before
  ciproxy starts a runner for it on Boat.

## Slots

- A job takes the smallest number of slots that covers both the vCPU and
  the memory of its [size](labels.md#terms).
- That number is the larger of the vCPU divided by 2 and the memory
  divided by 8 GB, each rounded up.
- Each slot is free, holds a warm runner, or holds a job.
- The host reserve keeps a quarter of the hardware threads and memory.
- [Disk](#disk) gives the separate image budget and disk reserve.
- The rest of the threads and the memory forms whole slots.
- One vCPU is one hardware thread.
- The slots of an own machine are the smaller of these two numbers:
  - three quarters of its hardware threads, divided by 2 and rounded down
  - three quarters of its memory, divided by 8 GB and rounded down
- The agent reports the slots and the disk of a machine when it connects.

### Disk

- The disk of an own machine is the space in which the agent keeps the
  images, the job VMs, and the machine cache.
- First subtract the image budget from the disk capacity.
- The default image budget is 140,000,000,000 bytes, or 140 GB.
- It holds two full images and one compressed download during an update.
- An explicit image budget override stays separate from the job disk.
- The host reserve keeps a quarter of the remaining disk for the system
  and filesystem overhead.
- The other three quarters of the remaining disk are the job disk.
- A disk smaller than the image budget has no job disk.
- **Decision:** reserve images first, then split the remaining disk 1:3 for
  host and jobs. Rejected: a slim image and shared read-only job roots.
  Reason: keep all `ubuntu-latest` software and independent writable disks.
- Each job VM holds the disk of its size in the job disk, from its start
  until it ends.
- So a warm runner holds the disk of its size, as it holds its slots.
- The machine cache uses the job disk that no job VM holds.
- The machine cache frees space when a job VM needs it, so the machine
  cache never keeps a job from starting.
- The [Docker disks](cache.md#docker-disks) and their copies share the
  free job disk with the machine cache.
- When a job VM needs space, ciproxy deletes the Docker disk that no job
  uses and that a job used least recently.
- Reason: a Docker disk is a cache, so its loss costs only time.
- **Decision:** a job needs job disk as well as slots, and the machine cache
  gets the rest of the disk. Rejected: slots that ignore the disk, because
  jobs could then fill a small disk. Reason: one rule serves every
  resource, as for the threads and the memory.

### Fit

- A size fits an own machine when the machine has at least as many slots
  as the size takes, and its job disk is at least the disk of the size.
- A job VM starts on an own machine only when the machine has enough free
  slots and enough free job disk for its size.
- The free job disk is the job disk that no job VM holds. It includes the
  space that the machine cache uses.
- The jobs of one size that an own machine runs at a time are the smaller
  of these two numbers:
  - its slots, divided by the slots of the size and rounded down
  - its job disk, divided by the disk of the size and rounded down

### Worked examples

[Worked examples](job-routing-examples.md) covers slots, the 250 GB minimum,
and a mini PC whose disk limits concurrent jobs.

## Where a job runs

### Own machines first

- GitHub gives each waiting job to any idle runner of its label.
- ciproxy decides how many runners start and where, not which job a
  runner takes.
- So the dashboard shows no machine for a waiting job.
- A job waits up to the overflow wait for enough free slots and free job
  disk for its size on one own machine.
- ciproxy starts one Boat runner for each job that has waited the overflow
  wait.
- The overflow wait applies only to a job that an own machine can run.
- An own machine can run a job when the label allows own machines and the
  size of the job fits the machine.
- The machine must also be connected and not
  [draining](agent.md#drain-and-remove).
- When no own machine can run a job, ciproxy starts a Boat runner for it
  at once, if Boat can take the job.
- ciproxy never re-runs a job by itself, for example after an own machine
  goes offline.

### The overflow wait

- The customer sets the overflow wait of the workspace in Settings under
  Overflow, with "Start on Boat after a job waits".
- The overflow wait is 0, 15, 30, 60, or 120 seconds, and 30 seconds by
  default.
- The dashboard shows these values as "At once", "15 seconds",
  "30 seconds", "1 minute", and "2 minutes".

### When Boat cannot take a job

- The customer turns [overflow](overflow.md#terms) on or off for the
  workspace with "Use Boat when your machines are busy".
- This switch is in Settings under Overflow.
- ciproxy starts a Boat runner for a job only when overflow is on, the
  label of the job allows Boat, and Boat is available.
- Boat is unavailable before the customer connects Boat and after Boat
  rejects the key, as [The Boat account](overflow.md#the-boat-account)
  says.
- Boat is also unavailable at the [spend limit](overflow.md#spend-limit).
- While Boat cannot take a job, the job runs on own machines only and
  waits there for free slots and free job disk.
- A job that neither an own machine nor Boat can take stays queued in
  GitHub until one of them can.

### What the dashboard shows

- The live queue shows the running jobs, the waiting jobs, the own slots
  in use, and the jobs on Boat.
- For each job, the dashboard shows where it ran: the own machine and its
  slot, or Boat.
- It also shows how long the job waited for a runner, and why it ran
  there.
- An example of a reason is "A warm runner was ready, so the job started
  at once."
- Another example is "Your machines were full, so the job started on Boat
  after 30 seconds."
- The waited time runs from the moment GitHub queued the job to the moment
  a runner took it.
- The job page also shows the label of the job and the VM size that ran it.

## Warm runners

- **Decision:** own machines keep warm runners. Reason: jobs start at
  once.
- Each own machine has a warm runner count, which is 2 by default.
- The customer sets the count of each machine on its machine page.
- Warm runners serve the label `ciproxy` only.
- Each warm runner belongs to one organization of the workspace.
- GitHub gives a waiting job of the label `ciproxy` to a ready warm runner
  of its organization at once.
- In a workspace with several organizations, the warm runner count of a
  machine stays one count.
- ciproxy gives each warm runner to an organization by the recent jobs of
  the label `ciproxy` of each organization.
- The recent jobs are the jobs of the last 24 hours, with equal weight.
- Each organization gets a share of the count in proportion to its recent
  jobs, rounded down. Each runner left over goes to the organization with
  the most recent jobs that has the fewest warm runners.
- When no organization has a recent job, the warm runners go to the
  organizations in turn.
- **Decision:** one warm runner count for each machine, shared by the
  organizations of the workspace by their recent jobs. Rejected: a count
  for each organization, because more VMs then stay idle. Rejected: a count
  for each organization on each machine page, because it adds a control.
  Reason: the warm runners go where the jobs arrive.
- A job of another label gets a new runner in a fresh job VM, in free
  slots and free job disk.
- A warm runner holds the slots and the disk of the size of the label
  `ciproxy`, which are 1 slot and 80 GB by default.
- A warm runner never keeps a waiting job from an own machine.
- ciproxy stops warm runners to free the slots and the job disk that a
  waiting job needs.
- ciproxy starts warm runners up to the count of a machine, only in free
  slots and free job disk that no waiting job needs.
- When a warm runner takes a job, ciproxy starts a new warm runner in a
  fresh job VM when its slots and its disk are free.
- A draining machine keeps no warm runners.

## Failures

### Control plane outage

- No new runner starts on an own machine or on Boat until the control
  plane is back.
- Jobs without a runner stay queued in GitHub, and they start when the
  control plane is back.
- A control plane outage never stops a running job.

### Own machine offline

- [Connection](agent.md#connection) says when an own machine is offline.
- The dashboard shows no slots for an offline machine.
- ciproxy starts no runner on an offline machine.
- New jobs run on the other own machines and on Boat.
- Ready warm runners of an offline machine that still reach GitHub keep
  taking jobs.
- ciproxy does not stop or re-run the running jobs of an offline machine.
- GitHub decides the result of those jobs.
- When the machine is back, the dashboard shows its slots again.

### Boat fails

- While Boat refuses or fails to start sandboxes, new jobs run on own
  machines only and wait there for free slots and free job disk.
- ciproxy keeps trying Boat for the waiting jobs, as
  [Refused sandboxes](overflow.md#refused-sandboxes) says.
- A job that no own machine can run waits until Boat starts a sandbox for
  it.

### Queue time limit of GitHub

- A queued job stays in GitHub until a runner takes it or the queue time
  limit of GitHub ends it.
- ciproxy does not change that limit.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Labels](labels.md): which jobs use ciproxy, the labels, and their sizes.
- [Agent](agent.md): own machines, job VMs, the connection, and drain.
- [Overflow](overflow.md): Boat, the spend limit, and refused starts.
- [Cache](cache.md): the machine cache, which uses the free job disk.
- [Security](security.md): repository access.
- [Workspaces](workspaces.md): the organizations whose jobs the labels of a
  workspace serve.
