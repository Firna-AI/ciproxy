---
title: How ciproxy works
description: Warm runners, slots, overflow, and the cache.
---

ciproxy is the runner provider of its labels on GitHub. A job uses ciproxy
when its `runs-on` line names a ciproxy label. ciproxy then starts a runner
for the job on one of your machines, or in your Boat account.

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
- **Your Boat account** runs the jobs that your machines have no room for.
  Boat bills you for them.

## A job on your machines

1. GitHub queues a job whose `runs-on` names a ciproxy label, and tells
   ciproxy about it.
2. A ready warm runner takes a job of the label `ciproxy` at once. For a job
   of another label, ciproxy starts a runner in free slots of one of your
   machines.
3. The runner runs the job in a fresh VM that no other job used.
4. When the job ends, the agent ends the VM. ciproxy starts a new warm
   runner when the machine has room for it.

GitHub gives each waiting job to any idle runner of its label. ciproxy
decides how many runners start and where, not which job a runner takes.
**Jobs** in the dashboard shows where each job ran, and how long it waited.

## Warm runners

- The warm runner count of each machine is 2 by default. Set the count on
  the page of the machine.
- ciproxy starts warm runners up to that count, only in free slots and disk
  that no waiting job needs.
- Warm runners serve the label `ciproxy`. A job of another label gets a new
  runner in a fresh VM.
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

## Overflow to Boat

- A job that one of your machines can run waits there for room, up to the
  overflow wait. The wait is 30 seconds by default. Then ciproxy starts one
  Boat runner for the job.
- When none of your machines can run the job, for example because its size
  fits none of them, ciproxy starts the Boat runner at once.
- Each job on Boat runs in its own sandbox, of the cheapest Boat size that
  covers its label. ciproxy deletes the sandbox when the job ends.
- ciproxy starts a Boat runner only when you connected a Boat account,
  overflow is on, the label allows Boat, and the spend limit has room.
  Otherwise the job waits for your machines.
- ciproxy never runs a job again by itself.

[Connect Boat](/docs/guides/connect-boat/) and
[Set a spend limit](/docs/guides/set-a-spend-limit/) give the settings.

## The cache

- `actions/cache` and other actions that use the GitHub cache work without
  workflow edits.
- On your machines, a job restores an entry from the machine cache when the
  machine has it, and from the GitHub cache otherwise. A save goes to both.
- A job on Boat uses the GitHub cache only.
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

Every job starts from one image with the software of the GitHub runner image
of `ubuntu-latest`, on your machines and on Boat. So a workflow keeps its
steps, and only its `runs-on` line changes.

A job can run its steps in any Docker image, with the `container:` key of
GitHub Actions, on every plan. [Run steps in another image][container] gives
the full example.

[container]: /docs/guides/move-your-workflows/#run-steps-in-another-image
