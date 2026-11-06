# Workspaces

This doc states what a workspace is, which GitHub organizations it holds,
and what it owns. It also states how an Admin adds an organization to a
workspace and takes one out, and when an organization is inactive.

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
- An organization of a workspace is active or
  [inactive](#inactive-organizations).
- Free allows one organization in a workspace. Team and Business allow any
  number.
- So several GitHub organizations are a feature of Team, as
  [Tier features](billing.md#tier-features) says.
- A workspace that holds more than one organization when it moves to Free
  keeps them all. Its active organizations keep running their jobs.
- A workspace with more than one organization can choose "Move to Free".
- On Free, the number of organizations is a soft limit, as
  [Soft limits](metering.md#soft-limits) says for the machines. Above it,
  every page shows the banner and the offer.
- A new organization above the limit of the tier joins inactive, as
  [Inactive organizations](#inactive-organizations) says.
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
- An Admin adds an organization to a workspace with "Add an organization",
  which starts the same install on GitHub.
- So GitHub decides who may finish the install.
- An install that starts with "Add an organization" never makes a new
  workspace.
- On Free, an Admin can also add an organization. It joins inactive, as
  [Inactive organizations](#inactive-organizations) says.
- The jobs of a new active organization use the labels, the own machines,
  the settings, and the Boat account of the workspace at once.

### Choose the workspace

- ciproxy accepts a return from GitHub as part of an add only for the
  person who started the add.
- ciproxy then checks, with the GitHub sign-in of the person at that
  moment, that the person can reach the new installation.
- When the check fails, ciproxy links nothing, says that it cannot confirm
  the install, and asks the person to sign in with GitHub again.
- **Decision:** ciproxy trusts the return only for the person who started
  the add, and the installation only through that person's own GitHub
  sign-in. Rejected: the installation that the return names, and a check
  with the access of the App alone. Reason: a crafted return must not link an
  organization that the person cannot reach.
- After the check, ciproxy lists the workspaces where the person is an
  Admin at that moment.
- With one such workspace, the organization joins it at once.
- With several, the person picks one. No workspace is picked at first, and
  the list offers no new workspace.
- The list says when the tier of a workspace keeps the organization
  inactive there.
- With none, the organization joins no workspace and waits.
- The page then asks the person to get the Admin role from an Admin of a
  workspace, and to check again.
- ciproxy never chooses the workspace in place of the person. The page
  where the add started, the newest add, and the time of a GitHub event
  choose nothing.
- Neither the order of the GitHub events nor a second add at the same
  time changes the workspace that an organization joins.
- When the person who chose the workspace is no longer its Admin when the
  organization joins, the organization joins it inactive.
- **Decision:** the person picks the organization on GitHub and the
  workspace in ciproxy, after the check. Rejected: a pick of the
  organization in ciproxy before GitHub opens, and installs with no
  workspace that an Admin assigns later. Reason: the install page of GitHub
  stays as it is, and an organization joins only a workspace that its
  person chose.

### A request that an owner approves later

- A return from a request that waits for an owner links nothing and
  chooses no workspace.
- After an owner approves the request, the organization waits until the
  person who asked signs in again.
- ciproxy then checks the installation with the GitHub sign-in of that
  person, and the person chooses the workspace, as on a return.
- Only the person who asked sees an organization that waits for a choice.
- **Decision:** a late approval waits for the check and the choice of the
  person who asked. Rejected: a choice of the workspace when the request is
  sent, and a link from the GitHub event alone. Reason: no installation
  exists to check before the approval, and the event names neither the add
  nor its workspace.

## Inactive organizations

- An inactive organization belongs to its workspace and keeps the App.
- It counts as an organization of the workspace, for the tier and for every
  other rule.
- No job of an inactive organization gets a ciproxy
  [runner](job-routing.md#terms). GitHub keeps its jobs queued.
- The other organizations of the workspace keep their runners.
- Inactive is a state of ciproxy, apart from a suspension or a removal of
  the App, which [Removing or suspending the App](app-removal.md) states.
- An organization becomes inactive only when it joins through "Add an
  organization".
- It stays inactive while one of these reasons holds:
  - The plan: the workspace holds more organizations than its tier allows.
    A removed organization does not count.
  - The Admin: the person who chose the workspace was no longer its Admin
    when the organization joined, and no current Admin has turned it on.
- Both reasons can hold at once. The organization turns active when no
  reason holds.
- **Decision:** an organization that its workspace cannot take yet joins
  inactive, and turns active when its reasons end. Rejected: a new
  workspace for the organization, a removal of the App, and an install that
  waits outside the workspace until an Admin accepts it. Reason: the
  organization joins the workspace that was meant, and no job runs before
  the plan and an Admin allow it.
- The plan reason ends by itself as soon as the organizations of the
  workspace fit its tier.
- An active organization never turns inactive. So a move to Free keeps
  every active organization running.
- **Decision:** the plan reason counts every organization of the workspace,
  and an active organization stays active. Rejected: a count of the active
  organizations only, and an order in which waiting organizations turn on.
  Reason: the tier keeps one count, and an organization turns on only when
  all of them fit.
- An Admin of the workspace can turn on an organization that waits for an
  Admin.
- Before an Admin turns it on, a dialog says that its jobs then run on the
  machines of the workspace, and who added it.
- **Decision:** when the person who chose the workspace is no longer its
  Admin, the organization joins inactive, and a current Admin turns it on.
  Rejected: a link that runs at once. Reason: a person who lost the Admin
  role adds no running organization.
- The link of an organization gives no person a role, as
  [Who gets a role](security.md#who-gets-a-role) says.
- A removal and a new install within 30 days keep the inactive state and
  its reasons, as [Removing the App](app-removal.md#removing-the-app) says.
- **Decision:** a removal and a new install keep a hold. Rejected: a new
  install that resets the hold. Reason: neither step is an approval or a
  change of the plan.
- The Organizations tab of Settings marks an inactive organization, with
  one line for each reason.
- For the plan reason, the tab gives the
  [offer](metering.md#the-use-and-the-offer) of a tier that allows the
  organizations. "Turn on" shows when the Admin reason is the only reason.
- The tab also shows a banner that names the organization and its reasons,
  and says that the jobs of the other organizations keep running.
- This banner takes the place of the soft-limit banner. The banner of a
  failed payment takes the place of this banner.

## Take an organization out

- An Admin can take an organization out of the workspace.
- An Admin cannot take out the last organization of a workspace.
- No new job of the organization then gets a ciproxy
  [runner](job-routing.md#terms).
- Its runners that have no job stop at once, and its running jobs finish.
- ciproxy deletes the scale sets of the organization, and then removes its
  own install of the App from the organization through the GitHub API.
- The organization can then join any workspace again, with "Add an
  organization", as a new install.
- The rules of [Removing the App](app-removal.md#removing-the-app) do not
  apply to a take-out, because ciproxy itself removed the App.
- A take-out deletes no data. The jobs of the organization stay in the job
  history of the workspace, as [Data retention](security.md#data-retention)
  says.
- A person keeps a role only while the person is a member of an
  organization that stays in the workspace, as
  [Who gets a role](security.md#who-gets-a-role) says.
- **Decision:** a take-out removes the App from the organization, and the
  last organization stays. Rejected: a take-out that leaves the App, a join
  of organizations that keep the App, a pause in place of a take-out, and a
  take-out of the last organization. Reason: ciproxy keeps no install that
  no workspace serves, and the organization needs no new way to join.

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
