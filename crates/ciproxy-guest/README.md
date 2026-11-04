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
Start the stock listener with the JIT registration and `--once`.
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

The stock runner does not expose a trusted GitHub job ID on stdout.
The guest therefore does not invent `JobStarted` values.
Cache facts and Docker disk commands need the later cache integration.
This guest warns if those commands arrive.

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
