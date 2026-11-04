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
  a number of own machines. Add-ons buy extra starts and extra own
  machines. A
  start is one job that a ciproxy runner starts, on an own machine or on
  overflow. The customer picks the tier by hand. This replaces the first
  choice of the same day: a monthly price in bands of jobs. Rejected
  before that: a price per own machine, per vCPU, or per job-minute, a
  share of the savings, and a flat price. Reason: starts grow with agents
  and pull requests, which drive CI load, and the machine count is simple to
  check.
- Billing: in-product billing is part of the MVP, with the tiers and the
  add-ons below. The user approved them and this plan on 3 November 2026,
  in the words "ok, looks good - lets go ahead". This withdraws my earlier
  call to leave billing out of the MVP. The user then limited the work to
  Milestone 1, in the words "impl milestone 1 only". On 4 November 2026
  the user gave the go-ahead for every milestone after the mockups, in the
  words "after mockups - continue to impl all plan milestones and tasks".
- Visual direction: Graphite, light and monochrome in the manner of Vercel,
  with Geist and Geist Mono, hairline borders, small corner radii, and
  black primary buttons. The user chose it on 3 November 2026, after
  calling the first look childish. Rejected: Midnight, a dark console,
  because every state and the Stripe screens need dark work. Rejected:
  Enterprise, with a navy sidebar, because its chrome is heavier. Rejected:
  Graphite with Midnight as a later dark mode, because every screen would
  need two themes. Reason: it looks like the best developer tools that the
  buyers compare against, and it costs the least to apply.
- Term: the product calls the customer's own hardware a "machine"
  everywhere, also in the pricing. The user chose it on 3 November 2026.
  Rejected: "node", the word of the first pricing draft, because some
  readers think of Kubernetes. Rejected: "server", because it reads oddly
  for a Mac mini on a desk.
- Marketing site and docs: ciproxy gets a marketing landing page and a
  docs site. Both are built with Astro. The user asked for them on
  3 November 2026, with mockups in Milestone 1 and the build in the
  implementation plan.
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
- Tier limits are soft. When a customer passes the starts or the own machines
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

- One Astro project serves the marketing pages at `/` and the docs at
  `/docs`. The docs use Starlight, the docs framework of Astro, themed with
  the Graphite tokens. Reason: one build and one deploy, search and docs
  navigation without custom work, and one look across the site and the
  dashboard.

Design calls from the mockups, my calls, 4 November 2026. The mockups are
part of the spec, or the call follows a Stripe default. Milestone 2 writes
them into the protocol docs, where the user reviews each requirement.

- A paid tier counts over its Stripe billing period. Free counts over a
  month from its start day. The Boat spend limit counts over the calendar
  month.
- A move from Free to a paid tier starts a new period at checkout. A move
  between paid tiers keeps the period, with Stripe proration, and the
  starts so far count against the new tier.
- The machines of a tier are the registered machines, offline ones
  included.
- A job is a start when a runner takes it, whatever its result.
- Business shares one pool of starts and machines across the
  organizations of one subscription.
- Add-on changes use Stripe proration. A manual move to Free takes effect
  at the end of the paid period. Failed payments follow the Stripe retry
  settings before the move to Free.
- The savings report keeps daily totals for 30 days on every tier, also
  when Free keeps job details for 7 days. It uses a monthly cost for each
  machine that the customer enters.
- At the Boat spend limit, or after Boat rejects the key, jobs run on own
  machines only and wait there for a free slot.
- A slot is 2 vCPU and 8 GB. A larger size takes several slots. A size
  that fits no own machine runs on Boat only.
- Each machine keeps 2 warm runners by default, and the customer can
  change the count for each machine.
- The overflow wait is 0, 15, 30, 60, or 120 seconds, 30 by default.
- The agent reports to the control plane every 10 seconds. A machine
  without contact for 60 seconds shows as offline.

### Pricing

My proposal, which the user approved for the MVP on 3 November 2026. It
uses the CI of the `mokly-ai` organization as a reference: 17,772 starts
in 30 days, with up to 92 jobs at the same time. Mokly fits the Team tier.
The numbers can still change before launch.

| | Free | Team | Business |
| --- | --- | --- | --- |
| Price per month | $0 | $49 | $299 |
| Starts per month | 2,000 | 25,000 | 150,000 |
| Own machines | 1 | 3 | 10 |
| Extra 10,000 starts | None | $10 | $10 |
| Extra own machine, per month | None | $20 | $20 |

- Every tier: own Linux machines, Boat overflow, the transparent cache, warm
  runners, the dashboard, and the savings report.
- Team adds: sticky Docker disks, alerts by email and Slack, 30 days of
  job history instead of 7, and email support.
- Business adds: several GitHub organizations, single sign-on, an audit
  log, custom VM images, 90 days of job history, and priority support.
- Later features join the tiers when they ship: AWS overflow and macOS
  machines from Team, Blacksmith sandboxes and a customer cache bucket from
  Business.

Open decisions, 4 November 2026. They wait for the user. The site mockups
showed them. The options check of the `plans` skill ran over them and
changed each recommendation. Its report is in
`.context/ciproxy-mvp-foundation/options-check-2.md`.

### 8. Tier features have no screens

- **Severity:** High. **Effort:** Medium.
- **Source:** Implementation.
- **Category:** Docs or spec.
- **Location:** `docs/mockups/src/fixtures/tiers.ts`,
  `docs/mockups/src/fixtures/plan-compare.ts`, and the Pricing section of
  this plan.
- **Background:** The approved pricing gives Team two extra features:
  sticky Docker disks, and alerts by email and Slack. Business adds four:
  several GitHub organizations, single sign-on, an audit log, and custom
  VM images. This plan designs two of them: the job-hook spike of
  Milestone 3 covers the sticky Docker disk, and a design call shares one
  pool across the organizations of a Business subscription. The other four
  have no design. The pricing page and the dashboard Plans page list all
  six, and the Business card description names two of them.
- **Problem:** The site sells four features that the plan neither designs
  nor builds, and the Milestone 5 plan names none of the six.
- **Impact if no action:** Customers pay for Team or Business and find
  features that do not exist, or the MVP grows late when someone finds
  the gap.
- **Root question:** What does ciproxy promise for Team and Business at
  launch?
- **Options:**
  - **A.** Keep all six features in the tiers, and mark each one without a
    screen as "Coming later" on every page that lists it. Effort: Small.
  - **B.** Move all six features to the later-features line of the
    pricing. Effort: Small.
  - **C.** Build the cheap features, such as email alerts, in the MVP, and
    mark the others "Coming later". Effort: Medium.
  - **D.** Design and build all six features in the MVP. This delays the
    paid launch. Effort: Large.
  - **E.** Keep the two features that the plan designs: sticky Docker disks
    on Team, and several GitHub organizations on Business. Add their
    missing mockup states, and name both in the Milestone 5 plan. Move
    alerts, single sign-on, the audit log, and custom VM images to the
    later-features line. Effort: Medium.
- **Left out:**
  - Removing the features from the site only: the dashboard Plans page
    would still list them.
  - Launching with Free and Team only: it breaks the decision of three
    tiers, and Team with add-ons costs about the same as Business.
  - Selling Business through "Contact us": it breaks the decision to bill
    every tier in the product.
- **Wider fix:** One record for each feature in the tier fixture, with its
  first tier and whether it ships at launch, used by every page and by the
  comparison table. A Milestone 5 TODO then names the milestone that
  builds each launch feature. This works with A, B, or E.
- **Previous examples:** None found.
- **Recommendation:** E, changed from A by the options check. Limits are
  soft, so features that only paid tiers get are the main reason to pay,
  and E keeps one real feature in each paid tier while it sells only what
  the MVP builds.
- **Action:** ask the user, because the recommended option changes the
  features of the approved pricing, and no recorded decision covers that.

### 9. The cost claim needs a price basis

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation.
- **Category:** UX wording.
- **Location:** `docs/mockups/src/screens/marketing/cost-compare.tsx`,
  `docs/mockups/src/fixtures/site.ts`, and the peak-overflow user flow.
- **Background:** The landing page compares 30 days of the Linux CI of
  `mokly-ai`: $309.20 with ciproxy, $698 on Blacksmith, and $1,045 on
  GitHub-hosted runners. The ciproxy total uses three Hetzner AX42 servers
  at the auction price of $53.40 each. Hetzner lists a new AX42 at €99 a
  month, which gives about $494. The page says 17,772 jobs, but 16,350 of
  them run on Linux. The peak-overflow flow repeats the claim. Boat also
  sells its own runner, and the options check estimates the same Linux
  jobs on Boat alone at about $360 to $760 a month, mostly start charges.
- **Problem:** The title "less than half the price" holds only with auction
  prices, and the job count includes jobs that the MVP cannot run.
- **Impact if no action:** A visitor who checks the prices finds the
  headline wrong, which damages trust before the first sign-in.
- **Root question:** Which cost claim can ciproxy make in public and
  defend?
- **Options:**
  - **A.** Keep the auction prices and the title, and name the Hetzner
    server auction in the caption. Effort: Small.
  - **B.** Use new-server prices with a fixed claim, such as "about $200
    less a month". Effort: Small.
  - **C.** Show a range, such as "30 to 55% less than Blacksmith", with
    both bases in the caption. Effort: Small.
  - **D.** Use a title with no fixed number, such as "The same jobs for
    less". The fixture computes the numbers from the list price of a new
    AX42, and the caption names each price source and its date. It can add
    that auction servers cost less. Effort: Small.
  - **E.** Add Boat's own runner to the comparison, and claim only what
    holds against it. ciproxy may cost about the same as Boat alone, so
    the section may argue speed instead: warm runners and the local cache.
    Effort: Medium.
  - **F.** Replace the fixed example with a calculator, where visitors
    enter their own jobs and machine price. Effort: Large.
- **Left out:** Removing the comparison: the hero says "Pay less for CI"
  and needs evidence.
- **Wider fix:** D. A title with no fixed number cannot drift when a price
  changes, and the fixture computes every number, also in the user flow.
- **Previous examples:** None found.
- **Recommendation:** D, changed from C by the options check. It stays
  true at any server price and after each price change, and the computed
  saving still gives a concrete figure. Every option also fixes the job
  count to the 16,350 Linux jobs.
- **Action:** ask the user, because the options trade a stronger headline
  against a claim that any buyer can check.

### 10. Paid-tier buttons drop the chosen tier

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation.
- **Category:** Docs or spec.
- **Location:** `docs/mockups/src/screens/marketing/price-cards.tsx`
- **Background:** On the pricing page, Free has "Start free", and Team and
  Business have "Get started". All three open sign-in. Every new
  organization starts on Free and picks a paid tier in the dashboard.
  When an organization passes the Free limit, a banner on every page
  offers the tier that covers its use.
- **Problem:** A visitor who selects "Get started" on Team is on Free after
  setup, and must choose Team again in the dashboard.
- **Impact if no action:** Some visitors who want to pay at once find the
  upgrade later, so their first payment comes later.
- **Root question:** How does a visitor who picks a paid tier on the site
  get to that tier?
- **Options:**
  - **A.** Keep the mockup. A caption explains the upgrade. Effort: Small.
  - **B.** Carry the tier through sign-in. After setup, Plans opens with
    that tier selected and a button to Stripe Checkout. This needs a rule
    for who in an organization may pay. Effort: Medium.
  - **C.** Open Stripe Checkout before setup. Buyers pay before their first
    job runs, so more of them may leave. Effort: Medium.
  - **D.** Label every button "Start free", as the hero and the other calls
    to action do. The soft-limit banner moves each growing organization to
    the right tier. Effort: Small.
- **Left out:**
  - A free trial of the chosen tier: the first payment waits, and with soft
    limits a trial gives little more than Free.
  - Paying on the site before sign-in: it breaks the design call that the
    customer picks the tier in the dashboard, and C gives the same early
    payment.
- **Wider fix:** None. D also gives every sign-in button the same label.
- **Previous examples:** None found.
- **Recommendation:** D, changed from B by the options check. It is the
  cheapest, adds no behaviour at sign-in, and the banner already sends each
  organization that outgrows Free to the right tier.
- **Action:** ask the user, because the options trade early payment from
  buyers who arrive decided against a simpler flow.

### 11. Two words name one VM size

- **Severity:** Low. **Effort:** Small.
- **Source:** Implementation.
- **Category:** UX wording.
- **Location:** `docs/mockups/src/screens/settings/`, the docs labels page,
  the job and machine detail screens, and the code names with "shape".
- **Background:** A label in `runs-on` gives a job a VM with vCPUs, memory,
  and a disk. The dashboard calls this a "shape", and the docs and the
  savings note call it a "size". Some text says "machine size" or "machine
  shape", which uses "machine" for the VM, although "machine" means the
  customer's own hardware since 3 November 2026.
- **Problem:** The product uses several terms for one thing.
- **Impact if no action:** A reader who moves between the docs and the
  dashboard may think that these are different things, and the
  implementation copies the mixed words into its types.
- **Root question:** Which one term names the vCPU, memory, and disk that a
  label gives a job, in the dashboard, the docs, and the code?
- **Options:**
  - **A.** Use "size" in the screens that the item first listed. Effort:
    Small.
  - **B.** Use "shape" everywhere. Effort: Small.
  - **C.** Use "size", and "VM size" where the word needs a qualifier.
    Never write "machine size". Rename the code and catalogue names, and
    add the word to the word rules of the mockups README. Effort: Small.
  - **D.** Use no size word, only "VM", as in "A larger VM takes several
    slots". Effort: Small.
- **Left out:** Keeping several words: it fails the goal.
- **Wider fix:** C. A word rule in the mockups README and matching code
  names carry one word into the implementation.
- **Previous examples:** "Machine" against "node" and "server", decided on
  3 November 2026.
- **Recommendation:** C, changed from A by the options check. It is A plus
  the places that A misses, and it keeps "machine" for the customer's own
  hardware only.
- **Action:** ask the user, because the options are correct words, and no
  copy rule picks one.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The hosted dashboard, for every MVP product area, in mobile and desktop
variants. The repository has no mockup tooling yet, so this milestone sets
up Mokly as `futex-ai/accounting` does.

- [x] Set up the Mokly catalogue under `docs/mockups`, with its
      configuration and the mockup check command.
- [x] Sign-in and the installation of the GitHub App.
- [x] Onboarding: add an own machine with one install command, connect a
      Boat account, and run the first job.
- [x] Machines: the list and the detail, with slots, warm runners, health,
      and cache use.
- [x] Jobs: the live queue and the history, with where each job ran.
- [x] Savings: the cost of the jobs on own machines and on overflow,
      against GitHub-hosted and Blacksmith prices.
- [x] Plans: compare the three tiers, and choose one.
- [x] Plan and usage: the tier, the starts and own machines of the month
      against the tier, the add-ons, the warning at a soft limit, and the
      way to the Stripe customer portal.
- [x] Checkout: from a plan choice through Stripe Checkout to the
      confirmed plan, and the return after a failed payment.
- [x] Settings: labels and machine shapes, the overflow wait time, the
      overflow spend limit, and repository access.
- [x] Marketing site: the landing page and the pricing page, in the
      Graphite look.
- [x] Docs site: the docs home, the getting-started guide, and a reference
      page, in the Graphite look.
- [x] User flows: from sign-in to the first green job, a peak in which
      jobs overflow to Boat, a Free organization that passes its starts and
      upgrades to Team, and a visitor who goes from the landing page to
      sign-in.
- [x] Design directions: show three professional visual directions on key
      screens. Apply the direction that the user picks to every screen.
- [x] Fix the findings of the first screenshot review. They are in
      `.context/ciproxy-mvp-foundation/reviews/mockup-review-1.md`.
- [x] Run the mockup check. Smoke-test the changed pages.
- [ ] Apply the answers to items 8 to 11 in the mockups. Count only the
      Linux jobs where the landing page and the savings screen compare
      Linux prices.
- [ ] Update the design and the TODOs of this plan for each problem that
      the mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

The docs state the requirements and the decisions for the implementation
plan. The design calls above give their first content.

- [x] Update `README.md`: the product summary, the `runs-on` labels, and
      the developer get-started steps. Branch:
      `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/architecture.md`: the components, their
      responsibilities, their boundaries, and the data flow. Branch:
      `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/protocol/job-routing.md`: labels and shapes, the order of
      own machines and overflow, the overflow trigger, warm runners, and
      the behaviour when the control plane, an own machine, or a backend
      fails. Name the default labels and the VM that each one starts, as
      the docs mockups show: `ciproxy`, `ciproxy-4vcpu`, and
      `ciproxy-8vcpu`. Define the host reserve, so that the smallest
      machine, 4 cores and 16 GB, gets at least one slot. Branch:
      `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/protocol/agent.md`: install, the outbound-only connection,
      machine identity, a fresh VM for each job, the image, updates, health,
      and drain. Branch: `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/protocol/overflow.md`: the backend contract, the customer
      account, teardown, the spend limit, the cleanup of orphan VMs, and
      the behaviour when Boat refuses a start because of its start limits.
      Branch: `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/protocol/cache.md`: transparent `actions/cache`, cache
      scopes, trusted write-back, the cache tiers, and the fallback to the
      GitHub cache. Branch: `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/protocol/security.md`: GitHub App permissions, key
      custody, tenant isolation, job isolation, and secrets. Branch:
      `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/protocol/metering.md`: what a start is, how ciproxy
      counts starts and own machines for each month, the soft tier limits, and
      the rules for the savings report. Branch:
      `conductor/ciproxy-mvp-foundation-m2-docs`.
- [x] Write `docs/protocol/billing.md`: the tiers, the add-ons, Stripe
      Checkout and the customer portal, the subscription states, failed
      payments, the move to the Free tier, and who in an organization may
      choose a plan and pay. Branch: `conductor/ciproxy-mvp-foundation-m2-docs`.

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
      becomes the first overflow backend. Measure the start limits of the
      Boat plan, and record what Boat returns when it refuses a start.
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
- [ ] Add the Astro package for the marketing site and the docs under
      `ts/site`, with Starlight, and add its build to the gate. Use light
      mode only with no theme switch, give notes a full border instead of
      a left accent edge, and use the shared site top bar.
- [ ] Move the Graphite tokens into a shared package under `ts/`, which the
      mockups, the dashboard, and the site import.
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
      Stripe, the marketing site and the docs in Astro, and the public copy
      of the agent at launch.
- [ ] Give the site milestone of that plan the 14 launch docs pages that
      the docs sidebar mockup lists, and a launch task for the terms of
      service and the privacy policy, which the user writes or approves.
- [ ] Order the launch steps in that plan: the public copy of the agent
      goes live before the site, because the site links to it.
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
