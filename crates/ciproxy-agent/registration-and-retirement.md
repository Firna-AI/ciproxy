# Runner registration and retirement

The agent gives each guest one in-memory GitHub runner registration.
The [agent protocol](../../docs/protocol/agent.md#drain-and-remove) defines
which stops preserve running jobs and when GitHub retirement comes first.

## Gateway endpoint

The [agent endpoint protocol](../../docs/protocol/agent-endpoint.md)
defines which control plane receives the install token and the machine
credential.

The default endpoint is `wss://app.ciproxy.dev/agent/`.
Install selects its endpoint from its command only. The install option
`--agent-url` comes first, then `CIPROXY_AGENT_URL`, then the default.
Both accept only a valid `wss://` URL with a host. The endpoint also needs
a valid TLS certificate. Install never reads saved state for this choice.

A machine without an identity registers at the selected endpoint.
The agent saves that endpoint in the identity, with the workspace ID, the
machine ID, and the machine credential. One atomic write of
`/var/lib/ciproxy/state.json`, with mode `0600`, saves all four values.
The daemon, the metadata relay, and uninstall take the endpoint for the
machine credential only from that identity. So an inherited setting or a
later install option cannot redirect the credential.
Cache metadata requests use the host and port of that endpoint over HTTPS.

A reinstall on a registered machine keeps the identity and its endpoint.
It sends no install token. It repairs the machine and succeeds.
When the selected endpoint differs, the last output line names the kept
endpoint and the selected endpoint. It says to uninstall first to move the
machine.
A remote removal clears the identity and its endpoint after the jobs end.
The next install then registers at the endpoint of its command.
Uninstall removes the state file and any old endpoint file.

Older agents saved the endpoint apart, in `/var/lib/ciproxy/agent-url`.
Before the agent builds a credential client, each identity load copies a
valid endpoint from that file into an identity without one.
Run claims `/var/lib/ciproxy/images.lock` before it loads identity state.
Image recovery keeps that same claim after workload area verification.
A rejected duplicate start changes no identity state or legacy endpoint file.
Run and uninstall save the copied endpoint in the identity.
Install inspects the identity without a service stop. That inspection
changes no identity on disk. Install uses the copy in memory for the
metadata relay and its last output line. At its end, a successful repair
still rewrites the service unit and restarts the service. Separate VM units
keep running jobs alive through that restart.

The agent keeps the file while the file names the identity endpoint.
An older agent can save the identity without its endpoint field.
A service stop does not end an older agent that a person started directly.
An older binary can also start later. An operator can also restore an
older state backup. After each of these, the next identity load copies the
endpoint from the file again. The agent removes the file only at uninstall,
when the identity ends or is absent, or when the file names another
endpoint.

A missing or unusable file leaves the identity without an endpoint.
That identity authorizes no credential traffic. The daemon keeps its jobs
without a connection, and uninstall keeps local cleanup available.
The metadata relay then returns unavailable without sending a machine
credential. Cache restores keep their normal GitHub fallback.
A reinstall reports the kept endpoint as unavailable.

The gateway authenticates Hello before Welcome.
An authenticated policy close with `[gateway] authentication failed`
revokes the local machine. Other failures reconnect and preserve its VMs.

## Registration delivery and retirement

The hosted runtime stores runner placement, GitHub IDs, and command IDs.
It removes an idle GitHub registration before StopRunner.
Drain disables admission while that removal runs.
A busy removal result preserves the job VM.
Remove and local uninstall still use immediate idle shutdown.
A job assigned before the guest's trusted handoff can race that direct path.
The user selected immediate stop for that unobserved-job window.
An App removal applies the same exception to that organization only.

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
After an authorized stop, the agent directly retires a guest with a proven
lost registration. It ends the VM and returns its capacity without a guest
notice. Running jobs remain protected. A confirmed registration keeps the
guest shutdown path.
Registration delivery and idle retirement claim the VM under one state
lock. A stop claim refuses a later registration retry. A delivery claim
keeps a concurrent stop on the guest path until delivery finishes.
Each stop uses current connection proof when saved loss proof is absent.
Confirmed loss survives restart and a failed host stop. A new delivery or
guest receipt clears it. Unknown connection state does not clear it.
