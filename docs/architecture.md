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
- **Machine gateway:** keeps the connections of the agents.
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
- It keeps the [Docker disks](protocol/cache.md#docker-disks) of the machine.

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
  GitHub API. GitHub also reports each
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
  [Connection](protocol/agent.md#connection) holds the rules.
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

### A job on an own machine

1. GitHub queues a job whose `runs-on` names a ciproxy label. It sends a
   message about the job to the control plane.
2. GitHub gives the job to a ready warm runner of its organization and
   label, and the job starts at once. Without a warm runner, the scaler
   starts a runner in free [slots](protocol/job-routing.md#slots) of an own
   machine of the workspace.
3. The control plane reads the ref, the event, and the head repository of
   the run of the job from the GitHub API. It sends them to the agent for
   the [cache scopes](protocol/cache.md#cache-scopes) and the Docker disk of
   the job.
4. Metering counts one start. When a slot and its disk are free, the
   scaler sends the agent a registration for a new warm runner.
5. When the job ends, the agent ends the job VM and reports it.

### A job on Boat

1. GitHub queues a job of a ciproxy label. No own machine has free slots
   for it, so the job waits in GitHub.
2. When the job has waited the
   [overflow wait](protocol/job-routing.md#where-a-job-runs), the scaler
   starts a runner on Boat. The backend client starts its sandbox from the
   image in the customer's Boat account.
3. GitHub gives a waiting job of the label to the new runner. Metering
   counts one start. [Overflow runners](protocol/overflow.md#overflow-runners)
   says what a Boat runner without a job does.
4. When the job ends, the backend client deletes the sandbox. It adds the
   cost of the sandbox to the Boat spend of the month.

### A cache restore and save

1. On an own machine, a step such as `actions/cache` restores an entry. The
   [cache scope](protocol/cache.md#cache-scopes) of the job decides which
   entries it can restore.
2. The machine cache gives the contents when it has them. Otherwise they
   come from the GitHub cache, and the machine cache keeps them. When the
   machine cache fails, the job falls back to the
   [GitHub cache](protocol/cache.md#fallback-to-the-github-cache).
3. A save goes to the machine cache and through to the GitHub cache.
   [Trusted write-back](protocol/cache.md#trusted-write-back) says which
   saves other branches can restore.
4. A job on Boat restores and saves through the GitHub cache only.
5. The agent reports the cache hits of its machine to the control plane.
   It also reports the [cache counts](protocol/cache.md#terms) of each job
   whose restores the machine cache handled. A sandbox reports no cache
   counts. The cache contents stay on the machine and in the GitHub cache.

### A checkout and a failed payment

1. In the dashboard, the customer picks a tier and the add-on quantities.
   Billing opens [Stripe Checkout](protocol/billing.md#checkout), where the
   customer pays.
2. Stripe sends an event that confirms the payment. Billing then changes
   the tier, and a new [period](protocol/metering.md#periods) starts.
3. When a renewal charge fails, Stripe sends an event and tries again
   later. [Failed payments](protocol/billing.md#failed-payments) says what
   the dashboard shows.
4. The customer changes the card in the Stripe customer portal. Stripe
   sends an event for each later charge.
5. After the last failed retry, Stripe ends the subscription. Billing moves
   the workspace to Free.

### Machine reports and offline detection

1. The agent registers once with the
   [install token](protocol/agent.md#install). Then it connects out with
   its machine credential.
2. At a fixed interval, the agent reports its slots, its job VMs, its warm
   runners, its [health](protocol/agent.md#health) checks, and its version.
3. When no report arrives for a [set time](protocol/agent.md#connection),
   the control plane marks the machine offline. The scaler starts no new
   runner on it. [Failures](protocol/job-routing.md#failures) says what
   happens to its jobs.
4. When the agent connects again, the machine is back online.

### A control plane outage

1. Running jobs go on, because each runner talks to GitHub directly.
2. No new runner starts on an own machine or on Boat. Jobs without a runner
   stay queued in GitHub. [Failures](protocol/job-routing.md#failures)
   holds the rules.
3. The agents keep their job VMs and warm runners, and they try to connect
   again.
4. When the control plane is back, the scaler starts runners for the queued
   jobs. The backend client deletes each sandbox that ciproxy started, that
   runs no job, and that no waiting job needs.

## Open source and hosted parts

- Every part that ciproxy ships to the customer's side is open source under
  Apache-2.0: the agent with its connection protocol, the image definition,
  the install script, and the cache sidecar and runner patch in the image.
- The control plane, the dashboard, and the public site are closed source.
  The hosted control plane and the dashboard are the paid part.
- **Decision:** open core. Rejected: open source only, because it gives no
  revenue. Rejected: a paid licence, because adoption is slower. Rejected:
  reselling overflow minutes, because the margins are thin.
- **Decision:** every part on the customer's side is Apache-2.0. Rejected:
  a closed image definition, and a source-available licence. Reason: the
  customer can audit each part, and its machines hold the built image anyway.

## Related docs

- [Workspaces](protocol/workspaces.md)
- [Labels](protocol/labels.md)
- [Job routing](protocol/job-routing.md)
- [Overflow](protocol/overflow.md)
- [Agent](protocol/agent.md)
- [Security](protocol/security.md)
- [Removing or suspending the App](protocol/app-removal.md)
- [Cache](protocol/cache.md)
- [Metering](protocol/metering.md)
- [Billing](protocol/billing.md)
