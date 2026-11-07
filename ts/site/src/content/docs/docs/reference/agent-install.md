---
title: Agent install options
description: >-
  The install command, the install token, and the commands of the agent.
---

## The install command

```sh title="Terminal"
curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install --token <install token>
```

The command fetches the install script over HTTPS from `get.ciproxy.dev`,
and runs it as root. The script installs the agent, then runs
`ciproxy-agent install` with the arguments after `--`. So the machine needs
no second command.

Copy the command from Setup, or from **Machines** under **Add machine**.
Only an Admin of the workspace sees it.

## What install does

- It prepares Firecracker, its jailer, and the host tools that the VMs
  need.
- It prepares the disk area of the agent: one XFS file with reflinks, on any
  filesystem. The area holds the job disk and the images.
- It registers the machine once, with the install token. After that, the
  agent connects with the credential of its own machine.
- It installs the systemd service of the agent.

## The install token

- An install token belongs to one workspace.
- It works for one hour.
- Within its hour, it adds any number of machines.
- An Admin can revoke it with **Revoke token** before its hour ends. After
  that, it adds no machine. The machines that joined with it stay connected.

## The commands of the agent

| Command                                         | What it does                                                          |
| ----------------------------------------------- | --------------------------------------------------------------------- |
| `ciproxy-agent install --token <install token>` | Registers the machine and installs its systemd service.               |
| `ciproxy-agent run`                             | Runs the agent. The service runs this command for you.                |
| `ciproxy-agent uninstall`                       | Finishes the running jobs, discards their VMs, and removes the agent. |

`ciproxy-agent --help` lists the commands.

### Run install again

Run the install command again on a registered machine to repair it. It
repairs the missing parts, and keeps the registration of the machine, its
drain state, and its jobs.

### Uninstall

```sh title="Terminal"
sudo /usr/local/bin/ciproxy-agent uninstall
```

Uninstall removes the disk area of the agent after the running jobs finish.
It does not remove the machine from your workspace. Select **Remove** on the
page of the machine for that, as
[Remove a machine](/docs/guides/remove-a-machine/) says.

## Network

- The agent connects out to ciproxy at `app.ciproxy.dev`. The machine opens
  no inbound port.
- The machine needs outbound HTTPS to ciproxy, to GitHub, and to
  `get.ciproxy.dev`.

## Updates

The agent updates itself to the newest version from `get.ciproxy.dev`. An
update never stops a running job. The page of each machine shows whether its
agent is up to date.
