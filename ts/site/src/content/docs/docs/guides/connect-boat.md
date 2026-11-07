---
title: Connect Boat
description: Start jobs in your own Boat account when your machines are busy.
---

Boat runs your overflow jobs in sandboxes in your own Boat account, and Boat
bills you for them. No ciproxy plan includes the Boat cost. One Boat account
serves every organization of your workspace.

## Connect your Boat account

Only an Admin connects Boat.

1. Create an API key in your Boat account.
2. In ciproxy, open **Settings**, then **Overflow**. Setup has the same
   step, and you can skip it there.
3. Paste the API key, and enter a monthly spend limit in US dollars.
4. Select **Connect Boat**. ciproxy checks the key with Boat before it saves
   it.

After the save, ciproxy shows the name of your Boat account, and never the
key. When Boat does not accept the key, ciproxy saves nothing, and the key
field says so.

[Set a spend limit](/docs/guides/set-a-spend-limit/) says how the limit
counts.

## Choose when jobs start on Boat

In **Settings**, under **Overflow**:

- **Use Boat when your machines are busy** turns overflow on or off for the
  workspace.
- **Start on Boat after a job waits** sets the overflow wait: at once,
  15 seconds, 30 seconds, 1 minute, or 2 minutes. The default is 30 seconds.

The wait applies to a job that one of your machines can run. ciproxy starts
a Boat runner at once for a job that none of your machines can run, for
example because its size fits none of them.

Each label also has the switches **Your machines** and **Boat**, in
**Settings** under **Labels**.
[Labels and VM sizes](/docs/reference/labels/#custom-labels) gives the
rules. The **Machines** page shows your Boat account and the overflow wait.

## How a job runs on Boat

- Each job runs in its own sandbox. ciproxy deletes the sandbox when the job
  ends.
- The sandbox has the cheapest Boat size that covers the vCPU, the memory,
  and the VM disk of the label. The job page names that Boat size.
- Boat ends each sandbox after 6 hours, or 2 hours on a Boat free trial. So
  a job on Boat runs for at most 6 hours, as on a GitHub-hosted runner.
- A job on Boat uses the GitHub cache, and it gets no Docker disk.
- A Boat runner can stay without a job when one of your machines took the
  job first. ciproxy then deletes its sandbox, and it is not a start.

## Change the key or disconnect

- **Change key:** new Boat runners use the new key. The Boat jobs that run
  at that moment finish, and ciproxy deletes the old key when the last of
  them ends.
- **Disconnect:** ciproxy starts no new Boat runner. The Boat jobs that run
  at that moment finish, and ciproxy deletes the key when the last of them
  ends.

When Boat stops accepting your key, jobs run on your machines only until you
connect a new key. **Settings**, under **Overflow**, shows the date when
Boat rejected the key. The new connection asks for the spend limit again,
and the field starts with your saved limit.

## How ciproxy keeps the key

ciproxy stores your Boat key encrypted, and uses it only for the sandboxes
of your workspace. ciproxy never sends the key to a machine, a job VM, or a
sandbox.
