# Job routing capacity

This doc states the disk of an own machine and when a job fits it.
[Job routing](job-routing.md) states the slots, where each job runs, and
the warm runners.

## Terms

- [Job routing](job-routing.md#terms) defines the slot, the host reserve,
  the image budget, the job disk, the runner, and the warm runner.

## Disk

- The disk of an own machine is the space in which the agent keeps the
  images, the job VMs, the machine cache, and the Docker disks.
- First subtract the image budget from the disk capacity.
- The default image budget is 140,000,000,000 bytes, or 140 GB, on Linux.
- It holds two full images and one compressed download during an update.
- A Mac has its own default image budget: 100,000,000,000 bytes, or
  100 GB. It holds one standard
  [macOS image](agent-images.md#the-macos-image), a 100 GB disk with about
  77 GB allocated, and headroom. During an update, the new image and its
  download need space beside the old image. The check of the real free
  space below stops new macOS VMs while that space is short. A failed
  extraction, such as on a full volume, removes its download and its
  partial image, so the old image goes on running jobs.
- **Decision (23 November 2026):** the default macOS image budget is
  100 GB, so a Mac with about 176 GB of free disk runs a default job.
  Rejected for now: a budget for two macOS images and a download, about
  200 GB, because no 256 GB Mac could then run a job. A space policy from
  the measured image size is a follow-up item.
- An explicit image budget override stays separate from the job disk.
- The host reserve keeps a quarter of the remaining disk for the system
  and filesystem overhead.
- The other three quarters of the remaining disk are the job disk.
- A disk smaller than the image budget has no job disk.
- **Decision:** reserve images first, then split the remaining disk 1:3 for
  host and jobs. Rejected: a slim image and shared read-only job roots.
  Reason: keep all `ubuntu-latest` software and independent writable disks.
- Each job VM holds its complete root allocation in the job disk until
  the agent removes the VM.
- On Linux, that root includes the image, the label's free job disk, and
  an allowance for the filesystem: ext4 metadata growth and its 1% root
  reserve.
- On a Mac, the root is an APFS clone of the fixed image disk. The clone
  shares the blocks of the image, which the image budget already holds,
  and takes space only as the job writes.
- So each macOS job VM holds its label's free job disk and a margin of
  4 GB in the job disk. The margin covers the swap, the logs, and the
  runner files that the guest writes beside the job.
- A macOS label fits a Mac only when its free job disk is at most the
  measured free bytes inside the Mac's selected image, because the guest
  cannot use more than the free space of the fixed image disk. The agent
  reports those free bytes with the image.
- A Mac whose agent reports no free bytes for its image gets no new macOS
  job until it does. Its running jobs go on. An older agent counts each VM
  by the Linux rule, so it would refuse each such start.
- A clone can still grow past what it holds. A write to a block that the
  clone still shares with the image takes a new block on the Mac, so a
  clone can grow up to the logical size of the image disk, also when the
  guest's own use stays the same.
- So the agent also refuses a new macOS VM when the real free space of its
  volume is less than the disk that the VM holds. Placement uses the real
  free space of the Mac's last report in the same way, so a job goes to a
  Mac that has the space.
- When the running jobs of a Mac fill its volume, a job that writes more
  gets a full disk inside its VM, and that job can fail. The agent removes
  each VM after its job, which frees the space of its clone.
- **Decision (23 November 2026):** a macOS job VM holds its label's free
  job disk and a margin. Rejected: the logical size of the image disk,
  because the shared blocks of a clone take no space, and a Mac with
  256 GB could then run no job. Rejected: the free bytes of the image,
  because a large free space in the image would also keep small Macs from
  jobs. Reason: a job writes about what its label gives it, and the check
  of the real free space covers a job that writes more.
- Linux keeps its rule: each job VM holds its complete ext4 root.
- Scheduling uses the logical image root length from the same image release
  that the VM starts. Installed file bytes are not that length.
- The agent reports the exact root length of each local image release.
- Scheduling uses each machine's selected local release for new roots.
- A machine that has not reported its selected image gets no new root, as
  [Own machines first](job-routing.md#own-machines-first) says.
- The agent selects and holds its newest image before final sizing and
  admission.
- It rechecks capacity if publication changed the image after the last report.
- Each runner report retains its original free-size request and actual total.
- Retries after publication or restart keep that original allocation.
- A correctly sized idle older root stays usable after a local image update.
- A warm runner holds this same complete root and its slots.
- The spare disk of a machine is the part of its job disk that no set of
  its job VMs can hold at once.
- The slots of the machine, the current labels that it can run, and the
  image of its new job VMs set the spare disk.
- Attached [Docker disks](cache.md#docker-disks) reserve space only from
  the spare disk.
- A Mac has no Docker disks, because a macOS job VM has no Docker.
- **Decision:** a Docker disk reserves only spare disk, so it stops no
  start, except for a short time after a label or image change. Rejected:
  all available job disk, because Docker disks could then delay jobs at any
  time.
- The machine cache uses the job disk that no job VM holds and no attached
  Docker disk reserves.
- The machine cache frees space when a job VM needs it, so the machine
  cache never keeps a job from starting.
- The Docker disks that no job uses share the available job disk with the
  machine cache.
- When a job VM or an attached Docker disk needs space, ciproxy deletes
  the Docker disk that no job uses and that a job used least recently.
- Reason: a Docker disk is a cache, so its loss costs only time.
- **Decision:** a job needs job disk as well as slots, and the machine cache
  gets the rest of the disk. Rejected: slots that ignore the disk, because
  jobs could then fill a small disk. Reason: one rule serves every
  resource, as for the threads and the memory.

## Fit

- A size fits an own machine when the machine has at least as many slots
  as the size takes, and its job disk covers the complete VM root.
- A job VM starts on an own machine only when the machine has enough free
  slots and enough available job disk for its complete root.
- The available job disk is the job disk that no job VM holds and no attached
  Docker disk reserves. It includes the space of the machine cache and of
  the Docker disks that no job uses.
- Outside a change of the labels or the image, Docker disk reservations
  keep no job VM from starting.
- After such a change, only the reservations made before it can delay a
  job VM, until their jobs end.
- A job VM starts on a Mac only when the Mac also runs fewer than 2 job
  VMs.
- The jobs of one size that an own machine runs at a time are the smallest
  of these numbers:
  - its slots, divided by the slots of the size and rounded down
  - its job disk, divided by the complete root allocation and rounded down
  - on a Mac, 2

## Related docs

- [Job routing](job-routing.md): slots, where a job runs, and warm runners.
- [Worked examples](job-routing-examples.md): these rules with the current
  image root.
- [Agent install](agent-install.md): the disk that an own machine needs.
- [Cache](cache.md): the machine cache and the Docker disks.
