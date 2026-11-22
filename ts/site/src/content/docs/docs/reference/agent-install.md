---
title: Agent install options
description: >-
  The install command, the install token, and the commands of the agent.
---

## The install command

```sh title="Terminal"
curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install --token <install token>
```

The same command works on a Linux machine and on a Mac. On a Mac, run it in
Terminal. The command fetches the install script over HTTPS from
`get.ciproxy.dev`, and runs it as root. The script installs the agent, then runs
`ciproxy-agent install` with the arguments after `--`. So the machine needs
no second command.

Copy the command from Setup, or from **Machines** under **Add machine**.
Only an Admin of the workspace sees it.

A preview command includes `--agent-url` for the control plane that issued
it. The production command keeps the form above.

## What install does

On a Linux machine:

- It checks the Linux distribution, systemd, and cgroup v2 of the machine,
  as [Add a machine](/docs/guides/add-a-machine/#what-the-machine-needs)
  lists them. When a check fails, it stops before it registers the machine,
  and it says what to fix.
- It prepares Firecracker, its jailer, and the host tools that the VMs
  need.
- It prepares the disk area of the agent: one XFS file with reflinks, on any
  filesystem. The area holds the job disk and the images.
- It registers the machine once, with the install token. After that, the
  agent connects with the credential of its own machine.
- It installs the systemd service of the agent.

On a Mac:

- It checks that the Mac has Apple Silicon and a macOS that runs the job
  image, as
  [Add a Mac](/docs/guides/add-a-machine/#add-a-mac) lists them.
- It prepares the disk area of the agent, which holds the job disk and the
  images.
- It registers the machine once, with the install token, as on Linux.
- It installs the launchd daemon `dev.ciproxy.agent`. The daemon keeps the
  Mac awake while the agent runs.

## The install token

- An install token belongs to one workspace.
- It works for one hour.
- Within its hour, it adds any number of machines.
- An Admin can revoke it with **Revoke token** before its hour ends. After
  that, it adds no machine. The machines that joined with it stay connected.

## The commands of the agent

| Command                                         | What it does                                                          |
| ----------------------------------------------- | --------------------------------------------------------------------- |
| `ciproxy-agent install --token <install token>` | Registers the machine and installs its service.                       |
| `ciproxy-agent run`                             | Runs the agent. The service runs this command for you.                |
| `ciproxy-agent uninstall`                       | Finishes the running jobs, discards their VMs, and removes the agent. |

`ciproxy-agent --help` lists the commands.

### Run install again

Run the install command again on a registered machine to repair it. It
reads the existing identity without stopping the service or changing that
identity on disk. It repairs missing parts and keeps the machine identity,
drain state, jobs, and endpoint.

The repair succeeds even when the command selects another endpoint. It
sends no install token. The last line names the retained endpoint and the
command endpoint. It says to uninstall first to move the machine to the
command endpoint.

### The control-plane endpoint

The control-plane endpoint is the `wss://` address that the agent connects
to. Each install command selects an endpoint in this order:

1. `--agent-url`, when the command includes it.
2. `CIPROXY_AGENT_URL`, when set in the environment.
3. `wss://app.ciproxy.dev/agent/`.

The endpoint must be a valid `wss://` URL with a host and a valid TLS
certificate. The command does not take its endpoint from saved state.

A machine without a saved identity registers at the selected endpoint.
Registration saves the endpoint and machine credential together in its
identity. The agent uses that endpoint for every connection with its
machine credential, including job metadata and uninstall. Service restarts
and agent updates keep it.

An update preserves the endpoint of an older installation. If the identity
has no endpoint, the agent recovers it from the old endpoint file. It keeps
a matching file while the identity exists. This keeps the same endpoint
when an older agent runs or you restore saved state. The agent removes that
file only at uninstall, when the identity ends, or when the file names a
different endpoint. This needs no manual action.

When remote removal clears the local identity, it also clears its endpoint.
The next install registers at the endpoint selected by its command.

### Uninstall

```sh title="Terminal"
sudo /usr/local/bin/ciproxy-agent uninstall
```

The command is the same on a Mac. Uninstall removes the disk area of the
agent after the running jobs finish.
It does not remove the machine from your workspace. Select **Remove** on the
page of the machine for that, as
[Remove a machine](/docs/guides/remove-a-machine/) says.

### Paths

| Path        | Linux                           | Mac                            |
| ----------- | ------------------------------- | ------------------------------ |
| The agent   | `/usr/local/bin/ciproxy-agent`  | `/usr/local/bin/ciproxy-agent` |
| The service | systemd `ciproxy-agent.service` | launchd `dev.ciproxy.agent`    |
| The log     | `journalctl -u ciproxy-agent`   | `/var/log/ciproxy/agent.log`   |

## Network

- The agent connects out to its saved control plane, which defaults to
  `app.ciproxy.dev`. The machine opens no inbound port.
- The machine needs outbound HTTPS to ciproxy, to GitHub, and to
  `get.ciproxy.dev`.

## Updates

The agent updates itself to the newest version from `get.ciproxy.dev`. An
update never stops a running job. The page of each machine shows whether its
agent is up to date.
