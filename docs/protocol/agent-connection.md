# Agent connection

This doc states the requirements for the connection between the agent and
the control plane. [Agent](agent.md) holds the other requirements of the
agent.

## Terms

- **gateway:** the [machine gateway](../architecture.md#control-plane) of
  the control plane, which keeps the connections of the agents.
- **upgrade:** the HTTPS request that opens the WebSocket connection of an
  agent at its [agent endpoint](agent-endpoint.md#terms).
- **handshake:** the start of a connection. It lasts from the upgrade until
  the gateway accepts the install token or the machine credential, or until
  the connection ends.
- **error close:** a close of a connection that the gateway starts, for
  example after a rejected credential or a passed limit.
- **report:** the message in which the agent sends the full state of its
  machine.
- [Agent](agent.md#terms) defines the own machine, the agent, the download
  site, and the machine credential.

## Network

- The agent connects out to the
  [control plane](../architecture.md#components).
- The own machine opens no inbound port.
- The own machine needs outbound HTTPS to the control plane, to GitHub, and
  to the download site.

## Contact

- The agent reports to the control plane every 10 seconds.
- The dashboard shows a machine without contact for 60 seconds as offline.
- [Own machine offline](job-routing-failures.md#own-machine-offline) says
  what happens to the jobs of an offline machine.
- When the connection fails, the agent keeps its running job VMs and its
  [warm runners](job-routing.md#warm-runners).
- The agent then tries to connect again.

## Connection limits

- After the upgrade, the gateway waits at most 15 seconds for the first
  complete message of the agent.
- A message is complete when all of its frames arrived.
- When no complete message arrives in that time, the gateway ends the
  connection with an error close.
- Each instance of the gateway has at most eight handshakes in progress at a
  time.
- The limit counts the handshakes from all client addresses together.
- While eight handshakes are in progress, the gateway answers each new
  upgrade with the HTTP status 503, Service Unavailable.
- That answer opens no WebSocket connection.
- The agent can try the upgrade again later.
- Each WebSocket frame of the agent holds at most 16 MiB, that is 16,777,216
  bytes.
- Each message of the agent holds at most 16 MiB, also when it arrives in
  several frames.
- The gateway does not process a larger frame or message.
- It ends that connection with an error close.
- **Decision:** reports have the same 16 MiB limit as the other messages.
  Rejected: a limit that every valid report fits. Reason: a report holds
  strings and cache job lists without a fixed bound, so no largest valid
  report exists.

## Closing a connection

- An error close sends a close frame with the WebSocket code 1008 and a
  fixed reason that names the error.
- The gateway then waits for the close frame of the agent in reply.
- Sending the close frame and waiting for the reply take at most five
  seconds in total.
- Then the gateway ends the connection, also when no reply arrived.
- The reason `[gateway] authentication failed` says that the gateway rejected
  the install token or the machine credential.
- After the agent sent its machine credential on a connection, the agent
  treats this close as a revocation of the credential.
- The agent then acts as a removed machine, as
  [Drain and remove](agent.md#drain-and-remove) says.
- A lost connection, or a close with another code or reason, does not prove
  a revocation.
- After such a failure, the agent acts as [Contact](#contact) says.

## Related docs

- [Agent](agent.md): install, machine identity, job VMs, drain, and remove.
- [Agent endpoint](agent-endpoint.md): the control plane that an agent
  connects to.
- [Failures](job-routing-failures.md#own-machine-offline): the jobs of an
  offline machine.
- [Security](security.md#key-custody): where the machine credential stays.
- [Architecture](../architecture.md): the gateway and the data flow.
