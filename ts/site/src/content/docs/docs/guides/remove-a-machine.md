---
title: Remove a machine
description: Drain a machine to finish its jobs, then remove it.
---

To take a machine out of your workspace, drain it, remove it in ciproxy, and
then uninstall the agent on the machine.

## Drain the machine

On the page of the machine, select **Drain**.

- The machine finishes its running jobs, and takes no new jobs.
- It keeps no warm runners. ciproxy stops each warm runner only after GitHub
  removes it, so a warm runner that just took a job keeps that job.
- **Machines** shows the machine as draining, with the number of its running
  jobs.

The drain lasts until you select **Stop draining** or remove the machine.
After **Stop draining**, the machine takes new jobs and keeps warm runners
again.

## Remove the machine

When the machine runs no job, select **Remove** on its page.

- **Remove** revokes the credential of the machine at once, also while the
  machine is offline.
- The machine leaves the **Machines** page and the machine count of your
  plan at once.
- The machine takes no new job, and its warm runners stop at once.
- Each job that already runs in a VM of the machine finishes. Then the agent
  ends its VMs and stops.

:::caution[Drain before you remove]
**Remove** stops the warm runners at once. A warm runner can take a job just
before that stop, and that job can then fail. Drain the machine first, and
remove it when it runs no job.
:::

## Uninstall the agent

On the machine, run:

```sh title="Terminal"
sudo /usr/local/bin/ciproxy-agent uninstall
```

Uninstall finishes the running jobs, discards their VMs, and removes the
agent with its disk area. Like **Remove**, it stops the warm runners at
once.

Uninstall does not remove the machine from your workspace. Until you select
**Remove** in ciproxy, the machine stays in **Machines**, offline, and in the
machine count of your plan.

## A machine that joins again as new

A machine that lost its registration, for example after a new install of
its operating system, joins as a new machine. Its old entry stays in
**Machines**, offline, until you remove it.
