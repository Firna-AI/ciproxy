# ciproxy-agent

The Apache-2.0 agent runs on the x64 Linux machines of a customer.
It runs one GitHub Actions runner in each fresh Firecracker VM.
It depends only on `ciproxy-protocol` inside this workspace.
It contains no hosted implementation or embedded credential.

## Responsibilities

Own machine identity, hardware reports, VM isolation, and agent updates.
Keep running VMs across control plane failures and agent restarts.
Keep job registrations out of VM files, command arguments, and logs.

The control plane chooses runners and supplies authenticated job facts.
The guest image owns Docker, the GitHub runner, and the guest program.
The cache service owns cache contents, scope enforcement, and eviction.
This crate does not implement those components.

## What This Crate Does

`ciproxy-agent install --token TOKEN` registers once and installs systemd.
A reinstall preserves the machine identity and its drain state.
`ciproxy-agent run` starts the daemon.
`ciproxy-agent uninstall` stops the daemon, finishes jobs, and removes it.
With no recorded VM, uninstall removes its owned area even after an
interrupted install or a lost mount.
Uninstall reclaims its image downloads and binary staging files.
Uninstall does not revoke the registration in the hosted control plane.
Use Remove in the dashboard to revoke the machine credential.

Fresh registration sends the install token and the initial machine report.
The gateway returns the workspace ID, machine ID, and machine credential.
The agent saves these values in its root-only identity state.

The shared release installer also needs its approved argument-forwarding
step. It must invoke the installed binary with the script arguments.

The daemon sends a full hello and reports every ten seconds.
Report waits and delivery do not wait for command work.
Reconciliation requests coalesce while the worker is busy.
It reports CPU, threads, memory, KVM, disk space, slots, VMs, and images.
Missing KVM gives zero slots. It does not reject registration.
Connection failures keep running jobs and warm runners.
Reconnect waits grow from one second to sixty seconds.
Drain blocks new runners and waits for confirmed idle stops.
Resume permits new runners. Remove is permanent until a new install.
The daemon stops after the jobs of a removed machine finish.
Runtime mode actions use one locked owner. Removal remains terminal.
A late Drain remains applied and cannot undo removal.
Reconciliation enforces current mode after each wait.

State lives in `/var/lib/ciproxy/state.json` with mode `0600`.
Its parent directory has mode `0700`.
State contains the machine credential and VM ownership.
It also keeps pending network indices and failed-start cleanup intent.
Network cleanup keeps ownership after VM and disk removal.
It does not keep a completed VM in reports or reserve its slots.
It contains no one-use runner registration or install token.
Each VM record keeps the original cache rights of its job, and the scheme,
host, and port of its cache and results URLs.
It keeps no runtime token, URL path, or query.
State without that record fails to decode. No old-format adapter exists.
A saved seal without its host-decision marker also fails to decode.
Each VM has a generated local ID and a separate systemd unit.
The agent service does not own those units.
Stopping the service also stops its preparation helpers.

Each VM gets one writable root disk with the image and free job space.
The label disk means free bytes available to the runner at job start.
`root_sizing::RootSizing` defines the allocation for every caller.
It adds the image and ext4 growth allowance to the free label bytes.
The [image budget and root sizing guide](image-budget.md) defines its
formula, image geometry, and deployment metadata contract.
Docker shares the root's free space and cached Docker images.
VM roots and disposable Docker disks use one reflink copy operation.
The copy requires block sharing and keeps sparse holes.
It extends only the copy to the admitted total and runs `resize2fs` there.
It restores the exact byte length after ext4 rounds to whole blocks.
The extra tail remains sparse. It never shrinks or changes the base image.
The existing capacity error applies only when the total cannot hold the image.
The agent discards the used root after the runner exits.
VMs use the newest verified local image. Existing VMs keep their image.
The image owner holds a selected base and its exact metadata through
preparation. Admission sizes that held release before it reserves disk.
It reduces cache space from the actual allocation.
It reclaims obsolete bases after publication and after a hold ends.
Running VMs keep independent root and kernel copies.
Runtime recovery reclaims abandoned downloads and interrupted removals.
The store renames cache entries before it deletes their contents.
An exclusive runtime lock protects this image folder.
The install CLI only reads the index and never runs destructive recovery.
Each VM reserves its complete root allocation, including a warm VM.
The default image budget reserves 140 GB before the remaining disk splits.
The host keeps one quarter of that remainder, and jobs get three quarters.
The [image budget](image-budget.md) limits physical image/support allocation.
The XFS area also includes measured overhead from the host reserve.
Admission and cache budgets use the shared protocol Resources API.
Filesystem free bytes are a health measurement.

A separate network namespace permits public IPv4 egress.
It blocks IPv6, private networks, metadata addresses, and host access.
The only host exception is this VM's cache address on TCP port 7887.
The agent opens no inbound TCP port.
The agent owns the `cpxN` namespace, root `cpxN` and `cpxNp` links, and
`inet cpxN` firewall table for each allocated network index `N`.
It also reclaims orphan resources with these reserved names.
Allocation skips active, pending, and occupied host indices.
An overlapping address blocks allocation without granting cleanup ownership.
Reconciliation checks cleanup results and retains ownership until absence
is confirmed. Remove and uninstall wait for that cleanup.
One Controller operation owns terminal completion for startup, the daemon,
and uninstall. It removes the saved identity only after checked cleanup.
Uninstall also finishes area, runtime tools, and service cleanup first.
Removed startup can finish saved jobs without a machine credential.

### Host requirements

Use root on x64 Linux with systemd and cgroup v2.
The documented minimum is four cores, 16 GB of memory, and 250 GB of disk.
The default budget leaves 82.5 GB for jobs and 27.5 GB for the host.
The measured image root has 60,263,759,872 bytes.
A default VM reserves 80,899,612,672 bytes.
That includes its 20 GB free job disk and fits once on the minimum machine.
Enable KVM to run VMs.
Install prepares the pinned Firecracker and jailer pair and host packages.
It creates one persistent sparse XFS file with reflinks on any filesystem.
Images, VM disks, Docker disks, and the cache directory use this area.
Allow outbound HTTPS and public IPv4 forwarding.
Existing host firewall rules must also permit this forwarding.
The [host preparation guide](host-preparation.md) gives distribution support,
version and SHA-256 pins, storage ownership, repairs, and smoke commands.
Missing KVM still permits registration with zero slots.

### Gateway, image, and cache integration

The default route is `wss://app.ciproxy.dev/agent/`.
`CIPROXY_AGENT_URL` selects another WSS route with a valid TLS certificate.
The gateway authenticates Hello before Welcome.
An authenticated policy close with `[gateway] authentication failed`
revokes the local machine. Other failures reconnect and preserve its VMs.

The hosted runtime stores runner placement, GitHub IDs, and command IDs.
It removes an idle GitHub registration before StopRunner.
Drain disables admission while that removal runs.
A busy removal result preserves the job VM.
Remove and local uninstall still use immediate idle shutdown.
A job assigned before the guest's trusted handoff can race that direct path.
The user selected immediate stop for that unobserved-job window.
An App removal applies the same exception to that organization only.

The agent verifies the release byte count and SHA-256 before extraction.
The released `.tar.zst` bundle contains the root, kernel, and five metadata
files. Extraction rejects other paths, links, and duplicate members.
The root remains a fresh label-sized reflink of `rootfs.ext4`.
The image's Docker data remains available on that root.
The declared root length comes from the same release manifest.
The agent checks it against the extracted root member before publication.
The persisted image index and each hold retain exact release metadata.
`RunnerSpec.size` carries the original free-size request.
`RunnerReport.requested_size` retains that request across restart.
`RunnerReport.size` carries the actual total VM allocation.
A matching retry keeps its original request, image, and allocation.
Image reports list exact metadata and identify the newest local release.
These are required new formats. No old-format adapter is supplied.
A dormant drive slot permits a later checked Docker disk switch.
The guest and agent use vsock port 5252.
The agent configures a runner only for a guest Hello of handoff format 2.
The guest repeats Hello and its current assignment after reconnect.
A late readiness event cannot replace an assigned job.
Registrations and runtime credentials stay in memory.
The agent keeps each JIT configuration in memory until the guest confirms
receipt through `RunnerConfigured` or its reconnect Hello. Before that
confirmation, reconnect sends the configuration again.
A duplicate delivery starts no second runner.
The guest Hello carries `configuration_received`. A runner report sets
`configuration_lost` only when that value is false and this agent process
holds no pending JIT. An in-flight delivery is therefore not lost.

The [cache and Docker guide](cache-and-docker.md) gives the runner handoff,
the machine cache service and its listener rules, cache budgets and
recovery, and Docker leases.

Machine reports carry the actual total job disk from the persisted image
budget. This value does not depend on a working cache service.
A machine without a report has no known disk capacity.
The hosted runtime does not guess an image or job-disk size.

### Agent updates and releases

The [updates and releases guide](updates-and-releases.md) gives the
hourly updater, public package format, and glibc release checks.

## Quick Start

Run these commands from the repository root:

```sh
cargo run --package ciproxy-agent -- --help
cargo run --package ciproxy-agent -- install --help
sudo /usr/local/bin/ciproxy-agent run
sudo /usr/local/bin/ciproxy-agent uninstall
```

Run the daemon only after the gateway completes machine registration.
Do not create or edit credential state by hand.

## Development

```sh
cargo test --package ciproxy-agent
cargo fmt --all -- --check
cargo clippy --package ciproxy-agent --features filesystem-regression \
  --all-targets -- -D warnings
cargo build --package ciproxy-agent
```

Unit tests use Unimock and deterministic barriers.
They use no host filesystem, subprocess, network, or real timer.
The clippy command also compiles the opt-in filesystem regression targets.
It needs no root access and does not run those targets.
The default gate does not compile them because it leaves their feature off.
Run this command after changing these files or their dependencies:

- `tests/root_disk_regression.rs` uses path includes for `src/error.rs`,
  `src/process.rs`, and `src/vm_root.rs`.
- `tests/storage_regression.rs` uses path includes for `src/budget_file.rs`,
  `src/error.rs`, `src/host_system.rs`, `src/image_space.rs`,
  `src/sparse_copy.rs`, and `src/storage_sizing.rs`.

Path includes compile separate copies of the production modules.
Keep each target's module list aligned when a production dependency moves.
The default Docker regression target also uses path includes.
The gate compiles that target because it needs no feature.
Real VM and GitHub checks use separate smoke scripts.
The [real root regression](image-budget.md#disposable-root-growth-regression)
checks free bytes as runner UID 1001 on two image sizes.
The [development guide](development.md) gives Docker integration checks,
KVM and GitHub smoke checks, and release packaging commands.

### Key Code

- [src/application.rs](src/application.rs) composes the Linux adapters.
- [src/startup.rs](src/startup.rs) checks local completion before workload
  recovery and permits a removed machine to drain without a credential.
- [src/controller.rs](src/controller.rs) defines durable machine control.
  `Agent::complete_removal` checks networks and finalizes local cleanup and
  identity without a credential.
- [src/runtime.rs](src/runtime.rs) owns daemon inputs and completion events.
- [src/connection.rs](src/connection.rs) separates connection failures from
  credential rejection after Hello on the current socket.
- [src/runtime_reports.rs](src/runtime_reports.rs) schedules independent
  reports.
- [src/mode.rs](src/mode.rs) owns terminal runtime mode transitions.
- [src/vms.rs](src/vms.rs) owns separate jailed VM units.
- [src/network.rs](src/network.rs) allocates host-free network indices and
  verifies that cleanup removed their resources.
- [src/network_lifecycle.rs](src/network_lifecycle.rs) keeps active and
  pending network ownership across retirement, restart, and uninstall.
- [src/root_sizing.rs](src/root_sizing.rs) defines the pure shared root
  allocation rule for the agent, scaler, and hosted image configuration.
- [src/vm_root.rs](src/vm_root.rs) reflinks and grows the disposable job root.
- [src/process.rs](src/process.rs) runs host tools and shares the required
  reflink copy operation for VM roots and disposable Docker disks.
- [src/cache_authority.rs](src/cache_authority.rs) keeps the original cache
  rights and URL origins of each job.
- [src/job_decisions.rs](src/job_decisions.rs) saves each GitHub-only
  decision before it releases a waiting Worker.
- [src/job_preparation.rs](src/job_preparation.rs) prepares checked local
  cache and Docker resources once the host has the trusted job facts.
- [src/cache_unit.rs](src/cache_unit.rs) writes the cache service unit and
  its private listener rules.
- [src/cache_install.rs](src/cache_install.rs) owns private install files
  and checked cache release work after protected activation.
- [src/docker.rs](src/docker.rs) owns Docker leases, attachment, and release.
- [src/docker_recovery.rs](src/docker_recovery.rs) discards safely closed
  failed attachments and keeps their byte reservations through job exit.
- [src/vm_config.rs](src/vm_config.rs) owns the root and dormant Docker drives.
- [src/vsock_stream.rs](src/vsock_stream.rs) keeps each registration in
  memory until the guest confirms receipt.
- [src/images.rs](src/images.rs) coordinates checked image fetches.
- [src/image_registry.rs](src/image_registry.rs) owns holds and reclamation.
- [src/image_store.rs](src/image_store.rs) defines the mockable cache boundary.
- [src/image_files.rs](src/image_files.rs) owns Linux image files and recovery.
- [src/image_verification.rs](src/image_verification.rs) seals checked values.
- [src/capacity.rs](src/capacity.rs) selects shared machine resources.
- [src/host_preparation.rs](src/host_preparation.rs) owns host setup and
  uninstall entry points. `Uninstall::start` selects direct guarded cleanup
  or VM recovery and drain. `Uninstall::finish` removes the drained area.
- [src/area.rs](src/area.rs) owns area preparation, repair, and removal.
- [scripts/](scripts/) contains smoke checks and network regression tests.

### Related Docs

- [Shared protocol](../ciproxy-protocol/README.md) owns wire types.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Agent](../../docs/protocol/agent.md) defines machine behavior.
- [Job routing](../../docs/protocol/job-routing.md) defines VM resources.
- [Labels](../../docs/protocol/labels.md) defines the sizes of job VMs.
- [Security](../../docs/protocol/security.md) defines isolation and key custody.
- [Cache](../../docs/protocol/cache.md) defines cache integration.
