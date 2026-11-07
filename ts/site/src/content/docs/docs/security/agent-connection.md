---
title: The agent connection
description: >-
  The agent connects out to ciproxy, and your machine opens no inbound port.
---

## Outbound only

- The agent connects out to ciproxy at `app.ciproxy.dev`.
- Your machine opens no inbound port.
- The machine needs outbound HTTPS to ciproxy, to GitHub, and to
  `get.ciproxy.dev`, the download site of the agent.

## The credential of a machine

- The agent registers its machine once, with the install token of the
  install command. After that, it connects with the credential of its own
  machine.
- Each machine has its own credential. It stays on the machine, where only
  root can read it.
- The credential works until a person removes the machine. **Remove** stops
  it at once, also while the machine is offline.
- An install token belongs to one workspace and works for one hour. An Admin
  can revoke it before its hour ends.
- A machine that lost its credential, for example after a new install of its
  operating system, registers as a new machine. Its old entry stays offline
  until a person removes it.

## What the connection carries

- The agent reports to ciproxy every 10 seconds: its slots, its job VMs, its
  warm runners, its health checks, and its version.
- ciproxy sends the agent the registration of each new runner. A
  registration works for one runner only.
- ciproxy also sends the event, the head repository, and the default branch
  of each job, for the cache and the Docker disk of the job. The machine
  reads the ref of the job from its signed GitHub token.
- ciproxy sends the actions of the dashboard, such as **Drain**,
  **Remove**, and **Clear cache**.

Each runner gets the code and the secrets of its job from GitHub, and sends
the logs straight to GitHub, over its own connection. None of them passes
through the control plane of ciproxy.

## When the connection fails

- The agent keeps its running job VMs and its warm runners, and tries to
  connect again.
- ciproxy shows a machine without contact for 60 seconds as offline, and
  starts no runner on it.
- Ready warm runners of an offline machine that still reach GitHub keep
  taking jobs. ciproxy does not stop the running jobs of an offline machine.
- When the agent connects again, the machine is back online.

During an outage of the control plane, running jobs go on, because each
runner talks to GitHub directly. Jobs without a runner wait in GitHub, and
start when the control plane is back.

## Keys that stay with ciproxy

- The private key of the GitHub App and the Stripe keys stay in the control
  plane of ciproxy.
- ciproxy stores the Boat key of a workspace encrypted, and never sends it
  to a machine, a job VM, or a sandbox.
- Stripe takes the card, so the card never reaches ciproxy.

## Open source

The agent and its connection protocol are open source under Apache-2.0, in
the public ciproxy repository. You can check that each agent binary comes
from the public source.

The agent updates itself to the newest version from `get.ciproxy.dev`. An
update never stops a running job.
