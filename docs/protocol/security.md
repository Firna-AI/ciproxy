# Security

This doc states the security requirements of ciproxy: sign-in, access,
GitHub permissions, keys, isolation, and job data.

## Terms

- **organization:** a GitHub organization that installed the ciproxy GitHub
  App.
- **runner right:** the GitHub right to manage the self-hosted runners of
  an organization. Every owner of the organization has it.

## Sign-in and access

- A person signs in to the [dashboard](../architecture.md#components) with
  GitHub.
- ciproxy has no password of its own.
- When a sign-in does not finish, the dashboard shows an error and the same
  sign-in button.
- A person installs the ciproxy GitHub App on an organization on GitHub.
- On GitHub, the person picks the organization and its repositories.
- A member who is not an owner can ask for the install, and GitHub sends the
  request to the owners.
- When an owner approves the request, GitHub installs the app on the
  organization.
- While the request waits, the person can check again or install the app on
  another organization.

### Roles

- ciproxy controls access to an organization with its own roles: Admin,
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
  settings, the Boat account, the roles, and the billing.
- **Decision:** each role contains the one below it, so Billing contains
  Viewer, and Admin contains Billing. Rejected: a Billing role that sees
  only the billing pages.
- A person who can see the jobs sees the jobs of all repositories, also of
  those that the person cannot read on GitHub.
- To add an own machine, an Admin also needs the runner right of the
  organization.
- Apart from adding a machine, the roles alone decide what a person can do.
- So an Admin without the runner right, the first Admin included, can do
  everything except add a machine.
- **Decision:** adding a machine needs the Admin role and the runner right,
  because an own machine receives the secrets of the jobs that its
  [runners](job-routing.md#terms) take. Rejected: the Admin role alone,
  because an Admin could then give a machine the job secrets of
  repositories that GitHub would not trust that person with. Rejected: the
  runner right for the first Admin only, because later Admins would then
  add machines without it.

### Who gets a role

- The person who installs the ciproxy GitHub App on an organization becomes
  its first Admin. When an owner approves an install that a member asked
  for, the member who asked becomes the first Admin.
- A role needs current membership of the GitHub organization.
- A person who leaves the GitHub organization loses access to it in ciproxy
  at the next sign-in check.
- **Decision:** a role needs current membership of the GitHub organization.
  Rejected: roles for people outside the organization. Reason: GitHub stays
  the source of who belongs to the organization.
- A member of the GitHub organization with no role asks for access by
  signing in.
- Such a member sees only a page that asks an Admin for access.
- An Admin sees each request for access, and can give the person a role.
- **Decision:** a member asks for access by signing in, and an Admin grants
  a role. Rejected: a list of the members of the GitHub organization,
  because the GitHub App would then need the "Members" permission. Reason:
  the App keeps its runner-only permissions.
- An owner of the GitHub organization can always claim the Admin role.
- ciproxy checks that a person is an owner with the GitHub sign-in of that
  person, from the list of that person's own organization memberships, not
  with a permission of the GitHub App.
- **Decision:** an owner of the GitHub organization can always claim the
  Admin role. Reason: an organization never loses control of ciproxy.

## GitHub App permissions

- The ciproxy GitHub App asks for "Self-hosted runners", read and write, to
  start a runner for each job.
- The app asks for "Metadata", read, to list the repositories.
- The app asks for no other permission.

## Repository access

- ciproxy runs the jobs of private repositories only.
- A job of a public repository never reaches a ciproxy runner, and the
  repository keeps its current runners.
- GitHub runner groups exclude public repositories by default, so a job
  from a fork of a public repository never reaches an
  [own machine](agent.md#terms).
- In Settings under Repositories, the customer allows all private
  repositories or selected repositories.
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
- ciproxy stores the [Boat](overflow.md#the-boat-account) key of an
  organization encrypted.
- ciproxy uses the Boat key only for the
  [sandboxes](overflow.md#overflow-runners) of that organization.
- The dashboard never shows the Boat key after the customer saves it.
- ciproxy never sends the Boat key to an own machine, a
  [job VM](agent.md#job-vms), or a sandbox.
- [Install](agent.md#install) says which organization an install token
  belongs to and how long it works.
- The [machine credential](agent.md#machine-identity) stays on its machine,
  where only root can read it.
- The machine credential stops working when a person removes the machine,
  as [Drain and remove](agent.md#drain-and-remove) says.
- A runner registration works for one runner only.
- Stripe takes the card in [Checkout](billing.md#checkout) and in the
  [customer portal](billing.md#customer-portal), so the card never reaches
  ciproxy.

## Tenant isolation

- Each organization has its own machines, [labels](job-routing.md#labels),
  settings, keys, caches, and data.
- An own machine runs the jobs of its own organization only.
- **Open:** whether the organizations of one [Business](billing.md#tiers)
  [subscription](billing.md#subscription-states) can share own machines. It
  waits for item 8, the tier features at launch.

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
  times, label, [size](job-routing.md#labels), and cache counts of a job.
- The control plane also stores what the agent reports about each machine,
  as [Health](agent.md#health) says.
- Job logs stay on GitHub, and the job page links to them.

## Related docs

- [Architecture](../architecture.md): the components and their boundaries.
- [Agent](agent.md): machine identity, job VMs, and removal.
- [Cache](cache.md): cache scopes, trusted write-back, and Docker disks.
- [Overflow](overflow.md): the Boat account and its sandboxes.
- [Job routing](job-routing.md): labels and runners.
- [Billing](billing.md): tiers, Checkout, and the customer portal.
