# ciproxy-gateway

The hosted machine gateway for ciproxy. The control plane uses this library
to register own machines, retain their latest health, and send commands.
This crate has no open licence. The public agent and protocol crates do
not depend on it.

## Responsibilities

Authenticate install tokens and machine credentials. Require the workspace
Admin role for token and machine management. Keep desired machine settings
apart from reported machine state. Serve the agent WebSocket route.

## What This Crate Does

An install token registers any number of machines in one workspace. It
expires exactly one hour after issuance. An Admin can revoke it early.
Revocation leaves existing machine credentials valid. The gateway stores
SHA-256 hashes of independent 256-bit secrets. It returns plaintext only
to the intended caller or agent. Diagnostics hide secret values.

`Register` receives `Registered` and continues on the same authenticated
WebSocket. `Registered` supplies the workspace, stable machine identity,
and credential. The agent must save that credential privately. A later
connection or reinstall uses `Hello` and receives `Welcome`. It preserves
the machine identity, warm count, and desired drain setting. An agent that
lost its credential registers a new machine. Registration accepts a
machine without KVM or slots.

Both handshakes accept the current and previous integer protocol versions.
The welcome uses the accepted version. A reconnect also receives `Drain`
or `Resume` when the desired setting needs reconciliation. Each connection
replaces its predecessor. The database fences reports with a session ID.
The transport checks that fence before it sends ordinary commands.

Reports retain all hardware, slot, runner, cache, Docker disk, and image
state. Reports and lifecycle messages update contact time with the gateway
clock. At 60 seconds without
contact, machine reads show offline and retain the last report. Closing a
socket does not clear health or mark the machine offline early.

Drain and resume persist desired state before command delivery. Agent
reports cannot change that setting. Removal revokes credentials before it
sends `Remove`. This also works without a connected agent. `Remove` tells
the agent to stop warm runners, finish running jobs, then stop. Removal
does not issue stop commands for running jobs.

A failed `Hello` closes with WebSocket code 1008 and the fixed reason
`[gateway] authentication failed`. The agent must treat this as permanent
credential rejection. An offline removed agent learns revocation through
this response. It must stop warm runners and finish active jobs, as with
`Remove`. A connected agent whose credentials another service revoked gets
this close on its next contact. A caller that needs immediate removal uses
`manage` with `MachineAction::Remove` on the gateway owner.
Unsupported versions use code 1008 and the fixed reason
`[gateway] unsupported protocol version`. A replaced connection uses code
1008 and `[gateway] connection replaced`. Frame contents never enter logs.

### Integration boundary

1. Open the PostgreSQL store through `PgStoreFactory`. It applies the
   gateway migration. Supply its `Arc<dyn GatewayStore>` to `Gateway`.
2. Supply trusted `Authorization`, `Clock`, `Entropy`, `Connections`, and
   `AgentEvents` implementations. Authorization checks the workspace Admin
   role. It performs no GitHub runner-right check.
3. Use `SystemClock`, `SystemEntropy`, and `LocalConnections` at the runtime
   composition root. Use one `Arc<dyn MachineGateway>` for all callers.
4. Construct `WebSocketTransport` with that gateway and the same
   connections. Consume it through `dyn AgentTransport`. Merge its router
   into the control-plane router. TLS ingress serves `/agent/` over HTTPS.
5. Run synchronous gateway APIs from blocking workers. The WebSocket
   adapter does this for its own calls. The API checks the store on every
   report and before each ordinary command delivery.
6. Send Admin actions through `manage`. Send trusted lifecycle commands
   through `send`. A `Delivery::Offline` result means no local socket took
   the command. Drain and removal still remain in the database.
7. Handle typed runner events and command results through `AgentEvents`.
   Check each event's organization and runner ownership in that service.
   The callback can send a command through the same gateway. It runs
   without the gateway operation lock.

Route each machine's command callers and socket to the same gateway owner.
The local mailboxes provide in-process routing. The database fences stale
connections across owners. Distributed command forwarding belongs to
runtime composition. The later agent service owns VM work and updates.
The later lifecycle service owns GitHub registration removal and job facts.
Machine reads require a workspace already authorized by the HTTP service.

## Quick Start

Run commands from the repository root with the test PostgreSQL configured:

```sh
export DATABASE_URL=postgresql://ciproxy:ciproxy@127.0.0.1:5432/ciproxy_test
cargo build --package ciproxy-gateway
cargo test --package ciproxy-gateway
```

The WebSocket contract starts the actual route on an ephemeral loopback
port. Its fake agent registers, reconnects, reports, receives commands, and
observes revocation. It needs no external credential. Each run creates and
removes an isolated database. The PostgreSQL user needs CREATEDB rights.

## Development

```sh
cargo test --package ciproxy-gateway --lib
cargo test --package ciproxy-gateway --test websocket_tests
cargo test --package ciproxy-store-pg gateway_tests
cargo fmt --all -- --check
cargo clippy --package ciproxy-gateway --all-targets -- -D warnings
```

### Key Code

- [src/gateway.rs](src/gateway.rs) defines `MachineGateway` and its service.
- [src/boundary.rs](src/boundary.rs) defines authority, time, and events.
- [src/transport.rs](src/transport.rs) serves the agent WebSocket route.
- [src/connections.rs](src/connections.rs) routes local commands by session.
- [src/types.rs](src/types.rs) defines identities and machine actions.
- [tests/websocket](tests/websocket) tests the complete transport contract.

### Related Docs

- [Architecture](../../docs/architecture.md) defines the hosted boundaries.
- [Agent](../../docs/protocol/agent.md) defines install, health, and removal.
- [Security](../../docs/protocol/security.md) defines roles and key custody.
- [Workspaces](../../docs/protocol/workspaces.md) defines machine ownership.
- [Store interface](../ciproxy-store-interface/README.md) owns atomic queries.
- [Protocol](../ciproxy-protocol/README.md) owns all agent wire messages.
