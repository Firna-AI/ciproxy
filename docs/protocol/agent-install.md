# Agent install

This doc states the install command of the agent, what install prepares,
the install token, and what an own machine needs.
[Install](agent.md#install) states what the dashboard shows after the
command, and the wait for the first image.

## Terms

- [Agent](agent.md#terms) defines the own machine, the agent, the install
  token, and the download site.
- [Job routing](job-routing.md#terms) defines the host reserve, the image
  budget, and the job disk.

## The command

- The [dashboard](../architecture.md#components) shows the command in Setup
  and in Machines under "Add machine".
- Only a person who may add a machine sees the command, as
  [Roles](security.md#roles) says.
- Next to the command, the dashboard says how long the install token works
  and what the machine needs.
- The command is one line, with the install token after `--token`:

  ```sh
  curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install --token <install token>
  ```

- The command fetches the install script over HTTPS from the download site,
  and runs it as root.
- The copied command registers with the control plane that issued it.
- A command for another environment supplies that environment's agent
  endpoint from trusted runtime configuration, never from the request.
- The production command keeps the form above.
- [Agent endpoint](agent-endpoint.md#install) says which control plane an
  install registers with, and what it does on a registered machine.
- The install script installs the agent, then runs the agent with the
  arguments that the command gives the script.
- So the agent runs `install` with the install token, and the machine needs
  no second command.
- **Decision:** the install script runs the agent with the arguments of the
  script. Rejected: a second command that runs the agent after the
  install. Reason: the install command stays one line.

## What install prepares

- Install prepares Firecracker, its jailer, and the required host tools.
- Install prepares a persistent reflink disk area in one XFS file on any
  filesystem.
- The area holds the [job disk](job-routing.md#terms).
- The area holds the separate [image budget](job-routing-capacity.md#disk).
- Its filesystem overhead comes from the host reserve.
- The first install finishes measured area growth in the same call.
- Repeated installation repairs missing parts and preserves machine
  identity, drain state, and job data.
- When the install command runs again on a registered machine, the machine
  keeps its registration and its place in the
  [machine count](metering.md#own-machines).
- Uninstall finishes running jobs before it removes its owned disk area.
- **Decision:** install prepares the tools and one XFS file with reflinks.
  Rejected: customer preparation, full copies without reflinks, and
  deferral until after launch. Reason: one command must prepare the machine
  without a slow full-copy path.

## The install token

- An install token belongs to one workspace.
- An install token works for one hour.
- Within its hour, an install token adds any number of machines.
- **Decision:** one install token adds any number of machines within its
  hour. Rejected: one machine for each token, an approval of each new
  machine in the dashboard, and a code that the agent prints for a person
  to enter in the dashboard, because each one adds a step for every machine.
- An Admin can revoke an install token before its hour ends.
- After the revoke, the install token adds no machine.
- The machines that registered with the token stay registered.
- **Decision:** an Admin can revoke an install token. Rejected: a token
  that always works for its whole hour, because a leaked token must not
  stay usable.

## What an own machine needs

- An own machine needs x64 Linux with KVM, 4 cores, 16 GB of memory, and
  250 GB of disk or more.
- Reason: the 140 GB image budget leaves 110 GB before the host reserve.
- The host keeps 27.5 GB, so the job disk is 82.5 GB.
- [Worked examples](job-routing-examples.md) shows the default job fit with
  the current image root.
- An own machine also needs systemd and cgroup v2.
- Its Linux distribution is Debian, Ubuntu, Fedora, RHEL, Rocky Linux, or
  AlmaLinux.
- **Decision:** own machines need systemd, cgroup v2, and one of these
  distributions. Rejected: a promise of any x64 Linux host that meets the
  hardware minimum, because customers must know these limits before they
  set up a machine.
- **Decision:** own machines run x64 Linux only. Rejected: more platforms,
  because one platform is the fastest way to ship.

## Related docs

- [Agent](agent.md): the dashboard after install, the first image, and the
  other requirements of the agent.
- [Agent endpoint](agent-endpoint.md): the control plane of an install.
- [Job routing capacity](job-routing-capacity.md): the image budget and the
  job disk.
