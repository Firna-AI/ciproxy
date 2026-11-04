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
Uninstall reclaims its image downloads and binary staging files.
Uninstall does not revoke the registration in the hosted control plane.
Use Remove in the dashboard to revoke the machine credential.

**Fresh registration needs a gateway contract.** The protocol requires a
workspace ID, but the opaque install token has no defined workspace
resolver. The CLI reports this condition before registration.
Reinstall of an existing machine works without that resolver.
The shared release installer also needs its approved argument-forwarding
step. It must invoke the installed binary with the script arguments.

The daemon sends a full hello and reports every ten seconds.
Report waits and delivery do not wait for command work.
Reconciliation requests coalesce while the worker is busy.
It reports CPU, threads, memory, KVM, disk space, slots, VMs, and images.
Missing KVM gives zero slots. It does not reject registration.
Connection failures keep running jobs and warm runners.
Reconnect waits grow from one second to sixty seconds.
Drain requests idle shutdown and preserves running jobs.
Resume permits new runners. Remove is permanent until a new install.
The daemon stops after the jobs of a removed machine finish.
Runtime mode actions use one locked owner. Removal remains terminal.
A late Drain remains applied and cannot undo removal.
Reconciliation enforces current mode after each wait.

State lives in `/var/lib/ciproxy/state.json` with mode `0600`.
Its parent directory has mode `0700`.
State contains the machine credential and VM ownership.
It contains no one-use runner registration or install token.
Each VM has a generated local ID and a separate systemd unit.
The agent service does not own those units.
Stopping the service also stops its preparation helpers.

Each VM gets a reflink root disk and a new, formatted Docker disk.
The two disks together use the disk size of the runner label.
The Docker disk gets the label size minus the root image size.
The agent discards both disks after the runner exits.
VMs use the newest verified local image. Existing VMs keep their image.
The image owner holds a selected base until VM preparation ends.
It reclaims obsolete bases after publication and after a hold ends.
Running VMs keep independent root and kernel copies.
Runtime recovery reclaims abandoned downloads and interrupted removals.
The store renames cache entries before it deletes their contents.
An exclusive runtime lock protects this image folder.
The install CLI only reads the index and never runs destructive recovery.
Each VM reserves its full label disk, including a warm VM.
The host and images keep one quarter of disk capacity.
Admission and cache budgets use the shared protocol Resources API.
Filesystem free bytes are a health measurement.

A separate network namespace permits public IPv4 egress.
It blocks IPv6, private networks, metadata addresses, and host access.
The only host exception is this VM's cache address on TCP port 7887.
The agent opens no inbound TCP port.

### Host requirements

Use root on x64 Linux with systemd and cgroup v2.
The documented minimum is four cores, 16 GB of memory, and 80 GB of disk.
Enable KVM to run VMs.
Install a matching, static Firecracker and jailer pair as
`/usr/local/bin/firecracker` and `/usr/local/bin/jailer`.
Install `ip`, `nft`, `sysctl`, GNU `cp`, and `mkfs.ext4`.
Store `/var/lib/ciproxy` on a filesystem with reflink support.
Allow outbound HTTPS and public IPv4 forwarding.
Existing host firewall rules must also permit this forwarding.
Install does not provision these host tools or the reflink filesystem.
Installation on a host with only the documented hardware remains unfinished.

### Gateway, image, and cache integration

The current image and guest need alignment before live smoke checks.
The image producer uses a zstd bundle with metadata and a 60 GB root disk.
Its guest uses vsock port 5252, accepts one connection, and does not emit
`JobStarted`. It stops the runner on `Shutdown`.
These behaviors do not yet meet the agent contract below.
The guest must reconnect, report current job state, and preserve a job
before it acknowledges shutdown. Those components need a separate change.

The default route is `wss://app.ciproxy.dev/agent/`.
`CIPROXY_AGENT_URL` can select a staging WSS route.
The agent requires a valid TLS certificate.
The gateway authenticates Hello before sending Welcome.
For a revoked Hello, it must send Remove before closing the connection.
An HTTP 401 or 403 during the upgrade also removes the machine.
The gateway binds every runner organization to the machine workspace.
It withdraws idle GitHub registrations before StopRunner, Drain, or Remove.
It preserves runners to which GitHub has already assigned a job.
JobFacts come from that authenticated gateway and the GitHub API.
The agent matches their organization and job ID to its current runner.
The gateway retains and replays StartRunner until RunnerReady.
A matching Starting retry delivers ConfigureRunner to the existing VM.
A retry cannot change the runner organization, label, or size.

The gateway sends FetchImage with the newest image metadata.
The HTTPS `.tar.gz` bundle contains exactly two regular files:
`vmlinux` and `rootfs.ext4` at the archive root.
The byte count and SHA-256 must match before extraction.
Only a checked download can enter extraction.
Only a complete checked bundle can become the newest image.
The rootfs is an ext4 disk smaller than the requested label disk.
The guest configures DNS and mounts `/dev/vdb` for Docker data.
The guest listens on vsock port 52 with newline-delimited protocol JSON.
It sends Hello with its image ID before runner configuration.
On reconnect, it repeats Hello and its current runner state.
It preserves a configured runner across that connection change.
ConfigureRunner is idempotent for that same runner.
A repeated registration never starts a second runner or replaces its job.
Shutdown requests are graceful. The guest must preserve an already-taken job.
The guest exits its runner before it reports RunnerExited.
The agent retains VM ownership and disk space until that report or unit exit.
A late JobStarted changes a pending shutdown to Running.
The agent retries pending shutdown requests after connection recovery.
Repeated Shutdown requests share one per-VM flag.
A guest that cannot connect does not fill the normal message queue.

The current agent expects an optional machine cache at
`/run/ciproxy-cache/control.sock`.
Root owns this socket with mode `0600`.
Requests and replies are newline-delimited JSON with a `type` field:

| Request | Fields | Reply |
| --- | --- | --- |
| `report` | None | `report` with `MachineCacheReport` |
| `budget` | `capacity_bytes` | `applied` after reclaim |
| `clear` | None | `applied` after deletion |
| `configure` | `runner_id`, `organization_id`, `address` | `configured` with a runner-scoped `token` |
| `facts` | `runner_id`, authenticated `facts` | `applied` |
| `release` | `runner_id` | `applied` |

The cache address uses the VM's host link address and TCP port 7887.
Before VM disk preparation, the agent reduces the cache budget.
The cache must reclaim space before it replies to that budget request.
The budget includes cache and unused Docker disk space.
The integrated cache component must evict unused Docker disks in LRU order.
An absent cache gives direct GitHub caching and zero observed cache use.
An absent cache makes ClearCache fail. It cannot confirm deletion.
The merged [cache sidecar](../ciproxy-cache-sidecar/README.md#agent-integration)
instead exposes HTTP administration and runtime-session registration.
The agent does not yet start or configure that service.
Own-machine integration must align these control and credential boundaries.
Cache-sidecar wiring and sticky Docker disk ownership remain integration work.
Persistent Docker disk reports stay zero until that ownership is implemented.

### Agent updates and releases

The daemon checks for a new agent version every hour with `cli-updater`.
The default release server is `https://get.ciproxy.dev`.
`CIPROXY_RELEASE_SERVER_URL` can select a staging HTTPS release server.
The update requires a checksum and restarts only the agent unit.
A failed update cycle keeps the next hourly check.
The updater source is pinned in the workspace manifest.
`RUST_LOG` selects logging verbosity. JSON operation events remain available.
An outer safety filter always excludes `tungstenite` and `tokio_tungstenite`.
This filter also checks normalized targets from the log-to-tracing bridge.
User directives cannot enable transport payload diagnostics.

The systemd service reads `/etc/ciproxy-agent/environment` when present.
Keep this root-only settings file at mode `0600`.

Release-plz tags only the agent, with `ciproxy-agent-vVERSION`.
The release workflow runs the complete gate in a read-only PostgreSQL job.
Versioning requires that job to pass.
Package builds run read-only. A separate job uploads the verified assets.
Action commits and release tool versions are pinned.
Set `RELEASE_PLZ_TOKEN` to a repository-scoped token with release and pull
request write access. Its tags must trigger the archive workflow.
The release server needs access to the private release assets.
It uses binary `ciproxy-agent` and tag prefix `ciproxy-agent-v`.

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
cargo clippy --package ciproxy-agent --all-targets -- -D warnings
cargo build --package ciproxy-agent
```

Unit tests use Unimock and deterministic barriers.
They use no host filesystem, subprocess, network, or real timer.
Real VM and GitHub checks use separate smoke scripts.

Build and package the GNU binary for glibc 2.17 or later:

```sh
python3 -m pip install cargo-zigbuild
cargo zigbuild --locked --release --package ciproxy-agent --target x86_64-unknown-linux-gnu.2.17
scripts/cli-release.sh verify-glibc target/x86_64-unknown-linux-gnu/release/ciproxy-agent 2.17
scripts/cli-release.sh package ciproxy-agent 0.1.0 x86_64-unknown-linux-gnu target/x86_64-unknown-linux-gnu/release/ciproxy-agent .context/agent-release
```

Use the version from the agent manifest.
Packaging produces the `.tar.gz` archive and its `.sha256` companion.

### KVM and GitHub smoke checks

Use a disposable private repository with the staging GitHub App installed.
Copy [the workflow fixture](scripts/github-smoke-workflow.yml) into that
repository as `.github/workflows/ciproxy-agent-smoke.yml`.
Push it to the branch that the scripts will dispatch.
The staging gateway must route its `ciproxy` jobs to this machine.
It must supply a verified image and keep a warm runner for the warm check.
The image must include the guest program and Docker.
Authenticate `gh` to dispatch and read the test workflow.
Run each check as root on the registered KVM machine:

```sh
export CIPROXY_SMOKE_REPO=owner/private-agent-smoke
export CIPROXY_SMOKE_REF=main
crates/ciproxy-agent/scripts/smoke-docker.sh
crates/ciproxy-agent/scripts/smoke-warm.sh
crates/ciproxy-agent/scripts/smoke-restart.sh
```

Optional inputs are `CIPROXY_SMOKE_WORKFLOW` and `CIPROXY_SMOKE_STATE`.
The restart check restarts the agent during a real Docker job.
It checks the same VM process and a successful job result.
It also requires a fresh accepted JobStarted observation from the new agent.
The script filters structured JSON journal events by service invocation.
The observation contains only the generated local VM ID.
The warm check proves that the runner was ready before dispatch.
It snapshots every VM ID before dispatch, including Starting VMs.
The replacement warm VM must have an ID absent from that snapshot.
An active machine that never reaches warm readiness fails the check.
The smoke host also needs journalctl.
Each check waits for the used VM and disks to disappear.
Evidence stays under `.context/ciproxy-mvp/`.
Missing resources produce `SKIP` and exit 77. Failed checks exit 1.

### Key Code

- [src/application.rs](src/application.rs) composes the Linux adapters.
- [src/controller.rs](src/controller.rs) defines durable machine control.
- [src/runtime.rs](src/runtime.rs) owns daemon inputs and task lifetimes.
- [src/runtime_reports.rs](src/runtime_reports.rs) schedules independent reports.
- [src/mode.rs](src/mode.rs) owns terminal runtime mode transitions.
- [src/vms.rs](src/vms.rs) owns separate jailed VM units.
- [src/vsock_stream.rs](src/vsock_stream.rs) delivers registrations in memory.
- [src/images.rs](src/images.rs) coordinates checked image fetches.
- [src/image_registry.rs](src/image_registry.rs) owns holds and reclamation.
- [src/image_store.rs](src/image_store.rs) defines the mockable cache boundary.
- [src/image_files.rs](src/image_files.rs) owns Linux image files and recovery.
- [src/image_verification.rs](src/image_verification.rs) seals checked values.
- [src/capacity.rs](src/capacity.rs) selects shared machine resources.
- [scripts/](scripts/) contains real KVM and GitHub smoke checks.

### Related Docs

- [Shared protocol](../ciproxy-protocol/README.md) owns wire types.
- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Agent](../../docs/protocol/agent.md) defines machine behavior.
- [Job routing](../../docs/protocol/job-routing.md) defines VM resources.
- [Labels](../../docs/protocol/labels.md) defines the sizes of job VMs.
- [Security](../../docs/protocol/security.md) defines isolation and key custody.
- [Cache](../../docs/protocol/cache.md) defines cache integration.
