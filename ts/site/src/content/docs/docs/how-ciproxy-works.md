---
title: How ciproxy works
description: Warm runners, slots, overflow, and the cache.
---

ciproxy is the runner provider of its labels on GitHub. A job uses ciproxy
when its `runs-on` line names a ciproxy label. The label has an order of
places where jobs can run. Each place is your own machines or one overflow
provider.

## The parts

- **The GitHub App** connects your organizations to ciproxy. GitHub sends
  ciproxy the jobs of its labels, and gives each runner its job.
- **The control plane** is the hosted service of ciproxy. It decides how
  many runners start, and where.
- **The dashboard** at `app.ciproxy.dev` is where you add machines, set the
  labels, and see the jobs.
- **The agent** runs on each of your machines. It connects out to ciproxy,
  and starts a fresh VM for each runner. The agent is open source under
  Apache-2.0.
- **Your overflow provider accounts** run Linux jobs in sandboxes. Each
  provider bills you for its sandboxes.

## A job on your machines

1. GitHub queues a job whose `runs-on` names a ciproxy label, and tells
   ciproxy about it.
2. A ready warm runner of the label takes the job at once. Otherwise,
   when the job goes to your machines, ciproxy starts a runner on a
   machine that has room.
3. The runner runs the job in a fresh VM that no other job used.
4. When the job ends, the agent ends the VM. If a warm runner took the
   job, ciproxy starts a new one when the machine has room for it.

GitHub gives each waiting job to any idle runner of its label. ciproxy
decides how many runners start and where, not which job a runner takes.
**Jobs** in the dashboard shows where each job ran, and how long it waited.

## Warm runners

- The warm runner count of each machine is 2 by default. Set the count on
  the page of the machine.
- ciproxy starts warm runners up to that count, only in free slots and disk
  that no waiting job needs.
- Warm runners serve `ciproxy` on Linux and `ciproxy-macos` on Macs.
  ciproxy keeps them only when your machines come first in that label's
  order.
- A warm runner never keeps a waiting job from your machines. ciproxy stops
  warm runners to free the slots and the disk that a waiting job needs.
- A draining machine keeps no warm runners.
- In a workspace with several organizations, the warm runners of a machine
  go to each organization by its jobs of the label `ciproxy` in the last
  24 hours.

## Slots and disk

A slot is 2 vCPU and 8 GB of memory of a machine. The machine keeps a
quarter of its threads and memory for the host, and the rest forms slots. A
job takes as many slots as its size needs.

A running job also holds the disk of its VM: the image and the free disk of
its label. A machine first keeps disk for the images, then a quarter of the
rest for the host. The machine cache uses the disk that the jobs leave.
[Labels and VM sizes](/docs/reference/labels/) gives the numbers.

## Where a job runs

ciproxy starts a runner at the first place in the label order that can
take the job. If no place can take it, the job stays queued in GitHub.

- A Linux job waits for room on your machines up to the overflow wait
  before it goes to the next place. If none of your machines can run it,
  the job goes to the next place at once.
- Jobs move between providers without an extra wait. A job skips a
  provider when overflow is off, the provider is disconnected or
  unavailable, or the provider rejected its key.
- A job also skips a provider when its spend limit holds new starts, its
  account needs billing, a refusal blocks new starts, or no size fits.
- macOS jobs wait for a Mac that can take them.

If a provider refuses new sandboxes, ciproxy tries it again for jobs that
still wait. ciproxy never reruns a job by itself.

[Labels and VM sizes](/docs/reference/labels/#custom-labels) explains how
to set the order and the overflow wait.

## A job on an overflow provider

At each provider, ciproxy chooses the cheapest size in your account that
covers the label's vCPU, memory, and complete VM disk. The disk holds the
image, the label's free job disk, and the filesystem allowance. When no
place fits, the label's [When nothing fits][fit] setting decides what to do.

Each sandbox has one runner that takes one job. The job page shows the
provider and the size that ran the job. ciproxy deletes the sandbox when
the job ends. ciproxy also deletes the sandbox of an idle runner when no
waiting job needs it.

[Connect an overflow provider](/docs/guides/connect-an-overflow-provider/)
and [Set a spend limit](/docs/guides/set-a-spend-limit/) give the settings.

## The cache

- `actions/cache` and other actions that use the GitHub cache work without
  workflow edits.
- On your machines, a job restores an entry from the machine cache when the
  machine has it, and from the GitHub cache otherwise. A save goes to both.
- A job on an overflow provider uses the GitHub cache only.
- Only jobs on the default branch write entries that every branch can
  restore. [Job isolation](/docs/security/job-isolation/#the-cache) gives
  the rules.
- A failure of the machine cache never fails a job. The job then uses the
  GitHub cache.
- **Clear cache** on the page of a machine deletes every entry of its
  machine cache. It never deletes an entry of the GitHub cache.

On Team and Business, sticky Docker disks keep the Docker data of each
repository on each machine between its jobs.
[Docker disks](/docs/security/job-isolation/#docker-disks) gives the rules.

## The job image

Each Linux job starts with the software of the GitHub runner image for
`ubuntu-latest`, on your machines and on overflow providers. You can keep
your workflow steps when you change its `runs-on` line.

A job can run its steps in any Docker image, with the `container:` key of
GitHub Actions, on every plan. [Run steps in another image][container] gives
the full example.

[container]: /docs/guides/move-your-workflows/#run-steps-in-another-image
[fit]: /docs/reference/labels/#when-nothing-fits
