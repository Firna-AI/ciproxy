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
- The command is one line, with the install token after `--token`.
- The same command works on x64 Linux and on a Mac:

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

- On Linux, install prepares Firecracker, its jailer, and the required host
  tools.
- On Linux, install prepares a persistent reflink disk area in one XFS file
  on any filesystem.
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
- On a Mac, install prepares a launchd service for the agent, the pf rules
  of [Job isolation](security-isolation.md#job-isolation-on-a-mac), and a
  disk area in a folder on an APFS volume.
- APFS makes copy-on-write clones on every APFS volume, so a Mac needs no
  extra filesystem file.
- A Mac needs no tool other than the agent. Virtualization.framework is
  part of macOS.
- Uninstall on a Mac removes the launchd service, the pf rules, and the
  disk area, after the running jobs finish.

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

- An own machine is an x64 Linux computer or an Apple Silicon Mac.

### Linux

- An x64 Linux machine needs KVM, 4 cores, 16 GB of memory, and 250 GB of
  disk or more.
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

### Mac

- A Mac needs Apple Silicon, which is an M1 chip or later.
- Virtualization.framework runs macOS VMs only on Apple Silicon. So
  `install` refuses a Mac with an Intel processor, with a message that
  names the cause.
- A Mac needs 16 GB of memory or more. With 16 GB, the
  [host reserve](job-routing.md#slots) leaves 1 slot.
- Virtualization.framework runs no macOS guest that is newer than the macOS
  of the Mac.
- So the macOS of a Mac must be the macOS version of the
  [macOS image](agent-images.md#the-macos-image) or later.
- `install` refuses a Mac with an older macOS, with a message that names
  the version that the Mac needs.
- The minimum is macOS 26.7.3 (25G342), the macOS of the macOS image. The
  build installs Apple's newest full restore image, macOS 26.6.2, and then
  updates the guest to macOS 26.7.3.
- The pinned GitHub runner image `macos-26-arm64/20261117.0516`, which
  `macos-latest` names now, runs macOS 26.7.2. Software Update offers no
  26.7.2 now, only 26.7.3 in the macOS 26 line.
- **Decision:** the macOS image runs macOS 26.7.3 (25G342), so a Mac needs
  macOS 26.7.3 or later. Reason: Software Update offers no 26.7.2, and a
  build can reach only the update that Apple offers. The user chose this
  on 22 November 2026. Rejected: a pin of the macOS 26 line alone, with the
  reached build in the manifest, because builds would then differ without
  a change of the pin.
- When a Mac cannot run the image that it has, its
  [virtualization check](agent.md#the-virtualization-check) reports
  `operating_system_too_old`.
- A Mac needs about 176 GB of free disk. The default area volume takes
  three quarters of the free space, and it needs at least 132 GB: the
  100 GB image budget and the job disk of one default job. A Mac with
  160 GB free sets `CIPROXY_AREA_BYTES` to 132 GB or more, or keeps the
  area on another disk.
- With 176 to 200 GB free, a Mac runs one default macOS job at a time. Its
  16 GB of memory also gives it 1 slot.
- A Mac must stay awake. The agent prevents system sleep while it runs.
- Other apps can also run macOS VMs. Each such VM counts against the limit
  of 2 macOS VMs on the Mac, as [Platforms](agent.md#platforms) says.
- In place of the startup disk, the disk area can be on an external APFS
  disk. The agent then needs Full Disk Access, as
  [Area on another disk](../../crates/ciproxy-agent/macos-host.md#area-on-another-disk)
  says.
- A Mac also needs outbound HTTPS to
  `785233830ca9dfa2eb7c3561dd52f675.r2.cloudflarestorage.com`, which serves
  the macOS image.
- **Decision:** own machines run x64 Linux or macOS on Apple Silicon. This
  replaces the decision of x64 Linux only. Reason: the user chose macOS
  jobs on the customer's own Macs on 21 November 2026. Rejected: Linux
  arm64 jobs on Macs.

## Related docs

- [Agent](agent.md): the dashboard after install, the first image, and the
  other requirements of the agent.
- [Agent endpoint](agent-endpoint.md): the control plane of an install.
- [Job routing capacity](job-routing-capacity.md): the image budget and the
  job disk.
