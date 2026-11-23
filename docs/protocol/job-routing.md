# Job routing

This doc states the slots of an own machine, where each job runs, and the
warm runners. [Failures](job-routing-failures.md) states what happens when a
part fails. [Labels](labels.md) states which jobs use ciproxy and the size
of each job.

## Terms

- **slot:** 2 vCPU and 8 GB of an [own machine](agent.md#terms).
- **host reserve:** the threads, memory, and disk that the
  [agent](agent.md#terms) keeps for the host of an own machine.
- **image budget:** the separate disk space for images and their downloads.
- **job disk:** the disk that the image budget and host reserve leave for
  the [job VMs](agent.md#job-vms) and the [machine cache](cache.md#terms).
- **runner:** the GitHub Actions runner program that takes one job, in a
  [job VM](agent.md#job-vms) or in a [sandbox](overflow.md#terms) on an
  [overflow provider](overflow.md#terms).
- **warm runner:** a runner on an own machine that is registered with
  GitHub before a job arrives.
- **overflow wait:** how long a job waits for an own machine before it goes
  to the next place of its [label order](labels.md#terms).

## Slots

- A job takes the smallest number of slots that covers both the vCPU and
  the memory of its [size](labels.md#terms).
- That number is the larger of the vCPU divided by 2 and the memory
  divided by 8 GB, each rounded up.
- Each slot is free, holds a warm runner, or holds a job.
- The host reserve keeps a quarter of the hardware threads and memory.
- [Disk](job-routing-capacity.md#disk) gives the separate image budget and
  disk reserve.
- The rest of the threads and the memory forms whole slots.
- One vCPU is one hardware thread.
- The slots of an own machine are the smaller of these two numbers:
  - three quarters of its hardware threads, divided by 2 and rounded down
  - three quarters of its memory, divided by 8 GB and rounded down
- The agent reports the slots and the disk of a machine when it connects.

### Slots on a Mac

- The same slot rule holds on a [Mac](agent.md#terms).
- A Mac also runs at most 2 job VMs at a time, warm runners included, as
  [Platforms](agent.md#platforms) says.
- So the job VMs of a Mac are the smaller of 2 and the number that its
  slots and its job disk hold.
- Example: a Mac mini with 10 threads and 16 GB keeps 2.5 threads and 4 GB
  for the host. Its memory gives 1 slot, so it runs 1 job of `ciproxy-macos`.
- Example: a Mac with 16 threads and 64 GB has 6 slots. It runs 2 jobs of
  `ciproxy-macos` at a time, and 4 of its slots stay free.
- **Decision:** a Mac uses the slot rule of Linux and a limit of 2 job VMs.
  Rejected: at most 2 slots on a Mac, because a label of 2 slots would then
  leave a large Mac with one job. Rejected: a slot rule only for Macs,
  because the scaler and the label dialog would then need a second rule.
  This is a proposal until the user decides it.

### Disk

[Disk](job-routing-capacity.md#disk) gives the image budget, the job disk,
and the complete root of each job VM. It also gives the spare disk, and how
the machine cache and the Docker disks share the job disk.

### Fit

[Fit](job-routing-capacity.md#fit) says when a size fits an own machine,
and how many jobs of one size the machine runs at a time.

### Worked examples

[Worked examples](job-routing-examples.md) covers slots, the 250 GB minimum,
and a mini PC whose disk limits concurrent jobs.

## Where a job runs

- GitHub gives each waiting job to any idle runner of its label.
- ciproxy decides how many runners start and where, not which job a
  runner takes.
- So the dashboard shows no machine for a waiting job.
- ciproxy never re-runs a job by itself, for example after an own machine
  goes offline.

### The label order

- Each label has an [order](labels-order.md) of places. A place is own
  machines or one provider.
- ciproxy starts the runner of a waiting job at the first place of its
  order that can take the job now.
- A job goes to the next place of its order when the current place cannot
  take it now. [Own machines](#own-machines) and
  [When a provider cannot take a job](#when-a-provider-cannot-take-a-job)
  say when a place cannot take a job.
- There is no extra wait between providers.
- **Decision:** a job goes to the next place when the current place cannot
  take it now, with no extra wait between providers. Rejected: a wait at
  each place, and a move after a missed
  [ready deadline](overflow-sandbox-end.md#the-ready-deadline). Reason: the
  user chose this on 23 November 2026.
- ciproxy starts one runner on a provider for each job that goes there.
- A job that no place of its order can take stays queued in GitHub until a
  place can.

### Own machines

- An own machine can run a job when the order of the label holds own
  machines, the machine has the [platform](labels.md#platform) of the
  label, and the size of the job fits the machine.
- **Decision:** a job runs only on a machine of the platform of its label.
  Reason: a macOS job needs macOS and Xcode, and a Linux job needs Linux.
- The machine must also be connected, must have reported its selected
  [image](agent.md#the-image), and must not be
  [draining](agent.md#drain-and-remove).
- ciproxy starts no runner on a machine before it reports its selected
  image. Until then, the dashboard shows a connected machine as preparing,
  as [Install](agent.md#install) says.
- **Decision:** the dashboard shows the machine state that routing uses.
  Rejected: a copy of the routing rule in the dashboard, because a copy can
  drift from routing.
- A job waits up to the overflow wait for enough free slots and available
  job disk for its complete root on one own machine.
- Own machines cannot take a job now when no own machine can run it, or
  when the job has waited the overflow wait and no own machine has room for
  it yet.
- The overflow wait applies only when a job leaves own machines for a later
  place, and only to a job that an own machine can run.
- A job without a runner still starts on an own machine as soon as one
  that can run it has room for it.
- A job of a macOS label never goes to a provider, and no overflow wait
  applies to it.
- It waits in GitHub until a Mac that can run it has a free place, as
  [Platform](labels.md#platform) says.

### The overflow wait

- The customer sets the overflow wait of the workspace in Settings under
  Overflow, with "Wait for a free slot on your machines".
- The overflow wait is 0, 15, 30, 60, or 120 seconds, and 30 seconds by
  default.
- The dashboard shows these values as "At once", "15 seconds",
  "30 seconds", "1 minute", and "2 minutes".

### When a provider cannot take a job

- The customer turns [overflow](overflow.md#terms) on or off for the
  workspace with "Use overflow providers", in Settings under Overflow. It
  is one switch for every provider.
- While overflow is off, each label order skips every provider, also a
  provider that comes before own machines.
- A provider cannot take a job now in these cases:
  - Overflow is off.
  - The provider is not connected, or it is not
    [available](overflow.md#terms).
  - The provider rejected the key, as
    [A rejected key](overflow.md#a-rejected-key) says.
  - The [spend limit](overflow-spend.md) of the provider holds new starts,
    for example at the limit.
  - The provider refused new sandboxes, and the retry time that it gave has
    not come, as [Refused sandboxes](overflow.md#refused-sandboxes) says.
  - The provider account needs billing.
  - No size of the provider account fits the job.
- A macOS label never holds a provider.

### What the dashboard shows

- The live queue shows the running jobs, the waiting jobs, the own slots
  in use, and the running jobs on each provider.
- For each job, the dashboard shows where it ran: the own machine and its
  slot, or the provider and its size.
- It also shows how long the job waited for a runner, and why it ran
  there. The reason names each place of the order that the job passed.
- An example of a reason is "A warm runner was ready, so the job started
  at once."
- Another example is "Your machines were full, so the job started on Boat
  after 30 seconds."
- The waited time runs from the moment GitHub queued the job to the moment
  a runner took it.
- The job page also shows the label of the job and the VM size that ran it.
- The job history can show the jobs of one place: own machines, or one
  provider that ran jobs in its window.
- [A size that nothing can run](labels.md#a-size-that-nothing-can-run)
  gives the wait reason of a label that no place can run now.

## Warm runners

- **Decision:** own machines keep warm runners. Reason: jobs start at
  once.
- Each own machine has a warm runner count, which is 2 by default.
- The customer sets the count of each machine on its machine page.
- On a Linux machine, warm runners serve the label `ciproxy` only.
- On a Mac, warm runners serve the label `ciproxy-macos` only.
- Own machines keep warm runners for a label only while own machines are
  the first place of its [order](labels-order.md).
- **Decision:** warm runners only while own machines come first. Rejected:
  warm runners while own machines are at a later place, because a ready
  warm runner then takes jobs against the order. Reason: the order of each
  label holds, as the user decided on 23 November 2026.
- When the workspace has no label `ciproxy-macos`, its Macs keep no warm
  runners.
- On a Mac, the warm runners also count against the limit of 2 job VMs.
- Each warm runner belongs to one organization of the workspace.
- GitHub gives a waiting job of the label of a warm runner to a ready warm
  runner of its organization at once.
- In a workspace with several organizations, the warm runner count of a
  machine stays one count.
- ciproxy gives each warm runner to an organization by the recent jobs of
  the label of the warm runner in each organization.
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
  slots and available job disk.
- A default warm runner holds 1 slot and the root for 20 GB of free job
  disk, including its image and filesystem allowance.
- A warm runner never keeps a waiting job from an own machine.
- ciproxy stops warm runners to free the slots, the job disk, and on a Mac
  the place of a job VM, that a waiting job needs.
- ciproxy starts warm runners up to the count of a machine, only in free
  slots and available job disk that no waiting job needs.
- When a warm runner takes a job, ciproxy starts a new warm runner in a
  fresh job VM when its slots and its disk are free.
- A draining machine keeps no warm runners.

## Failures

[Failures](job-routing-failures.md) states what happens when the control
plane, an own machine, or a provider fails, and the queue time limit of
GitHub.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Job routing capacity](job-routing-capacity.md): the disk of an own
  machine and the fit of a job.
- [Labels](labels.md): which jobs use ciproxy, the labels, and their sizes.
- [Agent](agent.md): own machines, job VMs, the connection, and drain.
- [Label order](labels-order.md): the places of each label, in order.
- [Overflow](overflow.md): the providers and refused starts.
- [Spend limit](overflow-spend.md): the spend limit of each provider.
- [Cache](cache.md): the machine cache, which uses the available job disk.
- [Security](security.md): repository access.
- [Workspaces](workspaces.md): the organizations whose jobs the labels of a
  workspace serve.
