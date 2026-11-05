# Security

This doc states the security requirements of ciproxy: sign-in, access,
GitHub permissions, keys, isolation, and job data.
[Workspaces](workspaces.md) states which organizations a workspace holds.

## Terms

- **runner right:** the GitHub right to manage the self-hosted runners of
  an [organization](workspaces.md#terms). Every owner of the organization
  has it.

## Sign-in and access

- A person signs in to the [dashboard](../architecture.md#components) with
  GitHub.
- ciproxy has no password of its own.
- When a sign-in does not finish, the dashboard shows an error and the same
  sign-in button.
- A sign-in lasts 30 days.
- After that, every page of the dashboard opens sign-in with a notice that
  says why, and the same button signs in again.
- While a sign-in lasts, ciproxy checks the organization memberships of the
  person with GitHub each hour.
- **Decision:** a sign-in lasts 30 days, with a membership check each hour.
  Rejected: a check only at sign-in, because a person could then keep
  access for 30 days after leaving. Reason: access ends soon after a person
  leaves the organization.
- [Add an organization](workspaces.md#add-an-organization) says how a
  person installs the ciproxy GitHub App on an organization, and which
  [workspace](workspaces.md#terms) the organization joins.

### Roles

- ciproxy controls access to a workspace with its own roles: Admin,
  Billing, and Viewer.
- **Decision:** ciproxy controls access with its own roles. Rejected:
  access by the runner right only, because a person who only pays or only
  views the jobs would then need the runner right. Rejected: mirroring the
  repository access of GitHub, because many pages would then need states
  that are filtered for each person.
- A Viewer sees the jobs, the machines, the savings report, and Plan and
  usage, and changes nothing.
- A person with the Billing role sees all that a Viewer sees.
- A person with the Billing role also manages the plan, the add-ons, and
  the invoices, as [Checkout](billing.md#checkout) says.
- An Admin can do everything: manage the machines, the labels, the
  settings, the organizations, the Boat account, the roles, and the
  billing.
- Only an Admin sees Settings, because Settings changes only what an Admin
  may change.
- **Decision:** each role contains the one below it, so Billing contains
  Viewer, and Admin contains Billing. Rejected: a Billing role that sees
  only the billing pages.
- A person who can see the jobs sees the jobs of all repositories of every
  organization of the workspace, also of those that the person cannot read
  on GitHub.
- An Admin adds own machines and connects the Boat account. ciproxy checks
  no GitHub right for it.
- An own machine and the Boat account run the jobs of every organization of
  the workspace, with their secrets.
- So the Admin role is the trust to control the runners of every
  organization of the workspace.
- When an Admin gives a person the Admin role, the dialog says that an
  Admin can add machines and connect Boat, which run the jobs and their
  secrets of every organization of the workspace.
- An organization that wants other people to control its runners uses a
  workspace of its own.
- **Decision:** adding a machine needs the Admin role only, in a workspace
  with any number of organizations. Rejected: the Admin role together with
  the runner right, because with several organizations the rule became too
  complex to understand and manage. Rejected: the runner right of every
  organization, a confirmation by each new organization, and a consent for
  each pair of machine and organization. Reason: the grant of the role is
  the one explicit step of trust, and its dialog names it.

### Who gets a role

- The person who installs the ciproxy GitHub App on the first organization
  of a new workspace becomes the first Admin of the workspace. When an
  owner approves an install that a member asked for, the member who asked
  becomes the first Admin.
- A role needs current membership of an organization of the workspace.
- A person who is no longer a member of any organization of the workspace
  loses the role and the access at the next membership check.
- **Decision:** a role needs current membership of an organization of the
  workspace. Rejected: roles for people outside its organizations. Reason:
  GitHub stays the source of who belongs to the customer.
- A member of an organization of the workspace with no role asks for access
  by signing in.
- Such a member sees only a page that asks an Admin for access.
- An Admin sees each request for access, and can give the person a role.
- **Decision:** a member asks for access by signing in, and an Admin grants
  a role. Rejected: a list of the members of the GitHub organization,
  because the GitHub App would then need the "Members" permission. Reason:
  the App keeps to the narrow permissions of
  [GitHub App permissions](#github-app-permissions).
- The Members tab of Settings shows the number of open requests for
  access.
- The page that asks for access names the Admins of the workspace, so that
  the person can ask them.
- **Decision:** a count on the Members tab tells the Admins of a request,
  and the page that asks for access names the Admins. Reason: ciproxy
  sends no email.
- An owner of an organization of the workspace can claim the Admin role
  when the workspace has no Admin, or when the person owns every
  organization of the workspace.
- An owner with the Billing or the Viewer role claims the Admin role from
  the account menu.
- ciproxy checks that a person is an owner with the GitHub sign-in of that
  person, from the list of that person's own organization memberships, not
  with a permission of the GitHub App.
- **Decision:** an owner claims the Admin role when the workspace has no
  Admin, or when the person owns every organization of it. Rejected: a
  claim by every owner of any organization, because an owner of a later
  organization could then take over the workspace of another. Reason: no
  workspace loses control of its runners, and no organization takes over
  another.

## GitHub App permissions

- The ciproxy GitHub App asks for "Self-hosted runners", read and write, to
  start a runner for each job.
- The app asks for "Metadata", read, to list the repositories.
- The app asks for "Actions", read, to read the event and the head
  repository of the run of each job from the GitHub API.
- ciproxy needs these facts for
  [trusted write-back](cache.md#trusted-write-back) and for the
  [Docker disks](cache.md#docker-disks).
- The "Actions" permission also lets the app read the logs and the
  artifacts of each run. The control plane never reads them.
- The app asks for no other permission.
- The machine checks the job's runtime-token signature, issuer, validity,
  and assignment identity outside the job VM.
- Its signed write scope supplies the full ref, as
  [Trusted write-back](cache.md#trusted-write-back) says.
- A failed check selects the unknown-ref rules of
  [Cache scopes](cache.md#cache-scopes), without failing the job.
- **Decision:** the App keeps all three permissions. The API supplies the
  event and head repository, and the checked token supplies the full ref.
  Rejected: the API alone, because its branch name can also name a tag.
  Rejected: a hook or an organization webhook, because the checked token
  needs no setup-order trust or manual webhook. Rejected: no checked ref,
  because it prevents trusted shared writes and persistent Docker changes.

## Repository access

- ciproxy runs the jobs of private repositories only.
- A job of a public repository never reaches a ciproxy runner, and the
  repository keeps its current runners.
- GitHub runner groups exclude public repositories by default, so a job
  from a fork of a public repository never reaches an
  [own machine](agent.md#terms).
- Each organization of a workspace keeps its own repository access.
- In Settings under Organizations, the customer allows all private
  repositories or selected repositories of each organization.
- The choice of all private repositories includes the new private
  repositories that ciproxy can see.
- A job of a repository that is not allowed gets no ciproxy runner, and
  GitHub keeps it queued.
- GitHub decides which repositories ciproxy can see.
- The customer changes these repositories on GitHub.

## Key custody

- The GitHub App private key stays in the
  [control plane](../architecture.md#components).
- The Stripe keys stay in the control plane.
- ciproxy stores the [Boat](overflow.md#the-boat-account) key of a
  workspace encrypted.
- ciproxy uses the Boat key only for the
  [sandboxes](overflow.md#overflow-runners) of that workspace.
- The dashboard never shows the Boat key after the customer saves it.
- ciproxy never sends the Boat key to an own machine, a
  [job VM](agent.md#job-vms), or a sandbox.
- A sandbox holds only a token that reports the end of its own runner, as
  [Overflow runners](overflow.md#overflow-runners) says. Job code can read
  this token, but it can only end its own job.
- [Install](agent.md#install) says which workspace an install token belongs
  to and how long it works.
- The [machine credential](agent.md#machine-identity) stays on its machine,
  where only root can read it.
- The machine credential stops working when a person removes the machine,
  as [Drain and remove](agent.md#drain-and-remove) says.
- A runner registration works for one runner only.
- Stripe takes the card in [Checkout](billing.md#checkout) and in the
  [customer portal](billing.md#customer-portal), so the card never reaches
  ciproxy.

## Tenant isolation

- Each workspace has its own machines, [labels](labels.md),
  settings, keys, caches, and data.
- An own machine runs the jobs of every organization of its workspace, each
  in a fresh [job VM](agent.md#job-vms).
- An own machine never runs a job of another workspace.
- The repositories of all the organizations of a workspace keep their cache
  entries and their Docker disks apart, as [Job isolation](#job-isolation)
  says.
- **Decision:** an own machine runs the jobs of every organization of its
  workspace, and tenant isolation holds between workspaces. Rejected: own
  machines for each organization. Reason: one fleet serves the customer,
  and the cache scopes already keep the repositories apart.

## Job isolation

- Each job on an own machine runs in a fresh
  [job VM](agent.md#job-vms).
- So a job never reads the files or the secrets of another job.
- On Boat, each job runs in its own sandbox, as
  [Overflow runners](overflow.md#overflow-runners) says.
- A job VM cannot reach another job VM, the agent, or the host of its own
  machine.
- A job VM can reach the [machine cache](cache.md#cache-tiers) of its own
  machine.
- Apart from the machine cache, a job VM reaches the internet only.
- A job VM cannot reach a private network or a cloud metadata service, also
  when its machine can reach them.
- **Decision:** a job VM reaches the internet only, as on GitHub-hosted
  runners, so a job behaves the same on an own machine and on Boat.
  Rejected: the private networks too, as on a classic self-hosted runner,
  because the code of a pull request could then probe the internal network
  and read the cloud credentials of the host.
- Cache entries and Docker disks follow the isolation rules of
  [Cache scopes](cache.md#cache-scopes),
  [Trusted write-back](cache.md#trusted-write-back), and
  [Docker disks](cache.md#docker-disks).

## Secrets and job data

- Code and secrets go from GitHub straight to the runner.
- The control plane never receives, stores, or logs the code, the secrets,
  the logs, or the cache contents of a job.
- The control plane stores the job metadata that the dashboard shows.
- The job metadata is the name, workflow, repository, ref, result, place,
  times, label, [VM size](labels.md#terms) that ran, cache source, and
  [cache counts](cache.md#terms) of a job.
- The control plane also stores what the agent reports about each machine,
  as [Health](agent.md#health) says.
- Job logs stay on GitHub, and the job page links to them.
- [Removing the App](app-removal.md#removing-the-app) says when ciproxy
  deletes the data of an organization that removes the App.

## Data retention

- ciproxy keeps the job metadata of each finished job for 90 days, on every
  [tier](billing.md#tiers).
- Each page shows only the jobs of the job history of the tier.
- ciproxy keeps the daily totals of the
  [savings report](metering.md#savings-report) for 90 days.
- ciproxy keeps the last report of each own machine, and the cache hits of
  its last 7 days.
- ciproxy keeps the record of a person only while the person has a role or
  an open request for access.
- After 90 days, ciproxy deletes the job metadata and the daily totals.
- **Decision:** 90 days of job metadata on every tier. Rejected: the window
  of the current tier, 13 months of daily totals, and a setting that makes
  the window shorter. Reason: a tier change then deletes nothing, and an
  upgrade shows the older jobs at once.

## Related docs

- [Architecture](../architecture.md): the components and their boundaries.
- [Workspaces](workspaces.md): the organizations of a workspace, and what
  the workspace owns.
- [Agent](agent.md): machine identity, job VMs, and removal.
- [Cache](cache.md): cache scopes, trusted write-back, and Docker disks.
- [Overflow](overflow.md): the Boat account and its sandboxes.
- [Labels](labels.md) and [Job routing](job-routing.md): labels and
  runners.
- [Billing](billing.md): tiers, Checkout, and the customer portal.
- [Removing or suspending the App](app-removal.md): what ciproxy revokes,
  keeps, and deletes when an organization removes the App.
