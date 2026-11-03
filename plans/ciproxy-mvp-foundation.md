# ciproxy MVP foundation

Status: Active. On branch `calummoore/ci-proxy-load-balancer`, no pull
request yet. The pull request that completes the work deletes this plan, as
the `plans` skill says.

ciproxy runs GitHub Actions jobs on the own machines of a customer first.
When those machines are busy, it starts rented VMs from an overflow
provider. This plan builds the foundation of the MVP: the mockups of every
screen, the architecture and protocol docs, five feasibility spikes, and the
repository skeleton with its gate. Its last work milestone writes the
implementation plan, `plans/ciproxy-mvp.md`. This plan does not build the
product.

Contract owners:

- The architecture doc, `docs/architecture.md`. Milestone 2 creates it.
- The protocol docs under `docs/protocol/`. Milestone 2 creates them.
- The [repository README](../README.md).

## Decisions

User decisions, 3 November 2026:

- Approach: ciproxy is the runner provider. Every job uses a ciproxy label,
  such as `runs-on: ciproxy`. ciproxy gets the jobs through the GitHub scale
  set API. It starts each runner on a free own machine, or else on a rented
  VM. Rejected: adopting existing tools, because nothing is left to sell.
  Rejected: a label router that picks a provider label before each job
  queues, because it needs a router job and an edit in each job. This
  approach won for the simplest interface and the strongest product.
- Business model: open core. The agent on own machines is open source. The
  hosted control plane is paid. Rejected: open source only, because it gives
  no revenue. Rejected: a paid licence, because adoption is slower.
  Rejected: reselling overflow minutes, because the margins are thin.
- Goal: ship a paid product as fast as possible. The product must also solve
  the CI cost problem of the user's own team.
- Code location: this repository is private. At launch, the agent goes
  public as open source, from a public copy of its directory. Rejected: one
  public repository under Apache-2.0, because anyone could host the paid
  part. Rejected: one public repository with the control plane under the
  Functional Source License, because that licence lets a customer run the
  control plane for itself without paying. Rejected: two repositories now,
  because the protocol would cross repositories. Rejected: a public copy
  from the first push, because early code would go public before launch.
  Reason: the fastest layout that keeps the paid code and the plans
  private. The repository became private on 3 November 2026.
- Stack: Rust for the agent and the control plane, TypeScript for the
  dashboard, as in `futex-ai/accounting`. Rejected: Go, because no shared
  skill covers it. Rejected: TypeScript everywhere, because no shared skill
  covers a TypeScript server, and a VM agent on Node is weaker than a
  static binary. Rejected: a Rust agent with a TypeScript control plane,
  because the paid part would get the fewest shared rules. Reason: the
  shared Rust rules, the gate and the billing code of `futex-ai/accounting`,
  and a Rust cache server already exist. The gap to the official Go client
  is small.
- Pricing: three tiers. Each tier includes a number of starts per month and
  a number of own nodes. Add-ons buy extra starts and extra own nodes. A
  start is one job that a ciproxy runner starts, on an own node or on
  overflow. The customer picks the tier by hand. This replaces the first
  choice of the same day: a monthly price in bands of jobs. Rejected
  before that: a price per own machine, per vCPU, or per job-minute, a
  share of the savings, and a flat price. Reason: starts grow with agents
  and pull requests, which drive CI load, and the node count is simple to
  check.
- Billing: in-product billing is part of the MVP, with the tiers and the
  add-ons below. The user approved them and this plan on 3 November 2026,
  in the words "ok, looks good - lets go ahead". This withdraws my earlier
  call to leave billing out of the MVP.
- The options check of the `plans` skill ran over the last three decisions
  before the user answered. Its report is in
  `.context/ciproxy-mvp-foundation/options-check-1.md`.

Design calls, my calls, 3 November 2026:

- The MVP serves Linux x64 jobs in private repositories only. Reason: this
  is the fastest scope. GitHub runner groups exclude public repositories by
  default, so jobs from forks of public repositories never reach own
  machines.
- macOS comes after the MVP, on own Macs only. Reason: cloud Macs rented by
  API bill 24 hours at minimum under Apple's licence, so per-minute macOS
  overflow is not possible.
- The first overflow backend is Boat sandboxes, on the Boat account of the
  customer. AWS EC2 comes next, or first when the Boat terms forbid this
  use. Blacksmith sandboxes follow when their API leaves preview. Reason:
  Boat has a per-second sandbox API today. A customer account follows the
  open-core decision, which rejected reselling minutes.
- GitHub gives each waiting job to any idle runner of the scale set. So
  ciproxy decides only how many runners start, and where. Data for one job,
  such as a cache disk, attaches after the job starts. Reason: the scale set
  API has no call that pins a job to a runner.
- Own machines keep warm runners that are registered before jobs arrive.
  Reason: jobs start at once. Warm runners also keep work going through a
  short control plane outage, because a queued ciproxy job cannot move to
  another label.
- Each job runs in a fresh VM. Reason: a job must not read the secrets or
  the files of another job.
- One image definition serves own machines and overflow snapshots. It
  starts from the open-source GitHub runner images. Reason: a job must
  behave the same wherever it runs.
- The cache layer is transparent to workflows. Cache scopes follow the rules
  of the GitHub cache. Only jobs on the default branch write back to shared
  caches. Reason: no workflow edits, and no cache poisoning from pull
  requests.
- Tier limits are soft. When a customer passes the starts or the own nodes
  of its tier, jobs keep running. The dashboard shows a warning and offers
  the tier or the add-on that covers the use. Reason: a pricing limit must
  never stop the CI of a customer.
- Stripe takes the payments. The customer picks the tier and the add-on
  quantities in the dashboard, pays through Stripe Checkout, and manages
  the subscription in the Stripe customer portal. Reason: no card data
  reaches ciproxy, and `futex-ai/accounting` already bills through Stripe.
- Add-ons are monthly quantities that the customer sets. ciproxy charges no
  overage by itself. Reason: the user wants manual control of the plan.
- Nothing stops jobs in the MVP: not a limit, and not a failed payment.
  When Stripe ends a subscription after its retries, the organization moves
  to the Free tier. Reason: a billing problem must never stop the CI of a
  customer. A hard limit is a new rejection, which needs a user decision.
- A warm runner that gets no job is not a start. A re-run of a job is a new
  start. Reason: the customer pays only for jobs that ran.

### Pricing

My proposal, which the user approved for the MVP on 3 November 2026. It
uses the CI of the `mokly-ai` organization as a reference: 17,772 starts
in 30 days, with up to 92 jobs at the same time. Mokly fits the Team tier.
The numbers can still change before launch.

| | Free | Team | Business |
| --- | --- | --- | --- |
| Price per month | $0 | $49 | $299 |
| Starts per month | 2,000 | 25,000 | 150,000 |
| Own nodes | 1 | 3 | 10 |
| Extra 10,000 starts | None | $10 | $10 |
| Extra own node, per month | None | $20 | $20 |

- Every tier: own Linux nodes, Boat overflow, the transparent cache, warm
  runners, the dashboard, and the savings report.
- Team adds: sticky Docker disks, alerts by email and Slack, 30 days of
  job history instead of 7, and email support.
- Business adds: several GitHub organizations, single sign-on, an audit
  log, custom VM images, 90 days of job history, and priority support.
- Later features join the tiers when they ship: AWS overflow and macOS
  nodes from Team, Blacksmith sandboxes and a customer cache bucket from
  Business.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The hosted dashboard, for every MVP product area, in mobile and desktop
variants. The repository has no mockup tooling yet, so this milestone sets
up Mokly as `futex-ai/accounting` does.

- [ ] Set up the Mokly catalogue under `docs/mockups`, with its
      configuration and the mockup check command.
- [ ] Sign-in and the installation of the GitHub App.
- [ ] Onboarding: add an own machine with one install command, connect a
      Boat account, and run the first job.
- [ ] Machines: the list and the detail, with slots, warm runners, health,
      and cache use.
- [ ] Jobs: the live queue and the history, with where each job ran.
- [ ] Savings: the cost of the jobs on own machines and on overflow,
      against GitHub-hosted and Blacksmith prices.
- [ ] Plans: compare the three tiers, and choose one.
- [ ] Plan and usage: the tier, the starts and own nodes of the month
      against the tier, the add-ons, the warning at a soft limit, and the
      way to the Stripe customer portal.
- [ ] Checkout: from a plan choice through Stripe Checkout to the
      confirmed plan, and the return after a failed payment.
- [ ] Settings: labels and machine shapes, the overflow wait time, the
      overflow spend limit, and repository access.
- [ ] User flows: from sign-in to the first green job, a peak in which
      jobs overflow to Boat, and a Free organization that passes its starts
      and upgrades to Team.
- [ ] Run the mockup check. Smoke-test the changed pages.
- [ ] Update the design and the TODOs of this plan for each problem that
      the mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

The docs state the requirements and the decisions for the implementation
plan. The design calls above give their first content.

- [ ] Update `README.md`: the product summary, the `runs-on` labels, and
      the developer get-started steps.
- [ ] Write `docs/architecture.md`: the components, their
      responsibilities, their boundaries, and the data flow.
- [ ] Write `docs/protocol/job-routing.md`: labels and shapes, the order of
      own machines and overflow, the overflow trigger, warm runners, and
      the behaviour when the control plane, an own machine, or a backend
      fails.
- [ ] Write `docs/protocol/agent.md`: install, the outbound-only connection,
      machine identity, a fresh VM for each job, the image, updates, health,
      and drain.
- [ ] Write `docs/protocol/overflow.md`: the backend contract, the customer
      account, teardown, the spend limit, and the cleanup of orphan VMs.
- [ ] Write `docs/protocol/cache.md`: transparent `actions/cache`, cache
      scopes, trusted write-back, the cache tiers, and the fallback to the
      GitHub cache.
- [ ] Write `docs/protocol/security.md`: GitHub App permissions, key
      custody, tenant isolation, job isolation, and secrets.
- [ ] Write `docs/protocol/metering.md`: what a start is, how ciproxy
      counts starts and own nodes for each month, the soft tier limits, and
      the rules for the savings report.
- [ ] Write `docs/protocol/billing.md`: the tiers, the add-ons, Stripe
      Checkout and the customer portal, the subscription states, failed
      payments, and the move to the Free tier.

## Milestone 3: Feasibility spikes

Depends on: 1

Each spike proves one risky mechanism on real infrastructure before the
implementation plan depends on it. Spike code stays under `.context/`, not
on the branch. Each spike adds one line here that names its evidence file
under `.context/ciproxy-mvp-foundation/spikes/`. The spikes need a GitHub
App in a test organization, a KVM-capable Linux server, and a Boat API key
from the user.

- [ ] Scale set: create a scale set, open a message session, run a job on a
      just-in-time runner, and record the fields of each job message. Use
      the Rust port `imumesh18/scaleset`. Where the port falls short, write
      a thin client that follows the Go client.
- [ ] Firecracker: run a job with Docker in a fresh VM on the Linux server.
      Measure the time from queue to first step, with and without a warm
      runner.
- [ ] Boat: run a just-in-time runner in a Boat sandbox that starts from a
      snapshot. Measure the boot time. Boat sells its own GitHub runner,
      so check that its terms allow this use. If they do not, AWS EC2
      becomes the first overflow backend.
- [ ] Cache: point `actions/cache` at a sidecar in the VM through a patched
      runner. Confirm read-through and write-through to the GitHub cache.
- [ ] Job hooks: attach a disk for one repository after the job starts,
      and start Docker on it.

## Milestone 4: Repository bootstrap

Depends on: 1

- [ ] Create the workspace skeleton as in `futex-ai/accounting`: a Rust
      workspace under `crates/`, the gate in `xtask/`, and a TypeScript
      package for the dashboard under `ts/`. Keep the agent in its own
      crates, so that a public copy can publish them at launch.
- [ ] Write `AGENTS.md` from the `agents-md` template. Create `CLAUDE.md` as
      a symlink to it.
- [ ] Add a `.gitignore` that holds `.context/`.
- [ ] Add the gate: format, lint, the file-length check, tests, and the
      Markdown link check.
- [ ] Add CI on GitHub-hosted runners. Move the CI of ciproxy to ciproxy
      when the first own machine runs jobs.
- [ ] Add the Apache-2.0 licence to the agent crates. Keep the rest of the
      repository without an open licence.

## Milestone 5: Implementation plan

Depends on: 2, 3, 4

- [ ] Align the protocol docs with the spike results. Propose each
      requirement change as a decision.
- [ ] Write `plans/ciproxy-mvp.md`. Give it milestones for the GitHub App
      and scale sets, the Linux agent, the scaler, the Boat backend, the
      cache layer, the dashboard backend and UI, metering, billing with
      Stripe, and the public copy of the agent at launch.
- [ ] Run the options check over its decisions.

## Milestone 6: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record
      follow-up items under `plans/follow-ups/` for the rest, re-review
      once, and report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.
