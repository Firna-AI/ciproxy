# Workspaces

This doc states what a workspace is, which GitHub organizations it holds,
and what it owns. It also states how an Admin adds an organization to a
workspace and takes one out.

## Terms

- **workspace:** the account of one customer in ciproxy. It holds one or
  more organizations.
- **organization:** a GitHub organization that installed the ciproxy GitHub
  App.
- The product says "workspace" for the account of a customer, never
  "team".
- **Decision:** the product calls the account of a customer a workspace.
  Rejected: "team", because Team is a [tier](billing.md#tiers).

## Organizations of a workspace

- A workspace holds one or more organizations.
- An organization belongs to one workspace at most.
- Free allows one organization in a workspace. Team and Business allow any
  number.
- So several GitHub organizations are a feature of Team, as
  [Tier features](billing.md#tier-features) says.
- A workspace that holds more than one organization when it moves to Free
  keeps them all, and their jobs keep running.
- A workspace with more than one organization can choose "Move to Free".
- On Free, the number of organizations is a soft limit, as
  [Soft limits](metering.md#soft-limits) says for the machines. Above it,
  every page shows the banner and the offer.
- **Decision:** the number of organizations is a soft limit on Free.
  Rejected: no notice, and a refusal of "Move to Free". Reason: it treats
  organizations as the machine count is treated.
- A person can have a role in several workspaces, and switches between
  them in the [dashboard](../architecture.md#components).
- A new workspace takes the name of its first organization. An Admin can
  rename it.
- **Decision:** a workspace holds one or more organizations, and several
  organizations come with Team and Business. Rejected: a Business pool
  that organizations join on Plan and usage, by an invite, at their first
  install, or through the ciproxy team. Rejected: several organizations as
  a feature that comes later. Reason: a customer with several organizations
  then has one plan, one set of members, and one fleet of machines, and an
  organization joins only through an install that GitHub allows.

## What a workspace owns

- The workspace owns the tier, the subscription, the members and their
  roles, the [own machines](agent.md#terms), the
  [install tokens](agent.md#install), the [labels](labels.md),
  the settings, and the [Boat account](overflow.md#the-boat-account).
- Each organization keeps its own repository access, as
  [Repository access](security.md#repository-access) says.
- An own machine runs the jobs of every organization of its workspace, as
  [Tenant isolation](security.md#tenant-isolation) says.
- A person with a role sees the jobs of every organization of the
  workspace, as [Roles](security.md#roles) says.
- [Who gets a role](security.md#who-gets-a-role) says how a person joins a
  workspace.
- The starts, the machine count, the soft limits, and the savings report
  cover the whole workspace, as [Metering](metering.md) says.

## Add an organization

- A person installs the ciproxy GitHub App on an organization on GitHub.
- On GitHub, the person picks the organization and its repositories.
- A member who is not an owner can ask for the install, and GitHub sends the
  request to the owners.
- When an owner approves the request, GitHub installs the app on the
  organization.
- While the request waits, the person can check again or install the app on
  another organization.
- ciproxy serves GitHub organizations only.
- When a person installs the app on a personal account, ciproxy makes no
  workspace for it.
- The page that GitHub then returns to says that ciproxy works with
  organizations only.
- The page offers to install the app on an organization, and it links to
  the GitHub settings that remove the app from the personal account.
- **Decision:** a personal account gets this page and no workspace.
  Rejected: a workspace for a personal account. Reason: a GitHub
  organization costs nothing.
- An install that no Admin started with "Add an organization" makes a new
  workspace with that organization.
- The one exception is a new install within 30 days of a removal, as
  [Removing the App](app-removal.md#removing-the-app) says.
- An Admin adds an organization to the workspace with "Add an
  organization", which starts the same install on GitHub.
- So GitHub decides who may finish the install.
- ciproxy links the installation to the workspace of the Admin who started
  it, also when an owner of the organization approves the install later.
- A workspace on Free cannot add a second organization.
- The jobs of the new organization use the labels, the own machines, the
  settings, and the Boat account of the workspace at once.

## Take an organization out

- An Admin can take an organization out of the workspace.
- No new job of the organization then gets a ciproxy
  [runner](job-routing.md#terms).
- Its runners that have no job stop at once, and its running jobs finish.
- The App stays installed on the organization until an owner removes it on
  GitHub.
- A take-out deletes no data. The jobs of the organization stay in the job
  history of the workspace, as [Data retention](security.md#data-retention)
  says.
- A person keeps a role only while the person is a member of an
  organization that stays in the workspace, as
  [Who gets a role](security.md#who-gets-a-role) says.
- **Open:** how an organization that still has the App joins a workspace,
  and whether an Admin can take out the last organization of a workspace.
  It waits for the decision on organizations that leave a workspace.

## Related docs

- [Security](security.md): sign-in, roles, repository access, and tenant
  isolation.
- [Billing](billing.md): the tiers, their features, and the subscription of
  a workspace.
- [Metering](metering.md): the counts and the savings report of a
  workspace.
- [Removing or suspending the App](app-removal.md): what a removal does to
  an organization and to its workspace.
- [Labels](labels.md) and [Job routing](job-routing.md): the labels and the
  warm runners of a workspace.
- [Agent](agent.md): own machines and install tokens.
