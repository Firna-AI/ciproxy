# Agent

This doc states the requirements for the agent and for the job VMs that it
runs on own machines.

## Terms

- **own machine:** an x64 Linux computer of the customer that runs the
  agent.
- **agent:** the open-source program on an own machine.
- **job VM:** the VM on an own machine in which one job runs.
- **image:** the disk image of the job VMs and of the
  [Boat sandboxes](overflow.md#overflow-runners).
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
- The [dashboard](../architecture.md#components) shows the command in Setup
  and in Machines under "Add machine".
- Only a person who may add a machine sees the command, as
  [Roles](security.md#roles) says.
- The command is one line, with the install token after `--token`:

  ```sh
  curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install --token <install token>
  ```

- The command fetches the install script over HTTPS from the download site,
  and runs it as root.
- The install script installs the agent, then runs the agent with the
  arguments that the command gives the script.
- So the agent runs `install` with the install token, and the machine needs
  no second command.
- **Decision:** the install script runs the agent with the arguments of the
  script. Rejected: a second command that runs the agent after the
  install. Reason: the install command stays one line.
- An install token belongs to one workspace.
- An install token works for one hour.
- Within its hour, an install token adds any number of machines.
- **Decision:** one install token adds any number of machines within its
  hour. Rejected: one machine for each token, an approval of each new
  machine in the dashboard, and a code that the agent prints for a person
  to enter in the dashboard, because each one adds a step for every machine.
- An Admin can revoke an install token before its hour ends.
- After the revoke, the install token adds no machine.
- The machines that registered with the token stay registered.
- **Decision:** an Admin can revoke an install token. Rejected: a token
  that always works for its whole hour, because a leaked token must not
  stay usable.
- An own machine needs x64 Linux with KVM, 4 cores, 16 GB of memory, and
  80 GB of disk or more.
- Reason: with the host reserve of a quarter, such a machine fits one job
  of the label `ciproxy`, which needs 60 GB of disk.
- Next to the command, the dashboard says how long the install token works
  and what the machine needs.
- Until the machine connects, the dashboard shows that it waits for the
  machine.
- Then the dashboard shows the host name of the machine and what the agent
  reported: threads, memory, KVM, and [slots](job-routing.md#slots).
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
  [Disk](job-routing.md#disk) says.
- The dashboard then says how many such jobs the machine runs at a time.
- The machine page also gives the reason and the fix: add disk.
- The page of a machine that cannot run jobs offers to remove the machine,
  as [Drain and remove](#drain-and-remove) says.
- [Own machines](metering.md#own-machines) says how such a machine counts.
- When the install command runs again on a registered machine, the machine
  keeps its registration and its place in the
  [machine count](metering.md#own-machines).
- **Decision:** own machines run x64 Linux only. Rejected: more platforms,
  because one platform is the fastest way to ship.

## Connection

- The agent connects out to the
  [control plane](../architecture.md#components).
- The own machine opens no inbound port.
- The own machine needs outbound HTTPS to the control plane, to GitHub, and
  to the download site.
- The agent reports to the control plane every 10 seconds.
- The dashboard shows a machine without contact for 60 seconds as offline.
- [Failures](job-routing.md#failures) says what happens to the jobs of an
  offline machine.
- When the connection fails, the agent keeps its running job VMs and its
  [warm runners](job-routing.md#warm-runners).
- The agent then tries to connect again.

## Machine identity

- The agent registers its machine once, with the install token.
- After that, the agent connects with the machine credential of its
  machine.
- Each registered machine has its own machine credential.
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
- **Decision:** each job runs in a fresh VM. Rejected: a VM that runs
  several jobs, because a job must not read the secrets or the files of
  another job.

## The image

- The image builds on the open-source GitHub runner images.
- The image has the software of the GitHub runner image that
  `ubuntu-latest` names, so a workflow keeps its YAML apart from the
  `runs-on` line.
- The runner in the image is always a version that GitHub still accepts.
- So ciproxy releases a new image before GitHub stops accepting the runner
  of the current image.
- A job VM starts from the newest image that its machine has, and a
  running job keeps its image.
- **Decision:** one image definition serves the job VMs and the Boat
  sandboxes. Rejected: a separate image for each, because a job must behave
  the same wherever it runs.
- No workspace can run its jobs on a custom VM image yet.
- Custom VM images come later, on Business, as
  [Tier features](billing.md#tier-features) says.

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
- The agent of a removed machine lets its running jobs finish, then stops
  its job VMs and stops.
- "Remove" takes effect at once, also for an offline machine.
- [The last organization](app-removal.md#the-last-organization-of-a-workspace)
  says what happens to the machines of a workspace when its last
  organization removes the GitHub App.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Labels](labels.md): the labels and the size of each job VM.
- [Job routing](job-routing.md): slots, the disk of a job, warm runners, and
  failures.
- [Overflow](overflow.md): the Boat sandboxes, which use the same image.
- [Cache](cache.md): the machine cache and the Docker disks.
- [Security](security.md): key custody and job isolation.
- [Metering](metering.md): the machine count of a tier.
