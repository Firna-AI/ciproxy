# Removing or suspending the App

This doc states what happens when an [organization](workspaces.md#terms)
removes or suspends the ciproxy GitHub App. An owner can do both on GitHub,
without opening ciproxy.
[Add an organization](workspaces.md#add-an-organization) states how a
person installs the App.

## Removing the App

- After a removal, ciproxy has no GitHub access left to the organization.
- A removal takes the organization out of its
  [workspace](workspaces.md#terms).
- No new job of the organization then gets a ciproxy
  [runner](job-routing.md#terms).
- At the removal, the warm runners of the organization stop at once.
- A warm runner can take a job just before that stop.
- That job can fail when the runner has not handed it to the
  [guest](agent.md#drain-and-remove) yet.
- ciproxy also deletes each [sandbox](overflow.md#overflow-runners) of the
  organization that runs no job.
- The other jobs of the organization that run at that moment finish,
  including each job that the guest received.
- **Decision:** the warm runners stop at once, also when a job can then
  fail, as at "Remove" in [Drain and remove](agent.md#drain-and-remove).
- The scale sets and the runners of the organization stay on GitHub,
  because ciproxy has no GitHub access left to delete them.
- After a removal, the [dashboard](../architecture.md#components) says that
  the App was removed from the organization, and how to install it again.
- For 30 days, the Organizations tab of Settings shows the organization as
  removed, with the last day on which a new install keeps its job
  history.
- 30 days after the removal, ciproxy deletes the data of the organization.
- A new install within these 30 days joins the organization to the
  workspace that it left, with its job history.
- This holds unless an Admin of another workspace starts the install with
  "Add an organization".
- **Decision:** a removal takes only that organization out of its
  workspace. Reason: the other organizations of the workspace still use its
  machines, its Boat account, and its subscription.

### The last organization of a workspace

- These rules apply when the removed organization was the last
  organization of its workspace.
- At the removal, ciproxy revokes each
  [machine credential](agent.md#machine-identity) and each
  [install token](agent.md#install) of the workspace at once.
- Each [own machine](agent.md#terms) of the workspace then acts as a
  removed machine, as [Drain and remove](agent.md#drain-and-remove) says.
- At the removal, ciproxy deletes each sandbox of the workspace that runs
  no job, and starts no new Boat runner.
- The Boat jobs that run at that moment finish. ciproxy deletes each of
  their sandboxes when its runner reports its end, as
  [Overflow runners](overflow.md#overflow-runners) says.
- ciproxy deletes the Boat key when the last of these sandboxes ends, as
  after [Disconnect](overflow.md#change-key-and-disconnect).
- Reason: ciproxy needs the key to delete these sandboxes, so that none
  bills until its lifetime ends.
- The subscription of the workspace ends as
  [Subscription states](billing.md#subscription-states) says.
- 30 days after the removal, ciproxy also deletes the data of the
  workspace, such as its labels and its settings.
- A new install within these 30 days finds the labels, the settings, and
  the job history of the workspace again.
- After such an install, each own machine needs the install command again,
  and Boat needs a new connection.
- **Decision:** when the last organization leaves, the subscription ends at
  the end of the paid period, every credential goes at once, and the data
  goes after 30 days. Rejected: no change, an end at once with a refund, a
  pause of 30 days, and deletion at once. Reason: a workspace that no
  organization uses pays for no new period and keeps no usable credential,
  and a removal by mistake keeps its data for 30 days.

## Suspending the App

- While the App is suspended, no job of the organization starts on a
  ciproxy runner.
- Jobs that run when the suspension starts finish.
- A suspension changes nothing else: the organization stays in its
  workspace, and the machines, the Boat key, the subscription, and the data
  stay.
- After an unsuspend, the jobs of the organization start on ciproxy runners
  again.
- While an organization is suspended, the Jobs page shows a notice that
  names it and links to GitHub. The page shows no waiting count for it.
- Reason: the other reasons why jobs wait also show on the Jobs page.
- **Decision:** a suspension stops only the jobs, because the owner can undo
  a suspension at any time.

## Related docs

- [Workspaces](workspaces.md): the organizations of a workspace, and how an
  Admin takes one out.
- [Security](security.md): sign-in, roles, key custody, and job data.
- [Agent](agent.md#drain-and-remove): what a removed machine does.
- [Overflow](overflow.md#change-key-and-disconnect): the Boat key and the
  sandboxes.
- [Billing](billing.md#subscription-states): the end of a subscription.
- [Architecture](../architecture.md#boundaries): the events that GitHub
  sends to the control plane.
