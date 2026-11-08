---
title: Add a machine
description: Connect a Linux machine to your workspace with one command.
---

ciproxy runs your jobs on your own machines first. Each machine runs the
open-source agent, which starts a fresh VM for each job.

## What the machine needs

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

## Run the install command

Only an Admin of the workspace sees the install command.

1. In ciproxy, open **Machines** and select **Add machine**. For your first
   machine, Setup shows the same command.
2. Copy the command. It holds an install token of your workspace:

   ```sh title="Terminal"
   curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install --token <install token>
   ```

3. Run the command on the machine. It installs the agent, prepares the
   machine, and connects it to your workspace.
4. Keep the page open. It shows that it waits for the machine. Then it
   shows the host name of the machine and what the agent reported: threads,
   memory, KVM, and slots.

The install token works for one hour. Within that hour, it adds any number
of machines, so you can run the same command on each of them. To end the
token before its hour ends, select **Revoke token** next to the command.
The machines that joined with it stay connected.

[Agent install options](/docs/reference/agent-install/) says what the
command does on the machine.

## When a machine runs fewer jobs

- **No KVM:** the machine connects, but it cannot start VMs, so it gets no
  slots. Turn on virtualization for the machine, then run the command again.
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
