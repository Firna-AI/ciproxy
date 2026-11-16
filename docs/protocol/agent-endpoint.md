# Agent endpoint

This doc states which control plane an own machine registers with, and
which control plane receives its machine credential.
[Agent](agent.md) holds the other requirements of the agent.

## Terms

- **agent endpoint:** the `wss://` address of a control plane, at which the
  agent registers and connects.
- **machine identity:** what registration saves on the own machine: the
  workspace and machine IDs, the machine credential, and the agent
  endpoint.
- **earlier endpoint:** the agent endpoint that an earlier agent saved
  apart from the machine identity.
- [Agent](agent.md#terms) defines the own machine, the install token, and
  the machine credential.

## Install

- Install selects its agent endpoint from its command, never from saved
  state.
- The order is `--agent-url`, then the `CIPROXY_AGENT_URL` environment
  setting, then `wss://app.ciproxy.dev/agent/`.
- An install endpoint must be a `wss://` URL.
- A machine without a machine identity registers at the selected endpoint.
- A registered machine keeps its machine identity and its agent endpoint,
  also when the command selects another endpoint.
- An install on a registered machine sends its install token nowhere.
- That install repairs the machine and succeeds, also with another
  endpoint.
- When the selected endpoint differs from the endpoint of the machine
  identity, the last output line of the install names both.
- That line says to uninstall first to move the machine to the selected
  endpoint.
- **Decision:** install selects its endpoint from its command, and a
  registered machine keeps its own. A repair with another endpoint
  succeeds, so a script sees success although the machine did not move.
  Rejected: a refusal, because the kept and repaired machine is correct and
  safe. Rejected: a confirmation prompt, because the piped command gives
  the agent no terminal. Rejected: an automatic move, because uninstall and
  a new install already move a machine. Rejected: `--agent-url` in every
  command, so that an install without it could reuse the saved endpoint,
  because the production command keeps its form.

## The saved endpoint

- Registration saves the agent endpoint and the machine credential
  together, so an interruption saves both or neither.
- The agent sends the machine credential only to the agent endpoint of its
  machine identity.
- This holds for the service, an install, uninstall, and each request for
  the [entry facts](cache.md#restore-order) of a cache restore.
- Restarts, updates, and later installs keep that endpoint.
- A machine identity without a usable agent endpoint sends its machine
  credential nowhere.
- Such a machine stays offline.
- Its uninstall still finishes the local cleanup.
- When the machine identity has no agent endpoint, a newer agent copies the
  earlier endpoint into it.
- The newer agent keeps the earlier endpoint while the machine identity
  lives and has the same agent endpoint.
- So the machine keeps its agent endpoint when an earlier agent or a
  restored state backup drops it from the machine identity.
- The newer agent removes the earlier endpoint at uninstall and when it
  forgets the machine identity.
- It also removes an earlier endpoint that differs from the agent endpoint
  of the machine identity.
- **Decision:** the earlier endpoint stays while the machine identity
  lives. Rejected: its removal after the copy, because an earlier agent or
  a restored state backup can later drop the agent endpoint from the
  machine identity. A stop of the agent service does not prevent this,
  because an agent that a person started directly keeps running.
- The agent of a removed machine forgets its machine identity, with the
  agent endpoint, after its jobs end.
- A later install then registers a new machine at the endpoint of its
  command.
- **Decision:** the machine identity carries the agent endpoint, saved
  together with the machine credential. Rejected: an endpoint saved apart
  from the credential. An install, a cache request, a removal, or an
  interrupted registration could then pair the credential or an install
  token with another control plane. Rejected: a check in the control plane,
  because the credential has then already left the machine. Rejected: a
  signed key pair in place of the bearer credential, because it changes the
  protocol, the gateway, and the store. The saved endpoint already closes
  each of these paths.

## Related docs

- [Agent](agent.md): install, machine identity, drain, and remove.
- [Security](security.md#key-custody): where the machine credential stays.
- [Cache](cache.md#restore-order): the entry facts of a cache restore.
