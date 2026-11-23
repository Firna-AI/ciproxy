# Docker disks

This doc states the rules for the Docker disks of
[own machines](agent.md#terms). [Cache](cache.md) states the rules of the
machine cache.

## Terms

- **Docker disk:** a disk for one repository on one own machine that keeps
  Docker data between the jobs of the repository.
- **Docker disk figures:** how many repositories have a Docker disk on the
  machine, the space of these disks, and the space that attached Docker
  disks and copies reserve.
- The [tiers](billing.md#tiers) call the Docker disk feature "sticky Docker
  disks".

## Docker disks on a Mac

- A Mac has no Docker disks, because a macOS job VM has no Docker.
- The agent of a Mac offers no Docker disk to any job.
- The guest in a macOS job VM reports `docker_data_bytes` as 0.
- The machine page of a Mac shows no
  [Docker disk figures](#terms).
- So sticky Docker disks of Team and Business apply to Linux machines only.
- **Decision:** a Mac has no Docker disks. Reason: the macOS image has no
  Docker, as the GitHub macOS runner images have none.

## Writers and copies

- A Docker disk keeps the Docker data of one repository on one own machine
  between the jobs of the repository.
- A Docker disk attaches to the [job VM](agent.md#job-vms) after the job
  starts, because ciproxy learns the repository of a job only then.
- One job at a time keeps its changes to a Docker disk: the first job of a
  trusted event on the default branch that uses the disk.
- The trusted events are those of
  [Trusted write-back](cache.md#trusted-write-back): `push`,
  `workflow_dispatch`, `repository_dispatch`, `delete`, `registry_package`,
  `page_build`, and `schedule`.
- Each other job of the repository works on a copy of the disk, also each
  job that does not run on the default branch, and each job of another
  event.
- ciproxy discards a copy when its job ends.
- The Docker disk uses the same machine-checked ref and trusted-event rule
  as the machine cache.
- A job without a checked ref gets a disposable copy of the Docker disk.
- ciproxy discards that copy when the job ends.
- The Docker disk of one repository never attaches to a job of another
  repository.
- A job on Boat gets no Docker disk.
- **Decision:** only a job of a trusted event on the default branch keeps
  its changes to a Docker disk, as for the machine cache. Rejected: any
  event on the default branch, because a person outside the repository
  can start some of them, and `push` jobs only, because the disk would
  then update less often.
- **Decision:** Docker persistence uses the checked token scope. Rejected:
  an API branch name or a job-supplied ref, because neither proves the full
  ref that may keep disk changes.

## Disk space and failures

- An attached Docker disk or copy reserves its data plus the free disk of
  the label of its job.
- It takes that reservation only from the spare disk of the machine, as
  [Disk](job-routing.md#disk) says.
- It keeps the reservation until its job ends.
- It never shrinks or detaches during its job.
- When the spare disk has no room for the reservation, or the disk cannot
  attach, the job runs without its Docker disk.
- Docker then keeps its data on the root disk of the job VM.
- ciproxy discards a repository Docker disk that fails to mount.
- The next trusted writer builds a fresh disk.
- **Decision:** discard the failed disk instead of repairing it for reuse,
  because rebuilding disposable Docker data loses only time.
- So a machine whose disk limits its jobs gets few or no Docker disks.
- **Decision:** an attached Docker disk reserves its data plus the free disk
  of its label. Rejected: a Docker budget, because the 250 GB minimum then
  fits no default job. Reason: Docker then gets the free disk of the label,
  as it does on the root disk.

## Unavailable figures

- When the agent cannot read the Docker disk figures of a machine, it reports
  them as unavailable, as [Machine figures](cache.md#machine-figures) says.
- While these figures are unavailable, ciproxy never counts the reservations
  of the machine as zero bytes.
- The dashboard then shows the capacity of the machine as unknown. It does
  not say how many jobs the machine runs at a time, or that its disk limits
  them.
- ciproxy starts no new runner on the machine, also no warm runner.
- ciproxy still counts the machine for the labels whose size it
  [fits](job-routing.md#fit), so their jobs still wait the
  [overflow wait](job-routing.md#terms) before Boat runners start for them.
- The jobs and the runners already on the machine stay, and a ready warm
  runner can still take a job.
- The agent refuses each runner start while it cannot read the reservations.
- New runners can start on the machine again after a report with measured
  figures.
- **Decision:** a machine with unavailable reservations keeps its labels and
  gets no new runner. Rejected: new runners as before, because the agent
  refuses each of them.

## Tiers and the machine page

- The machine page shows how many repositories have a Docker disk on the
  machine, and the space that these disks take.
- Only Team and Business include Docker disks, as
  [Tier features](billing.md#tier-features) says.
- A job of a workspace on Free gets no Docker disk.

## Related docs

- [Cache](cache.md): the machine cache and its trusted write-back.
- [Agent](agent.md): own machines and job VMs.
- [Job routing](job-routing.md#disk): the disk of a job and the spare disk.
- [Billing](billing.md#tier-features): the tiers that include Docker disks.
- [Isolation](security-isolation.md#job-isolation): job isolation.
- [Data and retention](security-data.md): job data.
