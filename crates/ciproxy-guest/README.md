# ciproxy-guest

Run one GitHub Actions runner in a fresh ciproxy job image.
The binary runs inside the job VM or Boat sandbox.
The own-machine agent uses `ciproxy-protocol` to configure it.
This crate has the Apache-2.0 licence in [LICENSE](LICENSE).

## Responsibilities

Supervise one stock runner and report its readiness and process exit.
Keep runner lifetime separate from the agent connection.
Use no hosted workspace crate.
The only workspace dependency is `ciproxy-protocol`.

## What This Crate Does

In `vsock` mode, listen on the configured guest port.
Exchange newline-delimited JSON using `AgentToGuest` and `GuestToAgent`.
Send `Hello`, then wait for `ConfigureRunner` or `Shutdown`.
Start the listener with the JIT environment value and `--once`.
Report `RunnerReady` after its `Listening for Jobs` message.
Report `RunnerExited` with its actual process exit code.
An exit without a process code uses `null`.
Power off the own-machine VM after normal or failed execution.
The image's systemd unit also powers off after initialization fails.

Handle `Shutdown` while the runner waits.
Ignore a second configuration because each guest runs one runner.
An agent connection failure leaves an active runner running.
Failed observations produce a warning and do not stop that runner.
Runner stdout supplies readiness and is then discarded.
Runner stderr goes to `/dev/null`.
Neither pipe can block the child because the guest stopped reading it.

In `boat` mode, consume a private registration file.
Remove the file before starting the runner.
POST only the existing `GuestToAgent::RunnerExited` value.
Use the file's HTTPS URL and its sandbox-scoped bearer token.
Disable HTTP redirects to keep that token at its intended endpoint.
The control plane must validate the token and delete the sandbox.
If the report fails, the Boat lifetime remains the backstop.
The Boat wrapper ends its private namespace after the guest ends.
It retains Boat's provider kernel and provider services.

The pinned runner source hook reads the original assignment before action
setup. It sends the job UUID, original runtime credentials, and effective
cache rights over a private Unix socket. A separate task owns that read.
Cancelling a supervisor wait cannot discard the assignment.
The guest passes these values over vsock only.
It derives the proxy URL from Docker's actual bridge.
The host returns cache-session and checked Docker attachment results.
The guest and worker exchange a preparation acknowledgement before local
setup begins. The worker waits for the final ready or fallback reply.
Proxy configuration goes through stdin and stays in memory.
The guest replays the original assignment after an agent reconnect.
A late readiness message cannot change that state to warm.
Replays replace a selected proxy's machine credential through its private pipe.
The proxy acknowledges each replacement before the guest accepts it.
Failed replacements can retry. Docker preparation runs only once.
An initial choice of direct GitHub endpoints stays in force for that job.

The entry mode selects the byte-identical stock Worker assembly for Boat.
Its original cache and results endpoints therefore remain unchanged.
The direct shutdown path can still race an assignment before this handoff.
The user selected that immediate-stop exception for machine removal,
uninstall, and last-App removal. Ordinary retirement keeps GitHub-first order.

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
The [image guide](../../image/README.md) gives the Boat file format and
the build, publication, and live smoke commands.
No registration or control-plane token belongs in an image or snapshot.

## Development

```sh
cargo test --package ciproxy-guest
cargo fmt --all -- --check
cargo clippy --package ciproxy-guest --all-targets -- -D warnings
```

The unit tests use `unimock` at each runtime boundary.
The Shutdown test waits for an actual pending runner future.
It uses a notification, with no sleep or wall-clock assertion.
Live vsock, Docker, GitHub, and Boat checks need external infrastructure.

### Key Code

- [src/boundary.rs](src/boundary.rs) defines command, process, and power
  interfaces.
- [src/service.rs](src/service.rs) supervises the single runner lifecycle.
- [src/vsock.rs](src/vsock.rs) carries cancellation-safe JSON lines.
- [src/boat.rs](src/boat.rs) consumes the private file and sends exit reports.
- [src/assignment.rs](src/assignment.rs) owns the cancellation-safe handoff.
- [src/handoff.rs](src/handoff.rs) prepares the in-memory proxy.
- [src/process.rs](src/process.rs) drains the listener and controls its group.
- [src/cli.rs](src/cli.rs) composes the selected Linux runtime.
- [src/_tests_/service_tests.rs](src/_tests_/service_tests.rs) protects the
  lifecycle requirements.

### Related Docs

- [Image guide](../../image/README.md) defines both packaging targets.
- [Protocol crate](../ciproxy-protocol/README.md) owns the JSON message types.
- [Agent protocol](../../docs/protocol/agent.md) defines job VMs and images.
- [Overflow protocol](../../docs/protocol/overflow.md) defines sandbox cleanup.
- [Architecture](../../docs/architecture.md) defines the open-source boundary.
