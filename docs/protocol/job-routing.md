# Job routing

This doc states which jobs ciproxy takes, the VM size of each job, and
where each job runs. It also states what happens when a part fails.

## Terms

- **label:** the value in `runs-on` that sends a job to ciproxy and picks
  the size of its VM.
- **size:** the vCPU, memory, and disk of the VM that a label starts.
- **Open:** the word for the vCPU, memory, and disk of a label. It waits
  for item 11, the word "size" or "shape".
- **slot:** 2 vCPU and 8 GB of an [own machine](agent.md#terms).
- **host reserve:** the threads and memory that the
  [agent](agent.md#terms) keeps for the host of an own machine.
- **runner:** the GitHub Actions runner program that takes one job, in a
  [job VM](agent.md#job-vms) or in a [sandbox](overflow.md#terms) on
  [Boat](overflow.md#terms).
- **warm runner:** a runner on an own machine that is registered with
  GitHub before a job arrives.
- **overflow wait:** how long a job waits for a slot on an own machine
  before ciproxy starts a runner for it on Boat.

## Labels

- A job uses ciproxy when its `runs-on` names a ciproxy label.
- **Decision:** ciproxy is the runner provider for its labels and gets the
  jobs through the GitHub scale set API. Rejected: adopting existing
  tools, because nothing is left to sell. Rejected: a label router that
  picks a provider label before each job queues, because it needs a router
  job and an edit in each job.
- Only jobs of the repositories that the customer allows get a ciproxy
  runner, as [Repository access](security.md#repository-access) says.

### Default labels

- Every [organization](security.md#terms) starts with these labels:

| Label | vCPU | Memory | Disk | Slots |
| --- | --- | --- | --- | --- |
| `ciproxy` | 2 | 8 GB | 60 GB | 1 |
| `ciproxy-4vcpu` | 4 | 16 GB | 80 GB | 2 |
| `ciproxy-8vcpu` | 8 | 32 GB | 120 GB | 4 |

- On an own machine, each job runs in a fresh job VM of the size of its
  label.
- On Boat, each job runs in a sandbox, as
  [Overflow runners](overflow.md#overflow-runners) says.
- The default labels allow both own machines and Boat.

### Custom labels

- The customer adds a label in the
  [dashboard](../architecture.md#components), in Settings under Labels.
- A new label has a name, vCPU, memory in GB, and disk in GB.
- The customer edits the size of any label in the same place.
- The dialog that adds or edits a label states how many jobs of its size
  fit on one own machine at a time.
- After the customer changes the size of a label, each job of the label
  that a runner takes gets the new size.
- So ciproxy stops the runners of the old size that have no job, and it
  starts runners of the new size.
- A job that runs during the change keeps its VM.

### Own machines, Boat, or both

- Each label allows own machines, Boat, or both, with the switches "Your
  machines" and "Boat" of its dialog.
- Jobs of a label that allows both start on own machines first, and on
  Boat when the own machines are busy.
- While the size of a label fits no own machine, the label runs on Boat
  only.
- The dialog says so before the customer saves the label.
- When an own machine that fits the size connects, the jobs of the label
  start on own machines first again.
- This holds unless the customer turned "Your machines" off for the label.

## Slots

- A job takes the smallest number of slots that covers both the vCPU and
  the memory of its size.
- That number is the larger of the vCPU divided by 2 and the memory
  divided by 8 GB, each rounded up.
- A size fits an own machine when the machine has at least as many slots
  as the size takes.
- Each slot is free, holds a warm runner, or holds a job.
- The host reserve of an own machine is a quarter of its hardware threads
  and a quarter of its memory.
- The rest of the machine forms whole slots.
- One vCPU is one hardware thread.
- The slots of an own machine are the smaller of these two numbers:
  - three quarters of its hardware threads, divided by 2 and rounded down
  - three quarters of its memory, divided by 8 GB and rounded down
- The agent reports the slots of a machine when it connects.

### Worked examples

- An own machine with 16 threads and 64 GB, such as a Hetzner AX42, keeps
  4 threads and 16 GB as its host reserve.
- Its other 12 threads give 6 slots, and its other 48 GB give 6 slots.
- So it has 6 slots, which offer 12 vCPU and 48 GB to jobs.
- An own machine with 4 threads and 16 GB keeps 1 thread and 4 GB as its
  host reserve.
- Its other 3 threads give 1.5 slots, and its other 12 GB give 1.5 slots.
- Both numbers round down, so it has 1 slot.
- So every own machine that meets the
  [hardware minimum](agent.md#install) of 4 cores and 16 GB has at least
  1 slot.

## Where a job runs

### Own machines first

- GitHub gives each waiting job to any idle runner of its label.
- ciproxy decides how many runners start and where, not which job a
  runner takes.
- So the dashboard shows no machine for a waiting job.
- A job waits up to the overflow wait for enough free slots for its size
  on one own machine.
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

- The customer sets the overflow wait of the organization in Settings
  under Overflow, with "Start on Boat after a job waits".
- The overflow wait is 0, 15, 30, 60, or 120 seconds, and 30 seconds by
  default.
- The dashboard shows these values as "At once", "15 seconds",
  "30 seconds", "1 minute", and "2 minutes".

### When Boat cannot take a job

- The customer turns [overflow](overflow.md#terms) on or off for the
  organization with "Use Boat when your machines are busy".
- This switch is in Settings under Overflow.
- ciproxy starts a Boat runner for a job only when overflow is on, the
  label of the job allows Boat, and Boat is available.
- Boat is unavailable before the customer connects Boat and after Boat
  rejects the key, as [The Boat account](overflow.md#the-boat-account)
  says.
- Boat is also unavailable at the [spend limit](overflow.md#spend-limit).
- While Boat cannot take a job, the job runs on own machines only and
  waits there for a free slot.
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

## Warm runners

- **Decision:** own machines keep warm runners. Reason: jobs start at
  once. Warm runners also keep work going through a short outage of the
  [control plane](../architecture.md#components), because a queued ciproxy
  job cannot move to another label.
- Each own machine has a warm runner count, which is 2 by default.
- The customer sets the count of each machine on its machine page.
- Warm runners serve the label `ciproxy` only.
- GitHub gives a waiting job of the label `ciproxy` to a ready warm runner
  at once.
- A job of another label gets a new runner in a fresh job VM in free
  slots.
- A warm runner holds the slots of the size of the label `ciproxy`, which
  is 1 slot by default.
- A warm runner never keeps a waiting job from an own machine.
- ciproxy stops warm runners to free the slots that a waiting job needs.
- ciproxy starts warm runners up to the count of a machine, only in free
  slots that no waiting job needs.
- When a warm runner takes a job, ciproxy starts a new warm runner in a
  fresh job VM when such a slot is free.
- A draining machine keeps no warm runners.

## Failures

### Control plane outage

- During a control plane outage, warm runners that are ready keep taking
  jobs.
- No new runner starts on an own machine or on Boat until the control
  plane is back.
- Jobs without a runner stay queued in GitHub, and they start when the
  control plane is back.
- A control plane outage never stops a running job.

### Own machine offline

- An own machine is offline when the control plane has had no contact
  with it for 60 seconds, as [Connection](agent.md#connection) says.
- The dashboard shows no slots for an offline machine.
- ciproxy starts no runner on an offline machine.
- New jobs run on the other own machines and on Boat.
- Ready warm runners of an offline machine that still reach GitHub keep
  taking jobs, as in a control plane outage.
- ciproxy does not stop or re-run the running jobs of an offline machine.
- GitHub decides the result of those jobs.
- When the machine is back, the dashboard shows its slots again.

### Boat fails

- While Boat refuses or fails to start sandboxes, new jobs run on own
  machines only and wait there for a free slot.
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
- [Agent](agent.md): own machines, job VMs, the connection, and drain.
- [Overflow](overflow.md): Boat, the spend limit, and refused starts.
- [Security](security.md): organizations and repository access.
