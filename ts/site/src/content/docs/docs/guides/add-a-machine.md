---
title: Add a machine
description: >-
  Connect a Linux machine or a Mac to your workspace with one command.
---

ciproxy runs your jobs on your own machines first. Each machine runs the
open-source agent, which starts a fresh VM for each job.

## What the machine needs

A Linux machine needs:

- x64 Linux with systemd, cgroup v2, and KVM.
- A supported distribution: Debian, Ubuntu, Fedora, RHEL, Rocky Linux, or
  AlmaLinux.
- 4 cores, 16 GB of memory, and 250 GB of disk or more.
- Outbound HTTPS to ciproxy, to GitHub, and to `get.ciproxy.dev`. The
  machine opens no inbound port.

A machine with more threads and memory has more slots, so it runs more jobs
at a time. [Labels and VM sizes](/docs/reference/labels/#slots) gives the
rule.

:::tip[Combine the disks of a CI machine]
A machine used only for CI can combine its disks into one filesystem, for
example with RAID0 or one LVM volume. Its jobs, its machine cache, and its
Docker disks are disposable.
:::

## Add a Mac

A Mac runs the jobs of macOS labels, each in a fresh macOS VM with Xcode. A
Mac needs:

<!-- The disk is a placeholder until the size of the macOS image is
measured. -->

- A Mac with Apple Silicon, M1 or later.
- macOS 26.7.3 or later. A macOS VM cannot run a newer macOS than its Mac,
  so the Mac needs the macOS of the job image.
- 16 GB of memory and 512 GB of disk or more.
- Outbound HTTPS to ciproxy, to GitHub, and to `get.ciproxy.dev`. The Mac
  opens no inbound port.

A Mac uses the same slots as a Linux machine, but it runs at most 2 jobs at
a time, because Apple allows at most 2 macOS VMs on one Mac. A Mac with
16 GB of memory has room for 1 slot, so it runs 1 job at a time. Jobs of a
macOS label never run on Boat. They wait for a free Mac. A Mac counts as one
machine of your plan, as a Linux machine does. Other apps that run macOS
VMs on the Mac count against its limit of 2 VMs too.

Run the same install command in Terminal on the Mac. The agent runs as a
launchd service, and it keeps the Mac awake, so the Mac needs no sleep
setting. In **System Settings**, under **Energy**, turn on **Start up
automatically after a power failure**.

:::caution[FileVault stops jobs after a restart]
With FileVault on, macOS starts no service after a restart until a person
logs in. So the Mac takes no jobs until someone logs in. To check, open
**System Settings**, then **Privacy & Security**, and find **FileVault**.
:::

## Run the install command

Only an Admin of the workspace sees the install command.

1. In ciproxy, open **Machines** and select **Add machine**. For your first
   machine, Setup shows the same command.
2. Copy the command. It holds an install token of your workspace:

   ```sh title="Terminal"
   curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install --token <install token>
   ```

3. Run the command on the machine, in Terminal on a Mac. It installs the
   agent, prepares the machine, and connects it to your workspace.
4. Keep the page open. It shows that it waits for the machine. Then it
   shows the host name of the machine and what the agent reported: threads
   or cores, memory, virtualization, and slots. A new machine first
   downloads the job image and takes jobs when it is ready.

The install token works for one hour. Within that hour, it adds any number
of machines, so you can run the same command on each of them. To end the
token before its hour ends, select **Revoke token** next to the command.
The machines that joined with it stay connected.

[Agent install options](/docs/reference/agent-install/) says what the
command does on the machine.

## When a machine runs fewer jobs

- **No KVM:** the machine connects, but it cannot start VMs, so it gets no
  slots. Turn on virtualization for the machine, then run the command again.
- **A Mac with an older macOS:** the Mac connects, but it cannot start VMs,
  so it gets no slots. Update the Mac to macOS 26.7.3 or later, then run the
  command again.
- **Too little CPU or memory:** a machine without room for one slot after
  the host reserve connects and gets no slots. ciproxy says why.
- **Too little disk:** each job also holds disk, so a machine can run fewer
  jobs of the label `ciproxy` at a time than its slots hold. ciproxy then
  says how many such jobs the machine runs at a time. Add disk to run more.

The page of a machine that cannot run jobs offers to remove it.

## After the machine connects

- **Warm runners:** the warm runner count of each machine is 2 by default.
  ciproxy starts warm runners up to that count when the machine has free
  slots and disk for them. Change the count on the page of the machine.
- **Cost:** enter the monthly cost of the machine on its page. The savings
  report compares that cost with the cost of hosted runners.
- **Plan:** each machine counts toward the machines of your plan, also while
  it is offline or draining. Above your plan, jobs keep running on every
  machine. [Plans and starts](/docs/reference/plans/) gives the numbers.
- **Updates:** the agent updates itself. The page of the machine shows
  whether its agent is up to date.

## Run the command again

Run the install command again on a registered machine to repair the agent.
The machine keeps its registration, its place in the machine count, its
drain state, and its jobs.

A machine that lost its registration, for example after a new install of
its operating system, joins as a new machine. Its old entry stays in
**Machines**, offline, until you remove it, as
[Remove a machine](/docs/guides/remove-a-machine/) says.
