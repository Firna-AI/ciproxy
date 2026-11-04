# Removing or suspending the App

This doc states what happens when an [organization](security.md#terms)
removes or suspends the ciproxy GitHub App. An owner can do both on GitHub,
without opening ciproxy.
[Sign-in and access](security.md#sign-in-and-access) states how a person
installs the App.

## Removing the App

- After a removal, ciproxy has no GitHub access left to the organization.
- At the removal, ciproxy revokes each
  [machine credential](agent.md#machine-identity) and each
  [install token](agent.md#install) of the organization at once.
- Each [own machine](agent.md#terms) of the organization then acts as a
  removed machine, as [Drain and remove](agent.md#drain-and-remove) says.
- At the removal, ciproxy also deletes each
  [sandbox](overflow.md#overflow-runners) of the organization that runs no
  job, and then the Boat key.
- Unlike after [Disconnect](overflow.md#change-key-and-disconnect), the
  Boat key does not wait for the Boat jobs that run at that moment.
- These jobs still finish, because a sandbox ends by itself when its runner
  exits, as the [backend contract](overflow.md#backend-contract) says.
- The subscription of the organization ends as
  [Subscription states](billing.md#subscription-states) says.
- After a removal, the [dashboard](../architecture.md#components) says that
  the App was removed, and how to install it again.
- The scale sets and the runners of the organization stay on GitHub,
  because ciproxy has no GitHub access left to delete them.
- 30 days after the removal, ciproxy deletes the data of the organization.
- A new install within these 30 days finds the labels, the settings, and
  the job history again.
- After such an install, each own machine needs the install command again,
  and Boat needs a new connection.
- **Decision:** a removal ends the subscription at the end of the paid
  period, revokes every credential at once, and deletes the data after
  30 days. Rejected: no change, an end at once with a refund, a pause of
  30 days, and deletion at once. Reason: an organization that left pays for
  no new period and keeps no usable credential, and a removal by mistake
  keeps its data for 30 days.

## Suspending the App

- While the App is suspended, no job of the organization starts on a
  ciproxy [runner](job-routing.md#terms).
- Jobs that run when the suspension starts finish.
- A suspension changes nothing else: the machines, the Boat key, the
  subscription, and the data stay.
- After an unsuspend, the jobs of the organization start on ciproxy runners
  again.
- **Decision:** a suspension stops only the jobs, because the owner can undo
  a suspension at any time.

## Related docs

- [Security](security.md): sign-in, roles, key custody, and job data.
- [Agent](agent.md#drain-and-remove): what a removed machine does.
- [Overflow](overflow.md#change-key-and-disconnect): the Boat key and the
  sandboxes.
- [Billing](billing.md#subscription-states): the end of a subscription, also
  in a Business pool.
- [Architecture](../architecture.md#boundaries): the events that GitHub
  sends to the control plane.
