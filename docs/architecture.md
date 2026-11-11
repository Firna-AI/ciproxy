# Architecture

ciproxy runs the GitHub Actions jobs of a customer's GitHub
[organizations](protocol/workspaces.md#terms) on the customer's
[own machines](protocol/agent.md#terms) first. When those machines are busy,
it starts runners in the customer's own
[Boat](protocol/overflow.md#the-boat-account) account. A
[workspace](protocol/workspaces.md#terms) holds the organizations, the
machines, and the plan of one customer. The protocol docs hold the rules.
Each section links to the doc that owns them.

## Components

The control plane is the hosted service of ciproxy. The dashboard is the web
app of ciproxy, where a person signs in. ciproxy also has the public site,
and the [agent](protocol/agent.md#terms) on each own machine. It uses three
external services: GitHub, Boat, and Stripe.

```text
                     +-----------------+
   Dashboard ------->|  Control plane  |<------- Agent on each own machine
                     +-----------------+                    | starts
                       ^      |      |                      v
                       v      v      v            Job VM with a runner
                    Stripe  GitHub  Boat                    |
                              ^      | starts               |
                              |      v                      |
                              |  Sandbox with a runner      |
                              |      |                      |
                              +------+----------------------+
                                 jobs, code, secrets, logs
```

Each arrow leaves the side that opens the connection or starts the VM.

### Control plane

The control plane is closed source and written in Rust. It has these parts:

- **Dashboard API:** serves the data and the actions of the dashboard. It
  signs people in and checks their [roles](protocol/security.md#roles).
- **GitHub integration:** keeps one scale set for each
  [label](protocol/labels.md) of a workspace in each of its
  organizations. The scale set makes ciproxy the runner provider of the
  label.
- **Scaler:** decides how many [runners](protocol/job-routing.md#terms)
  start, and where they start. This includes the
  [warm runners](protocol/job-routing.md#warm-runners) of each own machine.
- **Machine gateway:** keeps the connections of the agents. It also
  answers the requests of the agents for the cache
  [entry facts](protocol/cache.md#terms) of their running jobs.
- **Backend client:** starts and deletes the
  [sandboxes](protocol/overflow.md#terms) of
  [overflow](protocol/overflow.md#overflow-runners) runners on the
  [backend](protocol/overflow.md#backend-contract), Boat. It counts the Boat
  spend against the [spend limit](protocol/overflow.md#spend-limit).
- **Metering:** counts the [starts](protocol/metering.md#starts) and the own
  machines of each workspace. It builds the
  [savings report](protocol/metering.md#savings-report).
- **Billing:** keeps the [tier](protocol/billing.md#tiers), the
  [add-ons](protocol/billing.md#add-ons), and the
  [subscription](protocol/billing.md#subscription-states) of each
  workspace in step with Stripe.
- **Store:** keeps the state of the control plane.
  [Secrets and job data](protocol/security.md#secrets-and-job-data) says
  what ciproxy stores.

### Agent

The agent is open source and written in Rust. On its own machine, it does
these things:

- It starts a fresh [job VM](protocol/agent.md#job-vms) for each runner,
  warm runners included.
- It keeps the [image](protocol/agent.md#the-image) of the job VMs.
- It serves the [machine cache](protocol/cache.md#cache-tiers) to its job VMs.
- It keeps the [Docker disks](protocol/docker-disks.md) of the machine.

[Updates](protocol/agent.md#updates) says how the agent gets a new version.

### Job VMs and sandboxes

A job VM is a KVM VM on an own machine. A Boat sandbox plays the same part
for overflow. Both start from the image, which holds the GitHub Actions
runner. The runner takes one job.

### Dashboard

The dashboard is written in TypeScript. Its pages are Setup, Jobs, Machines,
Savings, Plan and usage, Plans, and Settings.

### Public site

The public site holds the marketing pages at `/` and the docs at `/docs`.
The docs use Starlight, the docs framework of Astro, with the Graphite
design tokens that the dashboard uses.

- **Decision:** one Astro project serves the marketing pages and the docs.
  Reason: one build and one deploy, search and docs navigation without
  custom work, and one look with the dashboard.

### External services

- **GitHub:** the GitHub App, the scale set API, the Actions service, the
  [GitHub cache](protocol/cache.md#terms), and sign-in. The Actions service
  queues the jobs and talks to each runner.
- **Boat:** the backend. It runs the sandboxes in the customer's own Boat
  account, and it bills the customer for them.
- **Stripe:** Checkout, the customer portal, and the events that confirm
  payments and subscription changes.
- **Download site:** the release server of `futex-ai/cli-release` at
  `get.ciproxy.dev`. It serves the install script and the agent releases.

## Boundaries

- **GitHub and the control plane:** the control plane acts as the ciproxy
  GitHub App, within the
  [App permissions](protocol/security.md#github-app-permissions). It opens a
  message session for each scale set. GitHub sends the messages about the
  jobs of each label through its session. For each new runner, the control
  plane gets a just-in-time runner registration from GitHub. It reads the
  ref, the event, and the head repository of the run of each job from the
  GitHub API. It also reads the [entry facts](protocol/cache.md#terms) of
  the GitHub cache of a repository. GitHub also reports each
  [removal or suspension](protocol/app-removal.md) of the App.
- **GitHub and the runners:** each runner opens its own connection to
  GitHub. It gets one job with its code and secrets, and it sends the logs
  and the artifacts to GitHub. These never pass through the control plane,
  as [Secrets and job data](protocol/security.md#secrets-and-job-data) says.
- **The control plane and the agents:** each agent opens an outbound
  connection to the machine gateway, with its
  [machine credential](protocol/agent.md#machine-identity). The agent sends
  reports. The control plane sends runner registrations, the ref, the
  event, and the head repository of each job, and the
  [drain](protocol/agent.md#drain-and-remove) and remove actions.
  [Connection](protocol/agent.md#connection) holds the rules. For each
  restore of its machine cache, the agent also asks the control plane over
  HTTPS for the entry facts of the repository of the job. The control plane
  answers only the machine that runs the job, as
  [Restore order](protocol/cache.md#restore-order) says.
- **The agent and its job VMs:** the agent passes one runner registration
  into each job VM. [Job isolation](protocol/security.md#job-isolation)
  says what a job VM can reach.
- **The control plane and Boat:** the backend client calls Boat with the
  Boat key of the customer. The
  [backend contract](protocol/overflow.md#backend-contract) says what Boat
  must give ciproxy. [Key custody](protocol/security.md#key-custody) says
  how ciproxy keeps the key.
- **The control plane and Stripe:** billing opens Checkout and the customer
  portal for the customer, and Stripe sends its events back. The card goes
  to Stripe only.
- **The dashboard and the control plane:** the dashboard calls only the
  dashboard API. For sign-in, the GitHub App, and payments, it sends the
  browser to GitHub or to Stripe.
- **The public site and the dashboard:** the sign-in links of the site open
  the dashboard.

## Data flow

The [data flow](architecture/data-flow.md) page follows the main flows
across these boundaries:

- [A job on an own machine][flow-own-machine]
- [A job on Boat][flow-boat]
- [A cache restore and save][flow-cache]
- [A checkout and a failed payment][flow-checkout]
- [Machine reports and offline detection][flow-reports]
- [A control plane outage][flow-outage]

[flow-own-machine]: architecture/data-flow.md#a-job-on-an-own-machine
[flow-boat]: architecture/data-flow.md#a-job-on-boat
[flow-cache]: architecture/data-flow.md#a-cache-restore-and-save
[flow-checkout]: architecture/data-flow.md#a-checkout-and-a-failed-payment
[flow-reports]: architecture/data-flow.md#machine-reports-and-offline-detection
[flow-outage]: architecture/data-flow.md#a-control-plane-outage

## Open source and hosted parts

- Every part that ciproxy ships to the customer's side is open source under
  Apache-2.0: the agent with its connection protocol, the image definition,
  the install script, and the cache sidecar and runner patch in the image.
- The control plane, the dashboard, and the public site have no open
  licence. The hosted control plane and the dashboard are the paid part.
- The whole repository is public. The agent builds and releases from it.
- **Decision:** open core. Rejected: open source only, because it gives no
  revenue. Rejected: a paid licence, because adoption is slower. Rejected:
  reselling overflow minutes, because the margins are thin.
- **Decision:** every part on the customer's side is Apache-2.0. Rejected:
  a closed image definition, and a source-available licence. Reason: the
  customer can audit each part, and its machines hold the built image anyway.
- **Decision:** one public repository for every part, from 5 November 2026.
  Rejected: a private repository with a public copy of the open parts at
  launch, because the agent binaries would then come from a private build.
  Reason: a customer can check that each agent binary comes from the public
  source.

## Related docs

- [Workspaces](protocol/workspaces.md)
- [Labels](protocol/labels.md)
- [Job routing](protocol/job-routing.md)
- [Overflow](protocol/overflow.md)
- [Agent](protocol/agent.md)
- [Security](protocol/security.md)
- [Removing or suspending the App](protocol/app-removal.md)
- [Cache](protocol/cache.md)
- [Docker disks](protocol/docker-disks.md)
- [Metering](protocol/metering.md)
- [Billing](protocol/billing.md)
