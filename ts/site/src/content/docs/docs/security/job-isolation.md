---
title: Job isolation
description: Each job runs in a fresh VM that no other job used.
---

## A fresh VM for each job

- On your machines, each job runs in a fresh job VM, which the agent starts
  with KVM.
- A job VM runs one runner, which takes one job. The VM ends when its job
  ends, and the agent never uses its disk again.
- So a job never reads the files or the secrets of another job.
- On Boat, each job runs in its own sandbox, which ciproxy deletes when the
  job ends.

## The network of a job VM

- A job VM reaches the internet, as a GitHub-hosted runner does.
- It can also reach the machine cache of its own machine.
- It cannot reach another job VM, the agent, or the host of its machine.
- It cannot reach a private network or a cloud metadata service, also when
  its machine can reach them.

So the code of a pull request cannot probe your internal network or read the
cloud credentials of the machine. A job behaves the same on your machines
and on Boat.

## Your workspace and its organizations

- Each workspace has its own machines, labels, settings, keys, caches, and
  data.
- A machine runs only the jobs of its own workspace.
- A machine runs the jobs of every organization of its workspace, each in a
  fresh VM.
- The cache entries and the Docker disks of each repository stay apart from
  those of other repositories.

## The cache

- A job restores entries by the same keys, versions, and scopes as in the
  GitHub cache. It never restores an entry of another repository.
- Only a job on the default branch writes machine cache entries that every
  branch can restore. Its event must be one of `push`, `workflow_dispatch`,
  `repository_dispatch`, `delete`, `registry_package`, `page_build`, or
  `schedule`.
- A pull request writes machine cache entries only for its own merge ref.
  Another branch or pull request cannot restore them.
- The scope of a save comes from GitHub, not from the job, so the code of a
  job cannot choose it. The machine checks the signed GitHub token of the
  job outside the VM, and reads the ref from it.
- When that check fails, the job restores only entries of the default branch
  from the machine cache, and saves in a private scope that no job restores.
  Its other restores and its permitted saves go through to the GitHub cache.
  The job does not fail.

## Docker disks

On Team and Business, a Docker disk keeps the Docker data of one repository
on one machine between the jobs of that repository.

- One job at a time keeps its changes to the disk: the first job of a
  trusted event on the default branch that uses it. The trusted events are
  those of [the cache](#the-cache).
- Each other job of the repository works on a copy of the disk, which
  ciproxy discards when the job ends.
- A job whose ref the machine cannot check also gets a copy.
- The Docker disk of one repository never attaches to a job of another
  repository.
- When the spare disk of the machine has no room for the disk, or the disk
  cannot attach, the job runs without its Docker disk. Docker then keeps its
  data on the root disk of the job VM.
- A job on Boat gets no Docker disk.

## What a job can read

- A job VM holds the registration of its own runner, which works for that
  one runner only.
- ciproxy never sends the Boat key to a machine, a job VM, or a sandbox.
- A sandbox holds a token that reports the end of its own runner. Job code
  can read this token, but it can only end its own job.

## Code, secrets, and logs

- Code and secrets go from GitHub straight to the runner, and the logs go
  from the runner straight to GitHub.
- The control plane of ciproxy never receives, stores, or logs the code, the
  secrets, the logs, or the cache contents of a job. The cache contents stay
  in the machine cache and in the GitHub cache.
- The control plane stores the details of each job that the dashboard
  shows: its name, workflow, repository, ref, result, place, times, label,
  VM size, cache source, and cache counts.
- Job logs stay on GitHub, and the page of each job links to them.
- ciproxy keeps the details of each finished job for 90 days, on every plan.
  Each page shows the job history of your plan.
