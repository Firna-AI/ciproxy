# Labels

This doc states which jobs use ciproxy, the labels of a workspace, and the
size that each label gives a job. It also states where the jobs of each
label may run. [Job routing](job-routing.md) states the slots of an own
machine and where each job runs.

## Terms

- **label:** the value in `runs-on` that sends a job to ciproxy and picks
  the size of its VM.
- **size:** the vCPU, memory, and disk that a label gives a job. The
  product says "VM size" where the word needs a qualifier.
- **Decision:** "size" is the one word for it. Rejected: "shape". Rejected:
  "machine size", because "machine" names only the hardware of the customer.

## Jobs that use ciproxy

- A job uses ciproxy when its `runs-on` names a ciproxy label.
- **Decision:** ciproxy is the runner provider for its labels and gets the
  jobs through the GitHub scale set API. Rejected: adopting existing
  tools, because nothing is left to sell. Rejected: a label router that
  picks a provider label before each job queues, because it needs a router
  job and an edit in each job.
- Only jobs of the repositories that the customer allows get a ciproxy
  [runner](job-routing.md#terms), as
  [Repository access](security.md#repository-access) says.

## Default labels

- Every [workspace](workspaces.md#terms) starts with these labels:

| Label | vCPU | Memory | Disk | Slots |
| --- | --- | --- | --- | --- |
| `ciproxy` | 2 | 8 GB | 60 GB | 1 |
| `ciproxy-4vcpu` | 4 | 16 GB | 80 GB | 2 |
| `ciproxy-8vcpu` | 8 | 32 GB | 120 GB | 4 |

- On an own machine, each job runs in a fresh [job VM](agent.md#job-vms)
  of the size of its label.
- On Boat, each job runs in a sandbox, as
  [Overflow runners](overflow.md#overflow-runners) says.
- The default labels allow both own machines and Boat.
- The labels of a workspace serve the jobs of all its organizations.

## Custom labels

- The customer adds a label in the
  [dashboard](../architecture.md#components), in Settings under Labels.
- A new label has a name, vCPU, memory in GB, and disk in GB.
- A label name follows the label rules of GitHub. When the customer saves a
  new label, ciproxy checks its name with GitHub, and the dialog shows the
  refusal of GitHub.
- **Decision:** GitHub checks each label name, and ciproxy adds no name
  rule of its own. Reason: the name must work in `runs-on`, and GitHub
  decides what works there.
- A label has no rename.
- The dialog that adds a label says that its name cannot change later.
- **Decision:** a label has no rename. Rejected: a rename, because the jobs
  of each workflow that still names the old label would then get no
  runner. Reason: a new label and a delete of the old one give a safe
  rename, because both labels take jobs while the customer moves the
  workflows.
- The customer can delete any label except `ciproxy`.
- The waiting jobs of a deleted label stay queued in GitHub, its running
  jobs finish, and ciproxy removes its scale sets.
- **Decision:** every label except `ciproxy` can be deleted. Rejected: a
  delete of every label, because the warm runners serve `ciproxy`.
  Rejected: a delete of custom labels only, because a workspace may drop a
  default label that it never uses.
- The customer edits the size of any label in the same place.
- The dialog that adds or edits a label states how many jobs of its size
  fit on each own machine at a time, as [Fit](job-routing.md#fit) says.
- When the machines fit different numbers of jobs, the dialog shows a
  range, such as "1 to 3 jobs".
- **Decision:** machines that fit different numbers of jobs give a range.
  Rejected: a total for all machines. Reason: the range keeps the rule of
  one machine, because a job runs on one machine.
- When some machines fit no job of the size, the dialog says how many
  machines it fits, such as "1 job on 3 of your 4 machines".
- The dialog counts only the connected machines that can run jobs, and no
  draining machine.
- After the customer changes the size of a label, each job of the label
  that a runner takes gets the new size.
- So ciproxy stops the runners of the old size that have no job, and it
  starts runners of the new size.
- A job that runs during the change keeps its VM.

## Own machines, Boat, or both

- Each label allows own machines, Boat, or both, with the switches "Your
  machines" and "Boat" of its dialog.
- Jobs of a label that allows both start on own machines first, and on
  Boat when the own machines are busy.
- While the size of a label [fits](job-routing.md#fit) no own machine, the
  label runs on Boat only.
- The dialog says so before the customer saves the label.
- When an own machine that fits the size connects, the jobs of the label
  start on own machines first again.
- This holds unless the customer turned "Your machines" off for the label.

## A size that nothing can run

- ciproxy reads the VM sizes that the customer's Boat account can start
  now, through the backend contract in [Overflow](overflow.md).
- A size fits Boat when one of those sizes has at least its vCPU, memory,
  and disk.
- Each label has the setting "When nothing fits". It applies while the
  size fits no own machine and no size of the Boat account.
- "Wait" is the default. The jobs of the label stay queued in GitHub until
  an own machine that fits connects, or until the Boat account can start a
  size that fits.
- "Run on the largest Boat size" starts each job of the label on the
  largest size that the Boat account can start, also when it is smaller
  than the label.
- The dialog, the label list, and the live queue show where the jobs of
  each label run now: own machines, Boat, or wait.
- **Decision:** the customer chooses between waiting and a smaller Boat VM.
  Rejected: rejecting such a label on save, and failing its jobs at once,
  because a label can stop fitting after a machine leaves or the Boat plan
  changes.

## Related docs

- [Job routing](job-routing.md): slots, the disk of a job, where a job
  runs, and warm runners.
- [Overflow](overflow.md): the Boat sizes and the sandboxes.
- [Agent](agent.md): own machines and job VMs.
- [Security](security.md): repository access.
- [Workspaces](workspaces.md): the organizations whose jobs the labels of a
  workspace serve.
