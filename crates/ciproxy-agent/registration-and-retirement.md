# Runner registration and retirement

The agent gives each guest one in-memory GitHub runner registration.
The [agent protocol](../../docs/protocol/agent.md#drain-and-remove) defines
which stops preserve running jobs and when GitHub retirement comes first.

## Gateway endpoint

The default route is `wss://app.ciproxy.dev/agent/`.
The install option `--agent-url` selects another WSS route with a valid TLS
certificate. Without a saved route, `CIPROXY_AGENT_URL` selects one too.
Both accept only a valid `wss://` URL with a host.
Install saves the selected route in `/var/lib/ciproxy/agent-url` with mode
`0600`. Install and the daemon prefer the saved route over the environment
setting, so an inherited setting cannot redirect the installed service.
Only an explicit install option replaces the saved route.
Uninstall removes the saved route.
Cache metadata requests use the selected route's host and port over HTTPS.
During its job drain, uninstall uses only that saved route for hosted cache
metadata. A missing or damaged route keeps local cleanup available.
The metadata relay then returns unavailable without sending a machine
credential. Cache restores keep their normal GitHub fallback.
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
