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

Open decisions, 3 November 2026. They wait for the user. The options check
of the `plans` skill ran over them. Its report is in
`.context/ciproxy-mvp-foundation/options-check-1.md`.

### 3. Where the paid code lives

- **Severity:** High. **Effort:** Small.
- **Source:** Design.
- **Category:** Process.
- **Location:** The GitHub repository `Firna-AI/ciproxy`, which is public.
- **Terms:**
  - **Source-available licence:** a licence that lets anyone read the code
    but limits its use. The Functional Source License forbids a competing
    service and becomes Apache-2.0 after two years.
- **Background:**
  - `Firna-AI/ciproxy` is public. Each push publishes the code and the
    plans, with their pricing and competitor notes.
  - The open-core decision makes the agent open source and the hosted
    control plane paid.
  - The agent gets no jobs without a control plane. When only the agent is
    open, open-source users get the free allowance of the hosted service,
    not a free self-hosted product.
  - `Firna-AI` is on GitHub Free. There, a private repository has no branch
    rules. The Team plan adds them for $4 per seat per month.
  - The MVP serves private repositories only. So only a private repository
    lets ciproxy run its own CI on its own machines during the MVP.
  - `futex-ai/accounting`, another product of yours, is one private
    monorepo with Rust crates, a TypeScript app, and shared types.
- **Problem:** Control plane code in this public repository is public.
  Under an open licence, anyone can run the paid part for free. Published
  code cannot be taken back.
- **Impact if no action:** The branch cannot be pushed, and the repository
  layout cannot start.
- **Root question:** What does ciproxy publish, when, and under which
  licence, so that the MVP ships fast, the agent is open, and the control
  plane earns money?
- **Options:**
  - **A. One public repository, all Apache-2.0.** Everything is open.
    Anyone, also a competitor, can host the control plane. Effort: Small.
  - **B. One public repository with two licences.** The agent is
    Apache-2.0. The control plane is under the Functional Source License.
    Risk: that licence allows internal use, so a customer can run the
    control plane for itself without paying. Effort: Small.
  - **C. Make this repository private now.** At launch, publish the agent
    as open source from a public copy of its directory. One monorepo keeps
    the agent, the control plane, and the shared types together. Risk: no
    building in public before launch, and branch rules need the Team plan.
    Effort: Small.
  - **D. Two repositories now.** A public repository holds the agent, and a
    private one holds the control plane. Risk: the protocol between them
    crosses repositories, with two plans and two gates. Effort: Medium.
  - **E. Make this repository private now, and publish the agent from the
    first push.** A CI job copies the agent directory to a public
    repository on each merge to `main`. Risk: early agent code goes public
    before launch, and outside pull requests to the copy must be ported
    back by hand. Effort: Medium.
- **Left out:**
  - A closed agent: it breaks the open-core decision, and customers must
    trust code that runs on their machines.
  - The Fair Core License with a licence key for self-hosting: a key to run
    the control plane is a paid licence, which the business decision
    rejected.
- **Wider fix:** None. This is a business choice, not a class of defect.
- **Previous examples:** `futex-ai/accounting` is a private monorepo.
- **Related:** Item 4. The chosen stack sets the directory of the agent
  that a public copy or a second repository holds.
- **Recommendation:** C. It is the fastest layout that keeps the paid code
  and the plans private, and the agent still goes public at launch.
- **Action:** ask the user, because the options trade openness against
  protection of the paid part, and only the user can weigh that.

### 4. Language and stack

- **Severity:** High. **Effort:** Medium.
- **Source:** Design.
- **Category:** Code structure.
- **Location:** None.
- **Terms:**
  - **xtask:** a Rust convention for repository automation, such as the
    gate, written as a Cargo command.
- **Background:**
  - The official GitHub scale set client is Go: `actions/scaleset`, MIT,
    public preview since February 2026. It has about 4,300 lines of
    non-test code.
  - The TypeScript port `biw/scaleset` is at 2.0.0 on npm. The Rust port
    `imumesh18/scaleset` is at 0.1.0, first published on 29 October 2026.
  - The scale set client runs in the control plane, not in the agent.
  - The agent drives VMs. Firecracker, a common VM manager for CI, takes
    commands over HTTP on a Unix socket, so any language can drive it.
  - The shared skills give agent rules for Rust, Postgres with Diesel,
    TypeScript, and Rust-to-TypeScript types. They have no rules for Go or
    for a TypeScript server.
  - `futex-ai/accounting` uses Rust crates, `xtask` for the gate, a
    TypeScript app, Mokly mockups, and Stripe billing in Rust.
  - `falcondev-oss/github-actions-cache-server`, an open-source server for
    the GitHub cache protocol, is written in Rust.
- **Problem:** The stack sets the skeleton, the gate, and the agent rules.
  The bootstrap milestone needs it.
- **Impact if no action:** The bootstrap milestone and every later code
  milestone wait.
- **Root question:** Which languages should the agent, the control plane,
  and the dashboard use, so that agents build a correct MVP fastest?
- **Options:**
  - **A. Go for the agent and the control plane, TypeScript for the
    dashboard.** It uses the official client directly. Risk: no shared
    skill covers Go or the Go-to-TypeScript type generators, such as
    `tygo`. Effort: Medium.
  - **B. Rust for the agent and the control plane, TypeScript for the
    dashboard.** It matches the shared skills and the `futex-ai/accounting`
    setup, so the gate, the rules, and the billing code can be copied.
    Risk: the Rust port is new, so ciproxy may own a thin client that
    follows the Go one. Effort: Medium.
  - **C. TypeScript everywhere.** One language, with the mature TypeScript
    port. Risk: no shared skill covers a TypeScript server or its
    database, and a long-running VM agent on Node is weaker than a static
    binary. Effort: Medium.
  - **D. Rust for the agent, TypeScript for the control plane and the
    dashboard.** The control plane uses the TypeScript port and the
    official Stripe SDK. The agent protocol types come from Rust through
    `ts-rs`. Risk: the paid part gets the fewest shared rules, and the Rust
    billing code of `futex-ai/accounting` does not carry over. Effort:
    Medium.
- **Left out:**
  - A Go control plane with a Rust agent: with the TypeScript dashboard, it
    needs three languages and three gates, which cost more than a thin own
    client in B.
  - A Go agent with a TypeScript control plane: D is better on rules and
    on type generation.
  - A fork of GARM, a Go runner manager for one installation: a hosted
    service for many customers would rewrite its core.
- **Wider fix:** None. This is a design choice, not a class of defect.
- **Previous examples:** `futex-ai/accounting` uses the stack of option B.
- **Related:** Item 3. The layout of the paid and the open code depends on
  the stack.
- **Recommendation:** B. The shared Rust rules, the gate and the billing
  code of `futex-ai/accounting`, and a Rust cache server already exist, and
  the gap to the official client is small.
- **Action:** ask the user, because the options trade the official client
  and mature SDKs against the existing agent rules and setup.

### 5. Pricing unit of the paid control plane

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** None.
- **Terms:**
  - **Band:** one price step that covers a range of monthly job counts.
- **Background:**
  - Customers pay their overflow provider directly, under the open-core
    decision. ciproxy charges only for the control plane.
  - The control plane places every ciproxy job, on own machines and on
    overflow.
  - Each option needs a free allowance, because the open agent gets no jobs
    without the hosted control plane.
  - Actuated charges $150 per month for the first server and $125 for each
    additional server. RunsOn sells a yearly licence of €300 to €3,600 in
    bands of runners per month, with one runner for each job.
  - In December 2025, developers rejected a planned GitHub fee of $0.002
    per minute for self-hosted runners, and GitHub postponed it.
  - The billing mockup and the metering protocol need the unit.
- **Problem:** The paid product needs a price unit before its billing
  screen, its metering, and its launch.
- **Impact if no action:** The billing mockup and the metering protocol
  wait. The other work continues.
- **Root question:** What should a customer pay for, so that ciproxy earns
  money soon after launch with little billing work, and the price stays
  fair and predictable?
- **Options:**
  - **A. Per own machine per month.** Simple. Risk: a 96-core server pays
    the same as a Mac mini. Effort: Small.
  - **B. Per vCPU of own machines per month.** It grows with the fleet and
    stays predictable. Metering counts only the connected vCPUs. Risk:
    overflow jobs pay nothing, although the control plane places them.
    Effort: Small.
  - **C. Per job-minute on own machines.** It grows with use. Risk: it
    looks like the GitHub fee that developers rejected. Effort: Small.
  - **D. A share of the measured savings.** It matches the value. Risk:
    customers dispute the baseline price, and the metering is complex.
    Effort: Medium.
  - **E. A flat monthly price for each GitHub organization, with no
    meter.** Risk: a large fleet pays the same as a small one, and a later
    move to a unit changes the price for existing customers. Effort: Small.
  - **F. A monthly price in bands of jobs.** The control plane counts each
    job that a ciproxy runner runs, on own machines and on overflow. A
    customer moves up a band only after two months above it. Risk: a short
    lint job counts the same as a long build. Effort: Small.
- **Left out:**
  - Per seat: CI load grows with agents and pull requests, not with people.
  - Free forever: it fails the goal to make money.
  - A share of the overflow bill: it pays ciproxy more when jobs leave own
    machines, which works against the goal of the customer.
- **Wider fix:** None. This is a business choice, not a class of defect.
- **Previous examples:** `futex-ai/accounting` bills per unit as a Stripe
  subscription quantity. Actuated is a precedent for A, and RunsOn for F.
- **Recommendation:** F, changed from B by the options check. Jobs grow
  with agents and pull requests, which drive CI load, and bands keep the
  bill predictable with no per-minute charge.
- **Action:** ask the user, because this is a business choice with real
  trade-offs between fairness, predictability, and revenue.

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
- [ ] Settings: labels and machine shapes, the overflow wait time, the
      overflow spend limit, and repository access.
- [ ] Billing and plan. Waits for item 5.
- [ ] User flows: from sign-in to the first green job, and a peak in which
      jobs overflow to Boat.
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
- [ ] Write `docs/protocol/metering.md`: what ciproxy counts for the
      savings report and for billing. Waits for item 5.

## Milestone 3: Feasibility spikes

Depends on: none

Each spike proves one risky mechanism on real infrastructure before the
implementation plan depends on it. Spike code stays under `.context/`, not
on the branch. Each spike adds one line here that names its evidence file
under `.context/ciproxy-mvp-foundation/spikes/`. The spikes need a GitHub
App in a test organization, a KVM-capable Linux server, and a Boat API key
from the user.

- [ ] Scale set: create a scale set, open a message session, run a job on a
      just-in-time runner, and record the fields of each job message. Use
      the scale set client of the stack that item 4 chooses.
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

Depends on: none. It waits for items 3 and 4.

- [ ] Create the workspace skeleton for the chosen stack.
- [ ] Write `AGENTS.md` from the `agents-md` template. Create `CLAUDE.md` as
      a symlink to it.
- [ ] Add a `.gitignore` that holds `.context/`.
- [ ] Add the gate: format, lint, the file-length check, tests, and the
      Markdown link check.
- [ ] Add CI on GitHub-hosted runners. Move the CI of ciproxy to ciproxy
      when the first own machine runs jobs.
- [ ] Add the licence files that item 3 chooses.

## Milestone 5: Implementation plan

Depends on: 2, 3, 4

- [ ] Align the protocol docs with the spike results. Propose each
      requirement change as a decision.
- [ ] Write `plans/ciproxy-mvp.md`. Give it milestones for the GitHub App
      and scale sets, the Linux agent, the scaler, the Boat backend, the
      cache layer, the dashboard backend and UI, metering, and billing.
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
