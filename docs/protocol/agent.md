# Agent

This doc states the requirements for the agent and for the job VMs that it
runs on own machines.

## Terms

- **own machine:** an x64 Linux computer or an Apple Silicon Mac of the
  customer that runs the agent.
- **Mac:** an own machine with Apple Silicon that runs macOS.
- **platform:** the operating system and the processor architecture of a
  machine, a label, an image, or a job VM. ciproxy supports Linux x64 and
  macOS arm64.
- **virtualization check:** the check of the agent that its machine can
  start job VMs.
- **agent:** the open-source program on an own machine.
- **job VM:** the VM on an own machine in which one job runs.
- **image:** the job image. Job VMs boot its machine image. The
  [sandboxes](overflow.md#overflow-runners) of each provider start from a
  build of it for that provider, as [Agent images](agent-images.md) says.
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

## Platforms

- Each own machine has one platform: Linux x64 or macOS arm64.
- The agent reports the platform of its machine with the hardware, in the
  registration, the hello, and each report.
- The platform of a machine never changes. A new install of the operating
  system registers a new machine, as
  [Machine identity](#machine-identity) says.
- On Linux x64, each job VM is a Firecracker VM over KVM, and it runs
  Linux.
- On a Mac, each job VM is a macOS VM over Apple's Virtualization.framework,
  and it runs macOS.
- A machine runs only the jobs of the [labels](labels.md#platform) of its
  platform.
- **Decision:** a Mac runs only macOS jobs. Rejected: Linux arm64 jobs on
  Macs, because a workflow that needs Xcode still could not run. Rejected
  for now: both.
- Apple's licence and Virtualization.framework allow at most 2 macOS VMs at
  a time on one Mac.
- So a Mac runs at most 2 job VMs at a time, warm runners included, also
  when its [slots](job-routing.md#slots) hold more.
- A third start fails at once with "The number of virtual machines exceeds
  the limit".
- The agent starts no third VM, and the control plane asks for none.
- The agent refuses a runner or an image whose platform differs from the
  platform of its machine. It reports the failure `platform_mismatch`.
- **Decision:** the start of a runner and the download of an image name
  their platform. Reason: a wrong platform then fails with a clear cause,
  and no job of the wrong platform reaches the runner.

## The virtualization check

- The agent checks at each report that its machine can start job VMs.
- On Linux, the check opens `/dev/kvm`.
- On a Mac, the check has three parts:
  - Virtualization.framework reports that the Mac supports VMs.
  - The agent binary has the entitlement
    `com.apple.security.virtualization` in its signature.
  - The macOS release of the Mac can run the
    [macOS image](agent-images.md#the-macos-image).
- The report gives the result: `available`, or the cause of the failure.

| Cause | Platform | Fix that the dashboard names |
| --- | --- | --- |
| `kvm_unavailable` | Linux | Turn on virtualization, then run the command again. |
| `framework_unsupported` | macOS | Install the agent on the Mac itself, not in a VM. |
| `entitlement_missing` | macOS | Install the agent again from the download site. |
| `operating_system_too_old` | macOS | Update macOS. |

- **Decision:** one virtualization result with a cause replaces the KVM
  check. Rejected: a KVM field and a separate Mac field, because the
  dashboard and the slot rule would need a case for each platform. Reason:
  each cause has its own fix.

## Install

- One command installs the agent and connects the own machine to a
  [workspace](workspaces.md#terms).
- [Agent install](agent-install.md) gives the command, what install
  prepares, the install token, and what an own machine needs.
- Until the machine connects, the dashboard shows that it waits for the
  machine.
- Then the dashboard shows the host name of the machine and what the agent
  reported: the platform, threads, memory, the virtualization check, and
  [slots](job-routing.md#slots).
- After the install, the agent downloads and extracts its first image.
- The machine takes no job until it reports its selected image, as
  [Own machines](job-routing.md#own-machines) says.
- Until then, each page that shows the connected machine or the first job
  shows the machine as preparing.
- The dashboard says that the machine gets the image, and that jobs start
  when the machine is ready.
- The first install ends its output with one sentence that explains this
  wait.
- No text about this wait promises a time.
- **Decision:** the text about this wait promises no time. Rejected: an
  expected time, because a slow link makes the large first download long.
- A machine whose virtualization check fails connects, but it cannot start
  a job VM, so it gets no slots.
- The dashboard then says that the machine cannot start VMs, and it names
  the fix of the cause, as
  [The virtualization check](#the-virtualization-check) says.
- A machine without room for one slot after the
  [host reserve](job-routing.md#slots) connects and gets no slots.
- The dashboard says why such a machine has no slots, as it does for a
  machine whose virtualization check fails.
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
- On Linux, the root also includes the ext4 metadata and 1% root reserve
  allowance.
- On Linux, the agent reflinks the image and grows only the private job
  copy before boot. It keeps the base image unchanged.
- On a Mac, the agent makes an APFS clone of the image disk and does not
  grow it. It keeps the base image unchanged. The free space in the image
  disk is the free disk of the job, as
  [macOS labels](labels.md#the-disk-of-a-macos-label) says.
- An APFS clone takes almost no time and no space at first. The spike made
  a clone in 1 ms.
- The actual total allocation must hold the image. An insufficient total
  keeps the existing capacity error.
- On Linux, Docker shares the root disk's free space and its cached Docker
  images, unless a sticky [Docker disk](cache.md#docker-disks) attaches
  under the cache rules. Docker then keeps its data on that disk.
- A macOS job VM has no Docker and no Docker disk, as
  [Docker disks on a Mac](docker-disks.md#docker-disks-on-a-mac) says.
- **Decision:** each job runs in a fresh VM. Rejected: a VM that runs
  several jobs, because a job must not read the secrets or the files of
  another job.
- On a Mac, a start fails when the VM helper of the new VM has not
  answered on its control socket after 60 s. The agent then stops the
  helper and reports a failed start. A stop frees the seat of a macOS VM
  only after its helper has ended, so no new VM uses the same MAC address
  and machine identifier while the old VM runs.
- **Decision (22 November 2026):** one 60 s deadline for a macOS VM
  helper to answer. Rejected: no limit while launchd shows the helper
  running, because a stuck helper would then hold its seat until a stop;
  and a start progress signal from the helper, because it adds a helper
  protocol message for a start that takes about 1 s. Reason: a bounded
  wait lets the agent clean up a hung helper.

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

- The machine page shows the platform, processor, threads, memory, and
  disks that the agent reported.
- The machine page shows these health checks: last contact,
  virtualization, free disk, and agent version.
- The virtualization check shows "Ready" or "Not available" with its cause.
- While a machine is offline, virtualization, free disk, and agent version
  show the values of the last contact, marked as old.
- The page then says to check that the machine is on and can reach the
  control plane.
- The machine page also shows each slot with its job or its warm runner.
- The machine page also shows the use of the
  [machine cache](cache.md#cache-tiers).
- The machine page of a Mac shows no Docker disk figures.

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
- [Overflow](overflow.md): the providers and their sandboxes.
- [Cache](cache.md): the machine cache and the Docker disks.
- [Security](security.md): key custody.
- [Isolation](security-isolation.md#job-isolation): job isolation.
- [Metering](metering.md): the machine count of a tier.
