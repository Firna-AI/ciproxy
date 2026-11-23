# ciproxy-guest

Run one GitHub Actions runner in a fresh ciproxy job image.
The binary runs inside the Linux job VM, the macOS job VM, or the Boat
sandbox. It builds for `x86_64-unknown-linux-gnu` and
`aarch64-apple-darwin`.
The own-machine agent uses `ciproxy-protocol` to configure it.
This crate has the Apache-2.0 licence in [LICENSE](LICENSE).

## Responsibilities

Supervise one stock runner and report its readiness and process exit.
Keep runner lifetime separate from the agent connection.
Use no hosted workspace crate.
The only workspace dependency is `ciproxy-protocol`.

## What This Crate Does

In `vsock` mode, listen on the configured guest port.
On Linux, the agent connects through Firecracker's vsock device. On macOS,
it connects through the virtio socket device of Virtualization.framework.
Both use the same port, 5252, and the same messages.
Exchange newline-delimited JSON using `AgentToGuest` and `GuestToAgent`.
Send `Hello`, then wait for `ConfigureRunner` or `Shutdown`.
`Hello` carries `configuration_received`. It is true after receipt, also in
the Hello that follows an agent reconnect.
`Hello` also carries `handoff_protocol_version`. The agent configures a
runner only for format 2.
Send `RunnerConfigured` to confirm receipt before the runner process starts.
Start the listener with the JIT environment value and `--once`.
Report `RunnerReady` after its `Listening for Jobs` message.
Report `RunnerExited` with its actual process exit code.
An exit without a process code uses `null`.
End the own-machine VM after normal or failed execution.
Use a guest reboot with the agent's `reboot=k` boot setting.
Firecracker ends its x86 process on that reset instead of booting again.
Guest `poweroff` leaves that process alive because x86 Firecracker has no
ACPI power control. See the [Firecracker shutdown contract][shutdown].
The image's systemd unit also requests shutdown after initialization fails.
A macOS guest powers off with `shutdown -h now` instead, because a reboot
starts a Virtualization.framework VM again. The agent sees the VM stop.
The [macOS start script](../../image/macos/guest/start.sh) also powers off
after the guest ends for any reason.

On Linux, the guest sets its `oom_score_adj` to `-1000` before it starts
the runner. If the write fails, the guest does not start the runner.
It uses `systemd-run --scope` with `OOMPolicy=continue` to start the runner
in a separate systemd control group. The child sets its `oom_score_adj`
back to `0` before `runuser` changes to the `runner` account. The guest
still waits for the runner and gets its exit code.
See [Out of memory](../../docs/protocol/job-usage.md#out-of-memory).
`--runner-account` names another account.
macOS has no `runuser`. There, `launchctl asuser` starts the listener in the
account's launchd domain, so per-user services such as the simulator and
the keychain are reachable. Then `sudo` changes the account. It keeps only
the JIT registration and the assignment socket value.
The guest clears its own service environment for this start.
It passes only the JIT registration and the assignment socket value.
`runuser` sets `HOME`, `USER`, `LOGNAME`, and `SHELL` from the account.
The image's [entry script](../../image/runner-entry.sh) then loads
`/etc/profile`, exports `/etc/environment`, and loads the account's own
`.profile`, in that order.
Jobs therefore keep the image tool `PATH` and the account's additions.
The [macOS entry script](../../image/macos/guest/runner-entry.sh) loads
`/etc/profile` and then the account's `.bash_profile` or `.profile`.

Handle `Shutdown` while the runner waits.
A second configuration starts no second runner, because each guest runs one
runner. A repeat for the same runner gets `RunnerConfigured` again.
An agent connection failure leaves an active runner running.
Failed observations produce a warning and do not stop that runner.
Runner stdout supplies readiness and is then discarded.
Runner stderr goes to `/dev/null`.
Neither pipe can block the child because the guest stopped reading it.

In `boat` mode, consume a private registration file.
The guest does not follow a symbolic link to the file.
The file must belong to the guest's user, with no group or other access.
Remove the file before starting the runner.
POST `GuestToAgent::RunnerExited` when the runner exits.
On Linux, also POST `GuestToAgent::UsageSamples` every 30 seconds during
the job. Each sample report contains the whole series so far.
Use the file's HTTPS URL and its sandbox-scoped bearer token.
Disable HTTP redirects to keep that token at its intended endpoint.
The control plane must validate the token. It deletes the sandbox after
the exit report, not after a sample report.
If the exit report fails, the Boat lifetime remains the backstop.
A missing file ends the guest with exit code 1 and the error
`registration file is missing`. The error names no path.

### Linux job usage

The guest samples the whole VM once per second. It reads busy CPU time
from `/proc/stat`, used memory from `/proc/meminfo`, and out-of-memory
kills from `/proc/vmstat`. In `vsock` mode, the trusted job handoff starts
sampling. In `boat` mode, the guest looks for `Runner.Worker` in the
runner's process tree. It checks once per second until it finds the worker.
The guest stops sampling when the runner exits. It sends the final
samples with the exit report when available. macOS guests do not send usage.

When the series would exceed 3,600 points, the guest merges adjacent
points. Each merged point keeps the average CPU, peak memory, and total
kill count. The last point can cover part of its stated width. After a
read gap, the guest keeps the longest series of consecutive samples.
A failed read or sample post logs a warning without figures. The runner
and its exit report do not depend on a sample or a sample post.
See [Job usage](../../docs/protocol/job-usage.md).
The guest then starts no runner and sends no report.
In a Boat sandbox, the guest runs as a service of Boat's own systemd.
The [Boat launcher](../../image/boat.md#boat-launch) starts that service
after it refreshes the restored services.
The guest leaves the end of the sandbox to the control plane and the Boat
lifetime.

Before the runner can take its one job, the guest measures the proxy URL on
Docker's actual bridge and the preloaded Docker bytes.
A macOS image has no Docker. Its steps run in the VM itself, so the proxy
listens on `127.0.0.1:8230`, and the guest reports that URL and 0 Docker
bytes. A macOS guest ignores a Docker disk attachment and reports it as not
attached.
The assignment socket is `/run/ciproxy/assignment.sock` on Linux and
`/var/run/ciproxy/assignment.sock` on macOS.
The pinned runner source hook reads the original assignment before action
setup. It sends the job UUID, original runtime credentials, and effective
cache rights over a private Unix socket, in handoff format 2.
A separate task owns that read, and the inbox keeps receiving when the hook
connects again. Cancelling a supervisor wait cannot discard the assignment.
The guest keeps the first original handoff for each retry.
The guest passes these values over vsock only.
The host answers with a `JobDecision`: `GitHubOnly`, or `MachineCache` with
the cache session and any checked Docker attachment.
The guest and worker exchange a preparation acknowledgement before local
setup begins. The worker waits for the host's decision with no time limit.
A failed local preparation reports the failure to the host, which then
returns `GitHubOnly`. The guest never decides GitHub-only by itself.
Proxy configuration goes through stdin and stays in memory.
The guest replays the original assignment after an agent reconnect.
It replays the decision and a completed preparation to a hook that
connects again. Docker preparation runs only once.
A late readiness message cannot change that state to warm.
Replays replace a selected proxy's machine credential through its private pipe.
The proxy acknowledges each replacement before the guest accepts it.
Failed replacements can retry.
A `GitHubOnly` decision stays in force for the whole job.
The guest warns and ignores preparation before configuration or for a job
that is not running in this guest.

The entry mode selects the byte-identical stock Worker assembly for Boat.
Its original cache and results endpoints therefore remain unchanged.
The direct shutdown path can still race an assignment before this handoff.
[Drain and remove](../../docs/protocol/agent.md#drain-and-remove) says which
stops use this path, and which stops wait until GitHub removes the runner.

## Quick Start

Run from the repository root:

```sh
cargo build --package ciproxy-guest
cargo run --package ciproxy-guest -- --help
```

The image starts own-machine guests with this interface:

```sh
ciproxy-guest --image-id IMAGE_ID vsock --port 5252
```

The agent must use the same guest port.
A macOS image starts the same interface from the launchd daemon
`dev.ciproxy.guest`, with `--runner-account`. The
[macOS guest guide](../../image/macos/guest/README.md) gives its install
steps.
The Boat snapshot starts its guest with this interface:

```sh
ciproxy-guest --image-id IMAGE_ID boat \
  --registration-file /run/ciproxy/registration.json
```

The [image guide](../../image/README.md) gives the build and publication
commands. The [Boat smoke guide](../../image/boat-smoke.md) gives the Boat
file format and the live smoke commands.
No registration or control-plane token belongs in an image or snapshot.

## Development

```sh
cargo test --package ciproxy-guest
cargo fmt --all -- --check
cargo clippy --package ciproxy-guest --all-targets -- -D warnings
```

The unit tests use `unimock` at each runtime boundary.
The source-hook integration test uses a real temporary Unix socket.
It checks the original credentials and the preparation and decision lines.
The Shutdown test waits for an actual pending runner future.
It uses a notification, with no sleep or wall-clock assertion.
The [runner environment check](../../image/checks.md#runner-account-environment)
starts the built binary with the real `runuser` and entry script. It uses
a stand-in for `systemd-run` and a fixture file for the score adjustments.
The [Boat launch check](../../image/checks.md#boat-launch) starts it
through the Boat launcher without a registration.
Live vsock, Docker, GitHub, and Boat checks need external infrastructure.

### Key Code

- [src/boundary.rs](src/boundary.rs) defines command, process, and power
  interfaces.
- [src/service.rs](src/service.rs) supervises the single runner lifecycle.
- [src/vsock.rs](src/vsock.rs) carries cancellation-safe JSON lines.
- [src/boat.rs](src/boat.rs) consumes the private file and posts reports.
- [src/usage/mod.rs](src/usage/mod.rs) runs the sampler for the job window.
- [src/usage/linux.rs](src/usage/linux.rs) reads Linux counters and detects
  the Boat worker.
- [src/usage/series.rs](src/usage/series.rs) bounds the series by merging
  points.
- [src/assignment.rs](src/assignment.rs) owns the cancellation-safe handoff.
- [src/guest_setup.rs](src/guest_setup.rs) measures the preloaded Docker
  facts before the runner can take its job.
- [src/runner_assignment.rs](src/runner_assignment.rs) replays the host's
  decision to each reconnected source-hook socket.
- [src/replay.rs](src/replay.rs) keeps the assignment and a completed
  preparation across a vsock reconnect.
- [src/handoff.rs](src/handoff.rs) prepares the in-memory proxy.
- [src/process.rs](src/process.rs) starts the listener as the runner
  account, drains its output, and controls its group.
- [src/power.rs](src/power.rs) requests the Firecracker reset or the macOS
  power-off, or leaves the end of a Boat sandbox to the control plane.
- [src/platform.rs](src/platform.rs) holds the paths and commands that
  differ between Linux and macOS.
- [src/cli.rs](src/cli.rs) composes the runtime of the build platform.
- [src/_tests_/service_tests.rs](src/_tests_/service_tests.rs) protects the
  lifecycle requirements.
- [tests/assignment_socket.rs](tests/assignment_socket.rs)
  checks the source-hook wire contract through the production socket path.
- [tests/boat_registration.rs](tests/boat_registration.rs) checks the safe
  error of the built binary when no Boat registration arrived.

### Related Docs

- [Image guide](../../image/README.md) defines both packaging targets.
- [Protocol crate](../ciproxy-protocol/README.md) owns the JSON message types.
- [Agent protocol](../../docs/protocol/agent.md) defines job VMs and images.
- [Sandbox end](../../docs/protocol/overflow-sandbox-end.md) defines the exit
  report and sandbox cleanup.
- [Architecture](../../docs/architecture.md) defines the open-source boundary.

[shutdown]: https://github.com/firecracker-microvm/firecracker/blob/v1.17.0/FAQ.md#how-can-i-gracefully-reboot-the-guest-how-can-i-gracefully-poweroff-the-guest
