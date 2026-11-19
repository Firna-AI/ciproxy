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
- Each job VM holds its complete root allocation in the job disk until
  the agent removes the VM.
- That root includes the image, the label's free job disk, and an allowance
  for ext4 metadata growth and its 1% root reserve.
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
- The jobs of one size that an own machine runs at a time are the smaller
  of these two numbers:
  - its slots, divided by the slots of the size and rounded down
  - its job disk, divided by the complete root allocation and rounded down

## Related docs

- [Job routing](job-routing.md): slots, where a job runs, and warm runners.
- [Worked examples](job-routing-examples.md): these rules with the current
  image root.
- [Agent install](agent-install.md): the disk that an own machine needs.
- [Cache](cache.md): the machine cache and the Docker disks.
