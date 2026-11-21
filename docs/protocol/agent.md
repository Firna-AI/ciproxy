# Agent

This doc states the requirements for the agent and for the job VMs that it
runs on own machines.

## Terms

- **own machine:** an x64 Linux computer of the customer that runs the
  agent.
- **agent:** the open-source program on an own machine.
- **job VM:** the VM on an own machine in which one job runs.
- **image:** the job image. Job VMs boot its machine image, and
  [Boat sandboxes](overflow.md#overflow-runners) start from its Boat
  snapshot, as [Agent images](agent-images.md#terms) says.
- **install token:** the token in the install command.
- **download site:** the site of ciproxy that serves the install script and
  each version of the agent.
- **machine credential:** the secret of one registered own machine.
- **drain:** a machine finishes its jobs and takes no new ones.
- The product says "machine" for an own machine, never "node" or "server".
- The product never calls a VM a machine.
- **Decision:** the product calls the hardware of the customer a machine.
  Rejected: "node", because some readers think of Kubernetes. Rejected:
  "server", because it reads oddly for a Mac mini on a desk.

## Install

- One command installs the agent and connects the own machine to a
  [workspace](workspaces.md#terms).
- [Agent install](agent-install.md) gives the command, what install
  prepares, the install token, and what an own machine needs.
- Until the machine connects, the dashboard shows that it waits for the
  machine.
- Then the dashboard shows the host name of the machine and what the agent
  reported: threads, memory, KVM, and [slots](job-routing.md#slots).
- After the install, the agent downloads and extracts its first image.
- The machine takes no job until it reports its selected image, as
  [Own machines first](job-routing.md#own-machines-first) says.
- Until then, each page that shows the connected machine or the first job
  shows the machine as preparing.
- The dashboard says that the machine gets the image, and that jobs start
  when the machine is ready.
- The first install ends its output with one sentence that explains this
  wait.
- No text about this wait promises a time.
- **Decision:** the text about this wait promises no time. Rejected: an
  expected time, because a slow link makes the large first download long.
- A machine without KVM connects, but it cannot start a job VM, so it gets
  no slots.
- The dashboard then says that the machine cannot start VMs, and it names
  the fix: turn on virtualization, then run the command again.
- A machine without room for one slot after the
  [host reserve](job-routing.md#slots) connects and gets no slots.
- The dashboard says why such a machine has no slots, as it does for a
  machine without KVM.
- A machine can run fewer jobs of the label `ciproxy` at a time than its
  slots hold, because each job also holds disk, as
  [Disk](job-routing-capacity.md#disk) says.
- The dashboard then says how many such jobs the machine runs at a time.
- The machine page also gives the reason and the fix: add disk.
- The page of a machine that cannot run jobs offers to remove the machine,
  as [Drain and remove](#drain-and-remove) says.
- [Own machines](metering.md#own-machines) says how such a machine counts.

## Connection

[Agent connection](agent-connection.md) holds the rules of the connection.
[Contact](agent-connection.md#contact) says when an own machine is offline.

## Machine identity

- The agent registers its machine once, with the install token.
- After that, the agent connects with the machine credential of its
  machine.
- Each registered machine has its own machine credential.
- [The saved endpoint](agent-endpoint.md#the-saved-endpoint) says which
  control plane receives the machine credential.
- The dashboard shows each machine by its host name.
- The machine credential works until a person removes the machine, as
  [Drain and remove](#drain-and-remove) says.
- A machine that lost its machine credential, for example after a new
  install of its operating system, registers as a new machine.
- Its old registration stays, offline, until a person removes it.
- [Key custody](security.md#key-custody) says where the machine credential
  stays.

## Job VMs

- Each job runs in a fresh job VM that no other job used.
- Each job VM runs one [runner](job-routing.md#terms), which takes one job.
- A job VM ends when its job ends.
- The agent never uses the disk of a job VM again.
- A job VM has the [size](labels.md#terms) of the
  [label](labels.md#terms) of its runner.
- Each job VM gets one writable root disk.
- Its root includes the image and at least the free disk of its label.
- The root also includes the ext4 metadata and 1% root reserve allowance.
- The agent reflinks the image and grows only the private job copy before
  boot. It keeps the base image unchanged.
- The actual total allocation must hold the image. An insufficient total
  keeps the existing capacity error.
- Docker shares the root disk's free space and its cached Docker images,
  unless a sticky [Docker disk](cache.md#docker-disks) attaches under the
  cache rules. Docker then keeps its data on that disk.
- **Decision:** each job runs in a fresh VM. Rejected: a VM that runs
  several jobs, because a job must not read the secrets or the files of
  another job.

## The image

[Agent images](agent-images.md) defines image contents, sizing, and releases.

## Updates

- The agent updates itself to the newest version.
- The agent gets each new version from the download site.
- An update never stops a running job.
- The dashboard shows whether the agent of each machine is up to date.
- **Decision:** the agent updates itself. Rejected: a new run of the install
  command for each version, a package source for the package manager of the
  machine, and a switch that turns updates off, because old agents would
  then stay in use, and the control plane would have to support each old
  version.

## Health

- The machine page shows the processor, threads, memory, and disks that the
  agent reported.
- The machine page shows these health checks: last contact, KVM, free disk,
  and agent version.
- While a machine is offline, KVM, free disk, and agent version show the
  values of the last contact, marked as old.
- The page then says to check that the machine is on and can reach the
  control plane.
- The machine page also shows each slot with its job or its warm runner.
- The machine page also shows the use of the
  [machine cache](cache.md#cache-tiers).

## Drain and remove

- A person drains a machine with "Drain" on its machine page.
- A draining machine finishes its running jobs, takes no new jobs, and
  keeps no warm runners.
- The dashboard shows the machine as draining, with the number of its
  running jobs.
- The drain lasts until a person selects "Stop draining" or removes the
  machine.
- "Stop draining" ends the drain, so the machine takes new jobs and keeps
  warm runners again.
- "Remove" revokes the machine credential at once.
- A removed machine leaves the Machines page and the machine count at once.
- A removed machine takes no new job, and its warm runners stop at once.
- "Remove" takes effect at once, also for an offline machine.
- [The last organization](app-removal.md#the-last-organization-of-a-workspace)
  says what happens to the machines of a workspace when its last
  organization removes the GitHub App.
- In a job VM, the guest is the ciproxy program that supervises the runner.
- The runner hands its job to the guest just before the first step.
- "Remove", uninstall, and an [App removal](app-removal.md#removing-the-app)
  by an organization owner are the immediate stops. They stop warm runners
  at once.
- A warm runner can take a job just before such a stop.
- That job can fail when the runner has not handed it to the guest yet.
- Each job that the guest received finishes. The agent of a removed machine
  then stops its job VMs and stops.
- **Decision:** these stops act at once, also when a job can then fail.
  Reason: the owner wants no extra mechanism, and accepts that a removal
  can fail a job. Rejected: a stop after GitHub removes the runner, a drain
  before "Remove", and a runner that finishes a taken job at a stop.
- Each stop that is not an immediate stop, such as a drain, ends the job VM
  of a warm runner only after GitHub removes the runner.
- A runner that GitHub reports as busy keeps its job.
- After an authorized stop, the agent ends a guest with a proven lost
  registration without waiting for a guest notice.
- That stop releases its VM capacity and preserves running jobs.
- A guest with a delivered registration keeps the guest shutdown path.
- **Decision:** the agent retires a proven unconfigured guest itself.
  Rejected: a guest notice before release, because a guest without its
  registration must not keep capacity reserved.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Agent install](agent-install.md): the install command, the install
  token, and what an own machine needs.
- [Agent connection](agent-connection.md): the connection and its limits.
- [Agent endpoint](agent-endpoint.md): the control plane of an install and
  of the machine credential.
- [Labels](labels.md): the labels and the size of each job VM.
- [Job routing](job-routing.md): slots, the disk of a job, warm runners, and
  failures.
- [Overflow](overflow.md): the Boat sandboxes, which use the same image.
- [Cache](cache.md): the machine cache and the Docker disks.
- [Security](security.md): key custody and job isolation.
- [Metering](metering.md): the machine count of a tier.
