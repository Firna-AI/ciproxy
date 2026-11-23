# Labels

This doc states which jobs use ciproxy, the labels of a workspace, and the
size that each label gives a job. It also states where the jobs of each
label may run. [Job routing](job-routing.md) states the slots of an own
machine and where each job runs.

## Terms

- **label:** the value in `runs-on` that sends a job to ciproxy and picks
  the size of its VM.
- **platform:** the operating system and architecture of the job VMs of a
  label: Linux x64 or macOS arm64, as [Platforms](agent.md#platforms) says.
  The product says "Linux" and "macOS".
- **macOS label:** a label whose platform is macOS arm64.
- **size:** the vCPU, memory, and free job disk that a label gives a job. The
  product says "VM size" where the word needs a qualifier.
- **Decision:** "size" is the one word for it. Rejected: "shape". Rejected:
  "machine size", because "machine" names only the hardware of the customer.
- A Linux job also gets a compressed swap in memory, as
  [One definition, two builds](agent-images.md#one-definition-two-builds)
  says. The swap is not part of the size, and it does not change the vCPU,
  memory, or free job disk of a label.
- A macOS job gets no compressed swap from ciproxy, because macOS manages
  its own swap.

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

| Label | Platform | vCPU | Memory | Free job disk | Slots |
| --- | --- | --- | --- | --- | --- |
| `ciproxy` | Linux | 2 | 8 GB | 20 GB | 1 |
| `ciproxy-medium` | Linux | 4 | 16 GB | 20 GB | 2 |
| `ciproxy-large` | Linux | 8 | 32 GB | 60 GB | 4 |
| `ciproxy-macos` | macOS | 2 | 8 GB | 20 GB | 1 |
| `ciproxy-macos-medium` | macOS | 4 | 16 GB | 20 GB | 2 |

- **Decision:** the name of a default label gives a tier, not a size, so
  the name stays true when the customer edits the size. Rejected: names
  with the vCPU count, such as `ciproxy-4vcpu`, because a label has no
  rename.
- The sizes of the macOS labels are a proposal until the user decides them.
- `ciproxy-macos` takes 1 slot, so it fits a Mac with 16 GB of memory.
  `ciproxy-macos-medium` takes 2 slots, so it needs a Mac with 24 GB or more.
- **Decision:** the default macOS labels mirror the two smaller Linux
  labels. Rejected: 4 vCPU and 8 GB, because the size takes 2 slots and
  fits no Mac with 16 GB. Reason: the base Mac mini has 16 GB.

- On an own machine, each job runs in a fresh [job VM](agent.md#job-vms)
  of the size and the platform of its label.
- On Boat, each job runs in a sandbox, as
  [Overflow runners](overflow.md#overflow-runners) says.
- The default Linux labels allow both own machines and Boat.
- The default macOS labels allow own machines only, as
  [Platform](#platform) says.
- The labels of a workspace serve the jobs of all its organizations.

## Custom labels

- The customer adds a label in the
  [dashboard](../architecture.md#components), in Settings under Labels.
- A new label has a name, a platform, vCPU, memory in GB, and free job disk
  in GB.
- Every label has at least 1 GB of free job disk.
- Disk and memory sizes use SI GB, with 1 GB equal to 1,000,000,000 bytes.
- **Decision:** memory stays in SI GB, also though a GitHub-hosted runner of
  8 GB shows about 7.75 GiB. Rejected: memory in GiB, because each machine
  then fits fewer slots. Rejected: the GiB figure from the host reserve,
  because the host then keeps less than a quarter. Reason: the compressed
  swap covers the gap.
- The free disk excludes the image, filesystem metadata, and the 1% root
  reserve. It is available to the runner when the fresh VM starts.
- **Decision:** label disk means free job bytes, so image growth cannot
  consume the promised job space. Rejected: an image floor with a warning,
  refusal on save, and an image floor for placement only.
- The VM root includes the image, the promised free disk, and the ext4
  allowance. Scheduling counts that complete root, as
  [Disk](job-routing.md#disk) says.
- A macOS label follows other disk rules, as
  [The disk of a macOS label](#the-disk-of-a-macos-label) says.
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
- The customer can delete `ciproxy-macos`. The Macs then keep no warm
  runners, as [Warm runners](job-routing.md#warm-runners) says.
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
- The dialog counts only the machines of the platform of the label, such as
  "1 job on 2 of your 3 Macs".
- After the customer changes the size of a label, each job of the label
  that a runner takes gets the new size.
- So ciproxy stops the runners of the old size that have no job, and it
  starts runners of the new size.
- A job that runs during the change keeps its VM.

## The label list

- The list of labels in Settings, under Labels, shows the Linux labels
  first, then the macOS labels.
- Each platform lists its labels from the smallest size to the largest: by
  vCPU, then memory, then free job disk, then name.
- So a label can move in the list when the customer edits its size.
- **Decision:** the list orders the labels by platform and size. Rejected:
  name order, because tier names such as `ciproxy-medium` and
  `ciproxy-large` do not sort by size. Rejected: the order in which the
  labels were made, because it needs a stored order.

## Platform

- Each label has one platform: Linux or macOS.
- The dialog that adds a label offers the platform, with Linux as the
  default.
- A label has no change of platform, as it has no rename.
- The dialog that adds a label says that its platform cannot change later.
- **Decision:** the platform of a label is fixed when the label is made.
  Rejected: a change of platform, because the jobs of each workflow written
  for one operating system would then run on the other. Reason: a new
  label and a delete of the old one give a safe change.
- The jobs of a label run only on own machines of its platform, from the
  image of its platform.
- A Linux machine runs only Linux labels, and a Mac runs only macOS labels.
- A macOS label allows own machines only. Its dialog has no "Boat" switch,
  and it says "macOS jobs run only on your Macs."
- "When nothing fits" of a macOS label offers only "Wait".
- **Decision:** a macOS label never uses Boat. Reason: cloud Macs that rent
  by API bill 24 hours at minimum under Apple's licence, so per-minute
  macOS overflow is not possible. This holds from the MVP foundation plan
  of 3 November 2026.

### The disk of a macOS label

- These rules are planned, and they are a proposal until the user decides
  them.
- The free job disk of a macOS label is at most the measured free bytes of
  the current [macOS image](agent-images-macos.md).
- The dialog shows that limit, and it refuses a larger value.
- Each macOS job VM gets all the free space of its image, which is at least
  the free disk of its label.
- Each macOS job VM holds the free bytes of its image in the job disk, as
  [Disk](job-routing-capacity.md#disk) says.
- When a new macOS image has less free space than a label, the jobs of the
  label still get the free space of the new image. The dialog then shows
  the label above the limit.
- **Decision:** the disk of a macOS label is at most the free space of the
  fixed image disk. Rejected: a disk that the agent grows for each label,
  as [The macOS image](agent-images-macos.md) says. This is a
  proposal until the user decides it.

## Own machines, Boat, or both

- Each Linux label allows own machines, Boat, or both, with the switches
  "Your machines" and "Boat" of its dialog.
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
  and a root disk that covers the image, free job disk, and ext4 allowance.
- Each label has the setting "When nothing fits". It applies while the
  size fits no own machine and no size of the Boat account.
- "Wait" is the default. The jobs of the label stay queued in GitHub until
  an own machine that fits connects, or until the Boat account can start a
  size that fits.
- "Run on the largest Boat size" starts each job of the label on the
  largest size that the Boat account can start, also when it is smaller
  than the label. Its disk must still hold the complete image.
- That fallback can give less free disk than the label promises.
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
