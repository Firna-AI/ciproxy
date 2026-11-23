# Overflow provider interface

Status: Active. On branch `calummoore/ciproxy-providers-interface-s8rp`, no
pull request yet. The pull request that completes the work deletes this plan,
as the `plans` skill says.

ciproxy starts overflow runners on Boat only. Boat names run through every
layer: the overflow service, the scaler, the store, the control plane, the
guest, metering, the dashboard API, the app, the mockups, the site, and the
protocol docs. A workspace can connect one Boat account, and each Linux label
has two switches, "Your machines" and "Boat". This change puts one
provider-neutral overflow interface in place of the Boat-only layer. Boat
becomes the first provider behind it. A workspace can connect several
providers, and each label sets the order of the places where its jobs run. A
later provider then adds an adapter, its seams, its facts, its image, and one
enum value with its store check and wire value. A provider with a new kind of
fact, such as a data disk or an egress cost, also adds the shared rule for
that fact.

The user chose this approach on 23 November 2026, in a brainstorm that
started from the request "create an interface for ciproxy providers (boat,
etc) that will allow add many different providers". The user's answers are
under [User decisions](#user-decisions). This change does not cover macOS
overflow or GPU sizes. A second real provider comes in its own plan and branch
after this change merges, as item 3 says.

Earlier work: the unmerged plan `plans/overflow-providers-and-instabox.md` on
branch `calummoore/instaboxsh-ci-provider-plan`, of 22 November 2026. It
designs the same interface with instabox as the second provider. This plan
reuses its interface design and its replaced ways. Its instabox milestones
move to the follow-up item of item 3. Notes of this brainstorm:
`.context/providers-interface/notes.md`.

Contract owners:

- [Overflow](../docs/protocol/overflow.md),
  [Overflow spend](../docs/protocol/overflow-spend.md), and
  [Sandbox end](../docs/protocol/overflow-sandbox-end.md).
- [Labels](../docs/protocol/labels.md),
  [Label order](../docs/protocol/labels-order.md),
  [Job routing](../docs/protocol/job-routing.md),
  [Job routing failures](../docs/protocol/job-routing-failures.md), and
  [Agent images](../docs/protocol/agent-images.md).
- [Security](../docs/protocol/security.md),
  [Metering](../docs/protocol/metering.md), and
  [Removing or suspending the App](../docs/protocol/app-removal.md).
- The [architecture doc](../docs/architecture.md).
- The READMEs of [ciproxy-overflow](../crates/ciproxy-overflow/README.md),
  [ciproxy-scaler](../crates/ciproxy-scaler/README.md),
  [ciproxy-schema](../crates/ciproxy-schema/README.md),
  [ciproxy-store-interface](../crates/ciproxy-store-interface/README.md),
  [ciproxy-store-pg](../crates/ciproxy-store-pg/README.md),
  [ciproxy-control-plane](../crates/ciproxy-control-plane/README.md),
  [ciproxy-guest](../crates/ciproxy-guest/README.md),
  [ciproxy-metering](../crates/ciproxy-metering/README.md),
  [ciproxy-api](../crates/ciproxy-api/README.md), the
  [image](../image/README.md), [deploy](../deploy/README.md), the
  [app](../ts/app/README.md), the [site](../ts/site/README.md), and the
  [mockups](../docs/mockups/README.md).

Replaced ways:

- Boat-named store tables, columns, and values → provider-neutral names with
  a `provider` column. Consumers: `ciproxy-schema`,
  `ciproxy-store-interface`, `ciproxy-store-pg`, `ciproxy-overflow`,
  `ciproxy-control-plane`, `ciproxy-metering`, and `ciproxy-api`. Moved in
  Milestone 3. Deleted in Milestone 3.
- One active account for each workspace → one active account for each
  workspace and provider. Consumers: the store index, the overflow service,
  the control-plane snapshot, and the dashboard API. Moved in Milestones 4,
  7, 8, and 9. Deleted in Milestone 4.
- The label columns `allow_machines`, `allow_boat`, and the value
  `largest_boat` → the label order and `largest_overflow`. Consumers: the
  store, the GitHub installation labels in
  `crates/ciproxy-github/src/app/installation.rs`, the registration queries
  in `crates/ciproxy-store-pg/src/github/registration.rs`, the scaler, the
  control-plane snapshot and `docker_budget.rs`, the size choice in
  `crates/ciproxy-overflow/src/service/provider_preparation.rs`, the label
  fit of `ciproxy-api`, the wire, and the app. Moved in Milestones 4, 6, 7,
  8, 9, and 10. Deleted in Milestone 4.
- `settings.boat_spend_limit_usd_micros` → one spend limit for each provider
  of a workspace. Consumers: the store, the overflow budget, the view, the
  wire, and the app. Moved in Milestones 4, 7, 9, and 10. Deleted in
  Milestone 4.
- `BoatAccountApi`, `BackendFactory`, `KeyWallets`, `AccountWallet`, and
  `OverflowConfig.shared_snapshot` → `ProviderAccountApi`,
  `ProviderFactory`, the provider registry, `KeyAccounts`, `BillingAccount`,
  and an image source for each provider. Consumers: the `ciproxy-overflow`
  service, its tests and live examples, and the control-plane composition.
  Moved in Milestone 5. Deleted in Milestone 5.
- The Boat types of `ciproxy-scaler`, such as `BoatAccount`, `BoatSize`,
  `BoatRunner`, `Destination::Boat`, `Snapshot.boat`, `place_boat`, and
  `select_boat_size` → provider-neutral types and a list of provider
  accounts. Consumers: the scaler, the overflow view, the control-plane
  snapshot, and the label fit of `ciproxy-api`. Moved in Milestone 6.
  Deleted in Milestone 6.
- The `boat_*` modules of the control plane and the `BOAT_KMS_KEY` setting →
  `overflow_*` modules and a neutral setting. Consumers: the control plane
  and its startup test, the Helm template `control-plane.yaml`, and the
  docs. The Terraform output `kms_key` and the deploy value `kmsKey` are
  already neutral. Moved in Milestone 8. Deleted in Milestone 8.
- The Boat mode of `ciproxy-guest` → a sandbox mode for every provider.
  Consumers: the guest, `image/ciproxy-boat-guest.service`,
  `image/check-runner-environment.py`, and the other image checks. Moved in
  Milestone 8. Deleted in Milestone 8.
- `JobPlace::Boat` and `RunnerPlace::Boat` of metering, and the stored value
  `"boat"` in the `facts` JSON of `metering_job_cursors` → one overflow place
  with the provider. Consumers: `ciproxy-metering`, its store queries, the
  saved cursor facts, and the savings views of the API. Moved in Milestones
  3 and 8. Deleted in Milestone 8.
- Boat-named wire types, fields, values, and problem codes → provider-neutral
  wire. Consumers: `ciproxy-api`, `ts/api/generated`, `ts/app/src/api`, and
  the app tests and harness scripts. Moved in Milestones 9 and 10. Deleted in
  Milestone 9.
- The chart colour token `chartColors.boat` of `ts/tokens/index.ts` → a
  neutral overflow token. Consumers: the mockups and the app. Moved in
  Milestone 10. Deleted in Milestone 10.
- Boat copy for rules of every provider → copy that names the provider.
  Consumers: the mockups, the app, and the site. Moved in Milestones 1, 10,
  and 11. Deleted in Milestone 11.
- Boat text for rules of every provider in the protocol and architecture
  docs → provider-neutral text. Moved in Milestone 2. Deleted in
  Milestone 2.

## Design

### The provider interface

- `OverflowProvider` is a closed enum with the value `boat`. The store keeps
  it as text with a check constraint. The API sends it as a string.
- A provider descriptor holds the fixed facts of a provider: the display
  name, the key hint, the guide link, the lifetime rule, the billing account
  kind, the disk model, the image source kind, and the platforms.
- The shared capabilities come from `sandbox-interface`: the lifecycle, the
  account capacity, and the files. A new adapter does not change them.
- Each provider supplies its seams through `ProviderFactory`:
  - `ProviderAccountApi` checks a key and lists its billing accounts. It
    reads the account facts: the name, the lifetime, the billing state, and
    the start-fee bound. It reads the runtime cost of one sandbox.
  - `GuestLauncher` prepares the private registration target and starts the
    guest once.
  - `GuestInspection` reads the launch marker without a change.
  - The image source says how a sandbox gets the job software. Boat uses a
    shared snapshot.
- The Boat-only rules move out of the shared service and into the Boat
  seams and descriptor:
  - the trial lifetime of 2 hours, now in
    `crates/ciproxy-overflow/src/service/preparation.rs`,
  - the start-fee bound, now required in
    `crates/ciproxy-overflow/src/service/start.rs`,
  - the `ciproxy` profile, now in
    `crates/ciproxy-overflow/src/service/execution.rs`,
  - the Boat paths `/workspace` and `/home/sandbox-workload`.
- The control-plane composition builds a provider registry. One
  `OverflowService`, behind the `Overflow` trait, serves every workspace. It
  picks the factory from the provider of each saved account and delivery.
- A provider without its deployment settings is unavailable. Setup and
  Settings show it as unavailable right now, as they show Boat today without
  `BOAT_SHARED_SNAPSHOT`.
- The rules of [Overflow](../docs/protocol/overflow.md) and
  [Sandbox end](../docs/protocol/overflow-sandbox-end.md) stay in the shared
  service for every provider: the size choice, the refusals, the spend, the
  end report, the orphan cleanup, the key change, and the App removal.

### Several providers and the label order

- A workspace can connect several providers. Each provider has at most one
  active key for the workspace, and its own key change and Disconnect.
- Each label has an order. An entry of the order is "own machines" or one
  provider. A label can list a provider before own machines, and it can
  leave own machines out.
- A macOS label's order holds only own machines.
- A waiting job goes to the next place of its order when the current place
  cannot take it now:
  - Own machines cannot take it after it waited the overflow wait, or at
    once when no own machine fits it.
  - A provider cannot take it when it is not connected, rejected, at its
    spend limit, waiting for a retry time, waiting for billing, unavailable
    in the deployment, or has no size that fits.
  - There is no extra wait between providers.
- The overflow wait applies only when a job leaves own machines.
- The workspace switch, today "Use Boat when your machines are busy", stays
  one switch for all providers. When it is off, every order skips every
  provider, also a provider that comes before own machines.
- "Run on the largest size" applies only when no place of the order fits.
  It uses the first provider of the order that jobs skip only because of
  its sizes, and whose largest size holds the complete image. Only a
  provider that is connected and available can be that provider. When no
  such provider exists, the job waits, as today.
- Each provider of a workspace has its own monthly spend limit. Each
  connection asks for the limit of its provider, as Boat does today.
- When a customer connects a provider that the workspace never offered
  before, ciproxy adds it to the end of the order of each Linux label. The
  customer can remove it from a label. `overflow_provider_settings` records
  that the workspace offered the provider.
- A new Linux label, from the customer or from the GitHub installation, gets
  own machines, then each offered provider in the order of its first
  connection. This keeps today's default, where both switches start on.
- A provider that is not connected stays in a label order and is skipped,
  so a key change or a reconnect needs no label edit.

### Data shape

- `overflow_provider_accounts` replaces `boat_accounts`, with `provider`.
  The unique index of active keys covers the workspace and the provider.
- `overflow_provider_settings` holds the spend limit of each workspace and
  provider. It outlives a key change and a Disconnect, so a new connection
  starts with the saved limit.
- `label_places` holds the order of each label: the label, the position,
  and the place, `machine` or a provider. The value `machine` matches the
  stored metering place. The pairs of label and place and of label and
  position are unique.
- The migration gives each existing label its order from its switches:
  both switches give own machines then Boat, one switch gives that place
  only. A label with both switches off gets an empty order. It also moves
  the saved spend limit to the Boat row of `overflow_provider_settings`.
- The migration marks Boat as offered for every workspace. Each label
  already holds a Boat choice, so a later first Boat key adds Boat to no
  label. This keeps today's rule that a connection changes no label.

### Design calls

My calls, 23 November 2026:

- Generic code, data, and copy use provider-neutral names. Boat names stay
  only for Boat-only parts: wallets, the free-trial lifetime, start fees,
  and the Boat image build. Reason: the user chose neutral names in every
  layer.
- "Overflow provider" replaces "backend" in docs and code. Product copy
  names the provider's brand, such as Boat. Reason: `AGENTS.md` reserves
  "workspace" and "machine" for other things.
- The store and the wire use a closed `OverflowProvider` enum. Reason: the
  compiler then finds each place that a new provider must handle.
- The scaler uses an opaque `ProviderId` and knows no provider. Reason: it
  stays a pure crate, and its tests can cover an order of two providers
  before a second real provider exists.
- The dashboard API puts the provider in the path of each provider route,
  as `/workspaces/{workspace_id}/overflow/providers/{provider}/...`. The
  workspace
  routes stay for the overflow switch and the overflow wait. Reason: each
  provider has its own key, wallet, and spend limit.
- Encrypted keys that are already saved stay readable. The authenticated
  data keeps the `boat-key` purpose for Boat, or a versioned format still
  reads v1. Reason: a renamed purpose would make saved keys unreadable.
- The KMS key `boat-account-keys` stays, and a neutral setting names it.
  Reason: Terraform protects the key from destroy, and GCP cannot rename a
  key.
- Every provider uses the existing exit callback route and token. The route
  moves to a neutral path. The old path works until no delivery that was
  saved before the deploy is unfinished. Reason: the route already serves
  one sandbox and needs no provider. A delivery seals its callback URL into
  its launch input, and an unsent delivery can start long after the deploy.
- The workspace switch, the new-label order, the offered mark, and the
  largest-size rule keep today's behaviour for a workspace with Boat only.
  Reason: the risk rule below, and no user decision changes them.
- The savings report has one overflow place with a row for each provider.
  Reason: one month can hold the costs of several providers.
- The decision "Boat is the only backend" becomes "a workspace can connect
  several providers, and Boat is the first". Reason: the user's answer to
  item 2.
- Mockups show the states with two providers with a fixture provider named
  "Second provider". The follow-up of item 3 replaces it with the real
  provider. Reason: the label order needs a second entry to design.
- Two words for two facts: a provider is "available" when the deployment
  has its settings, and "offered" when the workspace connected it once.
  Reason: the mockups showed that one word for both was unclear.

My calls from the mockups, 23 November 2026:

- When no place of a label's order can take a job, the Jobs notice gives
  the reason of the first place of the order that fits the size, such as
  "Overflow providers are off" or the spend limit of that provider. Reason:
  one short reason per label, as each wait reason has one notice today. A
  provider that is not connected or is unavailable counts as fitting, so
  its reason shows. Its sizes are unknown, and a skip could show a false
  reason. The label list, the order rows, and the Jobs notice take the
  reason from one function. An order row shows "off", "not connected", or
  "unavailable" before it says whether a size fits.
- A provider that counts as fitting for the wait reason never stops "Run
  on the largest size". Reason: the counting exists only to give a true
  reason, and the largest-size rule needs a provider that can start.
- Own machines keep warm runners for a label only while they are the first
  place of its order. Reason: the user's item 2 decision says that each
  label's order holds, and a ready warm runner would take jobs first.
- Protocol docs carry no status line for rules that later milestones of
  this plan build. Reason: the `documentation` skill allows status lines
  only on page kinds that `AGENTS.md` names, and it names none.
- Three protocol sections move to their own pages to stay near the length
  target: `overflow-spend.md`, `labels-order.md`, and
  `job-routing-failures.md`. Each old page keeps a pointer section.
- An unavailable provider keeps its account, spend, and Disconnect in
  Settings, and hides Change key. Reason: no new connection can start.
- The job history place filter wraps its choices onto a second line on a
  narrow screen. Reason: each provider adds a choice.
- "Add a place" in the label order offers own machines and each offered
  provider. Reason: it matches the offered mark and the append rule of item
  6. A provider that the workspace never connected joins the labels on its
  first connection.
- A macOS label's order is always own machines, with no handle, Remove, or
  Add. Reason: it keeps today's rule that a macOS label never overflows.
- The provider descriptor says whether a connection has a billing account
  choice. Boat checks the key and offers its wallets first. Reason: the
  wallet is a Boat fact.
- The descriptor holds facts only. The app keeps each provider's summary,
  key placeholder, key hint, and key error copy. Reason: copy belongs with
  the screens.
- Settings under Overflow shows the switch and the wait first, then one
  panel for each available provider. A notice about one provider names it
  and says when jobs start there again. Each order row says what that place
  does with a job of the label's size. Reason: the reader sees the rule for
  all providers once, then each provider.
- The live queue shows one running count for each provider, and the savings
  report and the job history filter have one row or choice for each
  provider. Reason: one month can hold jobs on several providers.
- The shared dialog scrolls its body between the title and the footer.
  Reason: the order rows pushed label dialogs past the top of a phone
  screen.

### Risks

- The label order changes routing for every label. The migration must keep
  the current behaviour of each existing label exactly.
- Each provider needs its own image build. No interface removes that cost.
- A provider that bills a cost with no bound, such as egress, breaks the
  full-lifetime spend bound. Such a provider needs a new spend rule first.

## User decisions

- **Item 1, decided on 23 November 2026:** C, provider-neutral names in every
  layer, with Boat as the first provider. Rejected: A, a provider trait only
  inside `ciproxy-overflow`. Rejected: B, a provider column and a registry
  with Boat names kept. Rejected: D, the seams moved into
  `sandbox-interface`, which can follow later. Reason: a rename costs least
  before production. This approves the renames of tables and API types on
  `origin/main`.
- **Item 2, decided on 23 November 2026:** a new option. A workspace can
  connect several providers, and each label sets its order, such as own
  machines, Boat, then another provider. The user's words: "you can have
  multiple providers, and for each label you set the order you want - e.g.
  local, boat, other provider". Rejected: one provider at a time, a
  workspace-wide order, the cheapest size across providers, and one
  provider for each label.
- **Item 4, decided on 23 November 2026:** A. A job goes to the next place
  when the current place cannot take it now, with no extra wait between
  providers. Rejected: B, a wait for each place. Rejected: C, a move after a
  missed ready deadline.
- **Item 5, decided on 23 November 2026:** B. One spend limit for each
  provider. Rejected: A, one limit for the workspace. Rejected: C, both.
- **Item 6, decided on 23 November 2026:** A. A newly connected provider
  joins the end of the order of each Linux label. Rejected: B, no label.
  Rejected: C, a choice of labels in the connect step.
- **Item 3, decided on 23 November 2026:** F. This change builds the
  interface with no stand-in provider. The second real provider, instabox by
  default, comes in its own plan and branch after this merge, with the
  two-provider tests outside the scaler. Rejected: A, a fake provider in the
  tests. Rejected: B, C, and D, a real provider in this change. Rejected: E,
  no plan for a second provider. Reason: the interface does not wait for a
  paid instabox account and three more answers. The follow-up item
  `plans/follow-ups/second-overflow-provider.md` holds that work.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The overflow screens show providers. Boat keeps its wallet and start-fee
states. Copy that names Boat for a rule of every provider names the provider.
The label dialog shows the label order.

- [x] Change the overflow step of Setup to a provider choice. Turn
      `app/onboarding/connect-boat` into a provider-neutral entry with a
      Boat branch.
- [x] Update Settings under Overflow: a list of providers, each with its
      account name, spend limit, spend, Change key, and Disconnect. Keep the
      workspace switch and the overflow wait.
- [x] Turn `app/settings/no-boat` into a state with no connected provider.
- [x] Replace the "Your machines" and "Boat" switches of the label dialog
      with the label order: own machines and each provider, in an order the
      customer can change, add to, and remove from. Show the empty order and
      a macOS label with own machines only.
- [x] Change "Run on the largest Boat size" to copy that names no single
      provider.
- [x] Update the Jobs notices, the live queue, and the job page so that they
      name the provider that ran a job and why the job went there.
- [x] Update the overflow panel of the Machines page and the savings rows,
      with a row for each provider.
- [x] Add the states with two providers, with the fixture "Second
      provider": Settings, the label order, the job page, and the savings.
- [x] Update the user flows `peak-overflow-to-boat`,
      `admin-changes-boat-key`, `admin-skips-boat`, and `first-green-job`.
- [x] Update the shared components and fixtures, such as `boat-fields.tsx`,
      `boat-spend.tsx`, `boat-wallets.tsx`, `fixtures/sizes.ts`, and
      `fixtures/boat-spend.ts`.
- [x] Run the mockup check named in `AGENTS.md`. Smoke-test the changed
      pages.
- [x] Add a state for a connected provider that becomes unavailable in the
      deployment: Settings keeps its account and Disconnect, and its order
      row says that jobs skip it.
- [x] Add one job history filter choice for each provider that ran jobs.
- [x] Make the site mockups provider-neutral where they state a rule of
      every provider: `labels-reference.tsx`, `quickstart.tsx`,
      `docs-home.tsx`, `docs-nav.ts`, `tiers.ts`, `hero-panel.tsx`,
      `hero.tsx`, `features.tsx`, `how-it-works.tsx`, `security.tsx`,
      `pricing.tsx`, `pricing-faq.tsx`, `add-machine-guide.tsx`, and the
      legal and terms fixtures.
- [x] Update the design and the TODOs of this plan for each problem that the
      mockups show.

Evidence: `.context/providers-interface/m1-screens/`.

## Milestone 2: Documentation and protocol

Depends on: 1

The protocol and architecture docs state the provider rules and the
decisions of this plan. A Boat-only rule stays under Boat.

- [x] In `docs/protocol/overflow.md`, use the provider terms. Replace
      "Decision: Boat is the only backend" with the decisions of items 1 and
      2 and their date. Keep the contract as the provider contract.
- [x] In `docs/protocol/overflow.md`, state that a workspace can connect
      several providers, one key for each, and the rule of item 6. Keep
      "The Boat account" for the Boat key, wallets, and trial.
- [x] In "Spend limit", state one limit for each provider, as item 5 says.
      Keep the start fee under Boat.
- [x] In `docs/protocol/overflow.md`, make "Overflow runners" and "Refused
      sandboxes" provider-neutral. In
      `docs/protocol/overflow-sandbox-end.md`, make the end report, "Orphan
      cleanup", and "After an App removal" provider-neutral.
- [x] In `docs/protocol/labels.md`, replace "Own machines, Boat, or both"
      with the label order. Make "A size that nothing can run"
      provider-neutral, with the first provider of the order that jobs skip
      only because of its sizes.
- [x] In `docs/protocol/job-routing.md`, state how a job moves along its
      order, as item 4 says. Make "When Boat cannot take a job", "What the
      dashboard shows", and "Boat fails" provider-neutral.
- [x] In `docs/protocol/security.md`, state key custody for every provider.
- [x] Make the other Boat text of `overflow-sandbox-end.md`, `metering.md`,
      `billing.md`, `app-removal.md`, `cache.md`, `docker-disks.md`,
      `agent.md`, `agent-images.md`, and `workspaces.md` provider-neutral
      where the rule holds for every provider.
- [x] Update `docs/architecture.md` and `docs/architecture/data-flow.md`:
      the provider registry and "A job on an overflow provider".
- [x] Update the overflow lines of `README.md`.

## Milestone 3: Store and schema: provider-neutral overflow records

Depends on: 2

The store keeps the same data under provider-neutral names, with a
`provider` column. Boat stays the only provider. Its behaviour does not
change.

- [ ] Add a migration that renames `boat_accounts` to
      `overflow_provider_accounts` and adds `provider`. Existing rows get
      `boat`. A check allows `boat`.
- [ ] In the same migration, rename `boat_runner_intents`, its index
      `boat_runner_intents_workspace`, and the view
      `boat_registration_ownership` to overflow names.
- [ ] In the same migration, rename `runners.boat_key_id`,
      `runners.boat_size_name`, and the index `runners_boat_key`.
- [ ] Rename the metering tables `metering_boat_costs`,
      `metering_daily_boat_costs`, and `metering_boat_cursors`, and the
      indexes `metering_boat_costs_workspace` and
      `metering_boat_cursors_workspace`. Add `provider` to the cost tables.
      Rename `boat_starts` and `boat_minutes` of `metering_daily_sizes`.
- [ ] Add the provider to the JSON records of `overflow_accounts`,
      `overflow_starts`, and the runner intents. Read a record without it as
      Boat.
- [ ] Rewrite the place `"boat"` in the `facts` JSON of
      `metering_job_cursors` to the overflow place with the provider `boat`.
- [ ] Rename the `boat`, `boat_delivery`, and `boat_runtime` modules of
      `ciproxy-store-interface` and their types, such as `EncryptedBoatKey`,
      `BoatCost`, `AccountedBoat`, and `DailyBoatCost`. Rename
      `RoutingSnapshot.boat_intents` and `RoutingSnapshot.boat_runners`. Add
      `OverflowProvider` to the account rows.
- [ ] Update `ciproxy-store-pg`, its tests, and each consumer that the
      rename touches, so that the workspace builds.
- [ ] Update the READMEs of `ciproxy-schema`, `ciproxy-store-interface`, and
      `ciproxy-store-pg`.

## Milestone 4: Store and schema: several providers and the label order

Depends on: 3

The store keeps one active key for each workspace and provider, one spend
limit for each provider, and the label order. Boat stays the only provider,
so behaviour does not change.

- [ ] Add a migration that changes the active-key index to the workspace and
      the provider.
- [ ] In the same migration, add `overflow_provider_settings` and move
      `settings.boat_spend_limit_usd_micros` to its Boat rows. Drop the old
      column.
- [ ] In the same migration, add `label_places`. Fill it from
      `allow_machines` and `allow_boat`, as "Data shape" says. Drop both
      columns.
- [ ] Change the stored value `largest_boat` of `labels.when_nothing_fits`
      to `largest_overflow`.
- [ ] In the same migration, mark Boat as offered for every workspace, as
      "Data shape" says.
- [ ] Add the store reads and writes of the label order and the provider
      settings. Add a write that appends a newly offered provider to each
      Linux label, for item 6.
- [ ] Give each new Linux label the order of "Several providers and the
      label order", also the labels of the GitHub installation in
      `crates/ciproxy-github/src/app/installation.rs`. Move the
      registration queries of `crates/ciproxy-store-pg/src/github/` to the
      order.
- [ ] Add store tests for the migration of each switch pair, the offered
      mark, the order writes, the append, the new-label order, and the
      per-provider limit.
- [ ] Update the READMEs of `ciproxy-schema`, `ciproxy-store-interface`, and
      `ciproxy-store-pg`.

## Milestone 5: Overflow service: the provider interface

Depends on: 3

`ciproxy-overflow` gets the provider interface of the design, with Boat as
its only provider. All Boat tests pass with no change of behaviour.

- [ ] Add `OverflowProvider` and the provider descriptor to the lowest crate
      that the store, the overflow service, metering, and the API share.
- [ ] Replace `BoatAccountApi` with `ProviderAccountApi`. Replace
      `KeyWallets` and `AccountWallet` with `KeyAccounts` and
      `BillingAccount`. Add the lifetime to `AccountFacts`.
- [ ] Replace `BackendFactory` with `ProviderFactory` and the provider
      registry. Replace `OverflowConfig.shared_snapshot` with an image
      source for each provider.
- [ ] Move the Boat-only rules of "The provider interface" into the Boat
      seams and descriptor.
- [ ] Move the Boat code behind the new seams in `src/boat/`. Keep its
      tests, the Boat live examples, and `LIVE.md` working.
- [ ] Repoint the `overflow.md#spend-limit` link of
      `crates/ciproxy-overflow/accounting.md` to `overflow-spend.md`.
- [ ] Add "Add a provider" to the `ciproxy-overflow` README: the adapter in
      `futex-ai/sandbox`, the enum value and descriptor, the seams under
      `src/<provider>/`, the image source, the connect form and copy, the
      protocol sections, the site guide, and the live probes.
- [ ] Update the README of the overflow crate.

## Milestone 6: Scaler: provider-neutral types and the label order

Depends on: 4

The scaler takes a list of provider accounts and walks the order of each
label, as item 4 says.

- [ ] Rename the Boat types of `ciproxy-scaler` to provider-neutral types
      with a `ProviderId`. Replace `Snapshot.boat` with a list of provider
      accounts.
- [ ] Replace `Destination::Boat` with a provider destination that names the
      provider and its size.
- [ ] Replace the label switches with the label order. Replace `place_boat`
      with one walk over the order.
- [ ] Apply "Run on the largest size" only when no place fits, with the
      first provider of the order that jobs skip only because of its sizes
      and whose largest size holds the image.
- [ ] Keep warm runners for a label only while own machines are the first
      place of its order.
- [ ] Skip every provider while the workspace switch is off.
- [ ] Keep every current scaler test passing on the order "own machines,
      then Boat".
- [ ] Add tests with two providers: the order, the skip of each unavailable
      reason, a provider before own machines, own machines left out, the
      overflow wait only after own machines, and the largest-size rule.
- [ ] Update the scaler README.

## Milestone 7: Overflow service: several providers for a workspace

Depends on: 5, 6

The service keeps one account for each provider of a workspace.

- [ ] Change `Overflow::wallets` to a key check that names the provider.
      Add the provider to `connect` and `disconnect`.
- [ ] Return the view of each connected provider, with its catalog, spend,
      and refusals, for the scaler snapshot.
- [ ] Check the spend limit of each provider in the budget.
- [ ] When a provider is offered for the first time, append it to each
      Linux label, as item 6 says.
- [ ] Read the label order in the size choice of
      `src/service/provider_preparation.rs`.
- [ ] Pick the factory of each delivery from its saved provider in start,
      resume, the end report, and cleanup.
- [ ] Add service tests through `PgStoreFactory` for connect, Disconnect,
      the spend limit, and the offered mark, with Boat.
- [ ] Update the README of the overflow crate.

## Milestone 8: Control plane, guest, and metering

Depends on: 7

The control plane composes the provider registry. The guest and metering use
provider-neutral names.

- [ ] Rename the `boat_*` modules of `ciproxy-control-plane` to `overflow_*`
      names that do not clash with the existing `overflow_composition`.
      Compose the provider registry at the composition root.
- [ ] Build the scaler snapshot with every provider account of a workspace
      and each label order. Move `snapshot.rs` and `docker_budget.rs` to the
      order.
- [ ] Replace `BOAT_KMS_KEY` with a neutral setting that names the same KMS
      key, in `src/config.rs`, its startup test, and the Helm template
      `control-plane.yaml`. Keep `BOAT_SHARED_SNAPSHOT` as the Boat image
      setting.
- [ ] Move the exit callback to a neutral path. Keep the old path while a
      delivery saved before the deploy is unfinished, and add a follow-up
      item that removes it.
- [ ] Rename the Boat mode of `ciproxy-guest` to a sandbox mode. Keep the
      private launch file and the exit callback. Update
      `image/ciproxy-boat-guest.service`,
      `image/check-runner-environment.py`, and the other image checks.
- [ ] Change `JobPlace::Boat` and `RunnerPlace::Boat` of metering to one
      overflow place with the provider. Give the daily costs and the savings
      report a row for each provider.
- [ ] Repoint the `overflow.md#spend-limit` link of
      `crates/ciproxy-metering/README.md` to `overflow-spend.md`.
- [ ] Update `crates/ciproxy-control-plane/overflow.md`,
      `crates/ciproxy-control-plane/overflow-smoke.md`, and the READMEs of
      the control plane, the guest, metering, and deploy.

## Milestone 9: Dashboard API: provider-neutral wire

Depends on: 8

The dashboard API sends the provider in every overflow body, the label order,
and the spend limit of each provider. The app's API layer moves in this
milestone, so that the gate stays green. Its screens move in Milestone 10.

- [ ] Add the provider routes of the design calls: the key check, connect,
      change key, Disconnect, and the spend limit.
- [ ] Give the live view one running count for each provider.
- [ ] Return a list of providers in `OverflowSettings`, each with its account
      view and spend. Keep the wallet types `BoatKeyCheck`, `BoatWallets`,
      `BoatWallet`, and `BoatConnection.wallet` for the Boat case only.
- [ ] Replace `allow_boat`, `LabelPlacement` with its values
      `MachinesThenBoat` and `BoatOnly`, and `NothingFits::LargestBoat` with
      the label order and `largest_overflow`.
- [ ] Rename the Boat wire types, fields, and values: `BoatLabelFit`,
      `BoatAccountView`, `BoatSpendView`, `LabelBoatSize`, the `LabelFit`
      fields `boat_connected`, `boat`, `largest_boat`, and
      `boat_required_root_disk_bytes`, `JobLocation::Boat`,
      `HistoryPlace::Boat`, `SetupPlace::Boat`, and the savings fields. Add
      the provider where the view names a provider.
- [ ] Replace `ProblemCode::BoatKeyRejected` with a key problem that names
      the provider.
- [ ] Regenerate the OpenAPI and `ts/api/generated`. Move `ts/app/src/api`
      to the new types with no visible change.
- [ ] Update the README and the docs files of `ciproxy-api`.

## Milestone 10: Dashboard UI: providers and the label order

Tags: ui

Depends on: 1, 9

The app shows the approved mockups.

- [ ] Move the provider-neutral parts of `ts/app/src/screens/boat` to
      overflow components. Keep the wallet choice for Boat only.
- [ ] Add the provider choice to Setup.
- [ ] Update Settings under Overflow to the list of providers, each with
      its spend limit.
- [ ] Replace the label switches with the label order control.
- [ ] Update the Jobs notices, the job page, the live queue, the Machines
      page, and the savings report.
- [ ] Update the app tests, the harness scripts, and the browser checks.
- [ ] Add one job history filter choice for each provider. Wrap the choices
      on a narrow screen.
- [ ] Make the shared dialog scroll its body between the title and the
      footer, as the mockups do.
- [ ] Replace `chartColors.boat` in `ts/tokens/index.ts` with a neutral
      overflow token, and move its consumers.
- [ ] Run the app typecheck, lint, tests, and export. Compare the screens
      with the mockups, and save screenshots under
      `.context/providers-interface/`.

## Milestone 11: Site: provider-neutral docs

Depends on: 2

Pages that name Boat for a rule of every provider become provider-neutral.

- [ ] Rename or split `guides/connect-boat.md` to match the mockup docs
      sidebar and Quickstart step "Connect an overflow provider". Keep the
      Boat key and wallet steps.
- [ ] Update `set-a-spend-limit.md`,
      `how-ciproxy-works.md`, `reference/labels.mdx`, `plans.mdx`, the
      security pages, the quickstart, and the index. Describe the label
      order.
- [ ] Update `tiers.ts` and `plan-compare.ts`.
- [ ] Run the site lint, typecheck, and build.

## Milestone 12: Second provider follow-up

Depends on: none

Item 3 chose F, so this change ships no second provider.

- [x] Add the follow-up item `plans/follow-ups/second-overflow-provider.md`
      for the second real provider and its two-provider tests.

## Milestone 13: Live checks before the merge

Depends on: 8

The live checks use the Boat test account. They need no deployment of the
branch.

- [ ] Run the Boat live probes, to prove that Boat still works behind the
      interface.
- [ ] Save the results under `.context/providers-interface/`.

## Milestone 14: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the fix rule: fix the `fix now` findings, record follow-up
      items under `plans/follow-ups/` for the rest, re-review once, and
      report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds.

## Post-merge follow-up (non-blocking)

- [ ] On the staging deployment, fill the own machines and run a job of a
      ciproxy label on Boat. Check the job page, the sandbox deletion, and
      the spend.
- [ ] Remove the old exit callback path when no delivery saved before the
      deploy is unfinished.
