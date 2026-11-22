# Overflow providers and instabox

Status: Active. On branch `calummoore/instaboxsh-ci-provider-plan`, no pull
request yet. The pull request that completes the work deletes this plan, as
the `plans` skill says.

ciproxy starts overflow runners on Boat only. Boat names run through every
layer: the overflow service, the scaler, the store, the control plane, the
guest, metering, the dashboard API, the app, the mockups, the site, and the
protocol docs. This change puts one provider-neutral overflow interface in
place of the Boat-only layer. Boat becomes the first provider behind it, and
[instabox](https://instabox.sh) the second. A later provider, such as AWS
EC2 or Blacksmith, then adds an adapter and its provider facts. The shared
layers do not change for it.

The user chose this approach on 22 November 2026, in two messages:
"instabox.sh add this as a CI provider option - create a plan" and "create
a unified interface so we can easily add other providers?". That choice
rejects these approaches:

- A second Boat-like code path beside Boat. Reason: each later provider
  would copy it again.
- instabox in place of Boat. Reason: the user asked for an option.
- Jobs sent to instabox's own GitHub App, which serves the
  `instabox-medium` labels. Reason: ciproxy must stay the runner provider of
  its own labels, as [Labels](../docs/protocol/labels.md) says.
- The agent on an always-on instabox machine, as an own machine. Reason: the
  agent needs KVM for its job VMs, and an always-on machine bills each hour.

This change does not cover macOS overflow, GPU sizes, or instabox's own CI
product. instabox rents a Mac only with a 24-hour lease. So macOS jobs still
never overflow, as
[Overflow runners](../docs/protocol/overflow.md#overflow-runners) says.
The user decided two items on 22 November 2026, under
[User decisions](#user-decisions). Four decisions still wait for the user,
under [Open decisions](#open-decisions). Milestone 3 needs a paid instabox
workspace and a workspace API key from the user.

Contract owners:

- [Overflow](../docs/protocol/overflow.md).
- [Labels](../docs/protocol/labels.md),
  [Job routing](../docs/protocol/job-routing.md), and
  [Agent images](../docs/protocol/agent-images.md).
- [Security](../docs/protocol/security.md),
  [Metering](../docs/protocol/metering.md), and
  [Removing or suspending the App](../docs/protocol/app-removal.md).
- The [architecture doc](../docs/architecture.md).
- The READMEs of [ciproxy-overflow](../crates/ciproxy-overflow/README.md),
  [ciproxy-scaler](../crates/ciproxy-scaler/README.md),
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
  Milestone 4. Deleted in Milestone 4.
- `BoatAccountApi`, `BackendFactory`, `KeyWallets`, `AccountWallet`, and
  `OverflowConfig.shared_snapshot` → `ProviderAccountApi`,
  `ProviderFactory`, the provider registry, `KeyAccounts`, `BillingAccount`,
  and an image source for each provider. Consumers: the `ciproxy-overflow`
  service, its tests and live examples, and the control-plane composition.
  Moved in Milestone 5. Deleted in Milestone 5.
- The Boat types of `ciproxy-scaler`, such as `BoatAccount`, `BoatSize`,
  `BoatRunner`, and `select_boat_size` → provider-neutral types. Consumers:
  the scaler, the overflow view, the control-plane snapshot, and the label
  fit of `ciproxy-api`. Moved in Milestone 5. Deleted in Milestone 5.
- The `boat_*` modules of the control plane and the `BOAT_KMS_KEY` setting
  → `overflow_*` modules and a neutral setting. Consumers: the control
  plane, Helm, the Terraform outputs, and the deploy scripts and tests.
  Moved in Milestone 6. Deleted in Milestone 6.
- The Boat mode of `ciproxy-guest` → a sandbox mode for every provider.
  Consumers: the guest, `image/boat-run.sh`, the Boat units, and the image
  checks. Moved in Milestone 6. Deleted in Milestone 6.
- The Boat place of metering → one overflow place with a row for each
  provider. Consumers: `ciproxy-metering`, its store queries, and the
  savings views of the API. Moved in Milestone 6. Deleted in Milestone 6.
- Boat-named wire types, fields, values, and problem codes → provider-neutral
  wire. Consumers: `ciproxy-api`, `ts/api/generated`, `ts/app/src/api`, and
  the app tests and harness scripts. Moved in Milestones 7 and 11. Deleted
  in Milestone 7.
- Boat copy for rules of every provider → copy that names the connected
  provider. Consumers: the mockups, the app, and the site. Moved in
  Milestones 1, 11, and 12. Deleted in Milestone 12.
- Boat text for rules of every provider in the protocol and architecture
  docs → provider-neutral text. Moved in Milestone 2. Deleted in
  Milestone 2.

## Terms

- **Overflow provider:** a cloud service where ciproxy starts overflow
  runners in the customer's own account. Boat and instabox are the first
  two. It replaces the word "backend" of the docs.
- **Provider account:** the customer's account at a provider. For Boat, it
  is a Boat account and one wallet. For instabox, it is the one instabox
  workspace that an instabox API key reaches.
- **Sandbox:** one VM of a provider. Product copy uses this word for an
  instabox machine too, because "machine" means the customer's own hardware.
- **Provider adapter:** the code that maps the shared sandbox capabilities
  to one provider's API. It lives in `futex-ai/sandbox`, as `sandbox-boat`
  does.
- **Provider seams:** the ciproxy-only parts of a provider: the account
  facts, the guest launcher, the launch inspection, and the image source.
  They live in `crates/ciproxy-overflow/src/<provider>/`.
- **Account image:** the snapshot of the job image that ciproxy builds in
  the instabox workspace of one customer. It takes the place of the shared
  snapshot of Boat.
- **Builder sandbox:** a short-lived sandbox that builds an account image.
  ciproxy deletes it after the snapshot save.

## Design

### The provider interface

- `OverflowProvider` is a closed enum with the values `boat` and `instabox`.
  The store keeps it as text with a check constraint. The API sends it as a
  string. My call, 22 November 2026: a closed enum. Reason: the compiler
  then finds each place that a new provider must handle.
- A provider descriptor holds the static facts of a provider: the display
  name, the key hint, the guide link, the lifetime rule, the billing
  account kind, the disk model, the image source kind, and the platforms.
- The shared capabilities come from `sandbox-interface`: the lifecycle, the
  account capacity, the files, and the processes. `ciproxy-overflow` does
  not change for a new adapter.
- Each provider supplies its seams through `ProviderFactory`:
  - `ProviderAccountApi` checks a key and lists its billing accounts. It
    reads the account facts: the name, the lifetime, the billing state, and
    the start fee bound. It reads the runtime cost of one sandbox.
  - `GuestLauncher` writes the private registration file and starts the
    guest once.
  - `GuestInspection` reads the launch marker without a change.
  - The image source says how a sandbox gets the job software: a shared
    snapshot for Boat, and an account image for instabox.
- The control-plane composition builds a provider registry. One `Overflow`
  service serves every workspace. It picks the factory from the provider of
  the workspace's active key.
- A provider without its deployment settings is unavailable. Setup and
  Settings do not offer it. Boat works the same way today without
  `BOAT_SHARED_SNAPSHOT`.
- The account image lifecycle is shared code in `ciproxy-overflow`. It keeps
  one state for each provider account and image release: building, ready,
  failed, or retired. A provider supplies only its build steps. The starts
  of an account wait until its first account image is ready.
- The disk model of a size offer is `whole root` or `data disk`. With
  `whole root`, the offer's disk must hold the image, the free job disk, and
  the ext4 allowance, as for Boat today. With `data disk`, the template
  holds the image, and the offer's disk must hold the free job disk only.
  instabox uses `data disk`.
- The rules of [Overflow](../docs/protocol/overflow.md) stay in the shared
  service for every provider: the size choice, the refusals, the spend, the
  end report, the orphan cleanup, the key change, and the App removal.

### What a new provider adds

The `ciproxy-overflow` README gets this list as "Add a provider". A new
provider adds only these parts:

1. An adapter crate in `futex-ai/sandbox` that passes its conformance
   harness, and the new pinned revision in ciproxy.
2. A value of `OverflowProvider`, its descriptor, and its seams under
   `crates/ciproxy-overflow/src/<provider>/`.
3. Its image source: a shared snapshot, or the build steps of an account
   image. Its launch script goes under `image/` when it needs one.
4. Its connect form, copy, and mockup states.
5. A provider section in [Overflow](../docs/protocol/overflow.md) and in
   [Agent images](../docs/protocol/agent-images.md).
6. A connect guide on the site, and its name in the legal pages.
7. A live probe example and its live checks.

### instabox behind the provider contract

The [Backend contract](../docs/protocol/overflow.md#backend-contract) lists
what a provider must give. instabox gives each function this way. The
[instabox API](https://instabox.sh/docs/api.md) and its
[OpenAPI document](https://api.instabox.sh/v1/openapi.json) are the
sources, read on 22 November 2026. Research notes:
`.context/overflow-providers-and-instabox/research.md`.

| Contract function | instabox |
| --- | --- |
| Start a VM of at least a size, with one runner registration | `POST /v1/machines` from the account image of the job's size, then `PUT /v1/machines/{id}/files` and `POST /v1/machines/{id}/exec` |
| Bill by the second or finer | Runtime is billed by the second while the sandbox runs |
| Report the state of each VM | `GET /v1/machines/{id}` and `GET /v1/operations/{id}` |
| Stop and delete a VM | `DELETE /v1/machines/{id}` with `X-Confirm-Delete` and `Idempotency-Key` |
| End a VM by itself at a lifetime | `deleteAfterSeconds`, 60 to 86,400 seconds, deletes the sandbox in any state |
| List the VMs by a mark | `metadata` at create, and `GET /v1/machines?metadata=key:value` |
| Give the cost of each VM | `GET /v1/usage/machines` with `computeUSD`, `diskUSD`, and `egressUSD`, deleted sandboxes included |
| Accept and check a customer key | An `ibk_` key reaches one workspace, and `GET /v1/orgs` reads it |
| Refuse a start with a typed reason | `error.code`, `retryable`, and `retryAfterSeconds` |
| Return the sizes the account can start | `allowedSpecs` of `GET /v1/orgs` |

Other instabox facts that the design uses:

- Sizes have 2 GiB of memory for each vCPU: 2/4, 4/8, 8/16, 16/32, and
  32/64. Pro allows up to 8/16, Builder up to 16/32, and Team up to 32/64.
- Prices: $0.004 per vCPU-hour, $0.00225 per GiB-hour of memory, $0.00011
  per GiB-hour of data disk beyond the plan's included disk, and $0.02 per
  GiB of egress beyond the plan's free egress. There is no start fee.
- Pro allows 30 starts a minute and 100 running sandboxes. More starts get
  `429 rate_limited` with a retry time.
- The Free plan cannot create a `ci` sandbox from the API
  (`403 template_job_only`), and its idle pause is fixed at 30 minutes.
- An idempotency record stays for 7 days. Boat keeps one for 24 hours.
- A snapshot belongs to one instabox workspace and fixes the vCPU and
  memory of the sandbox that saved it.
- A sandbox sends at 50 Mbit/s after a burst of 512 MiB to 1 GiB. A `ci`
  sandbox that runs a GitHub Actions job for a paid workspace gets
  1000 Mbit/s. Receiving has no cap.
- An exec runs as the login user of the sandbox, `user` by default, with
  passwordless `sudo`. Older templates use `dev`. A command runs for 10
  minutes at most.
- `429 quota_exceeded` and `429 concurrency_limit` cover both account
  limits and exec concurrency. Their `retryable` field tells them apart.
- An abuse check, such as `abuse_cpu` or `abuse_network_flows`, can pause
  a sandbox and lock it for 24 hours.

### User decisions

- **Item 6, decided on 22 November 2026:** the customer brings its own
  instabox key, and instabox bills the customer. The user's words: "ok, let
  the customer bring their own key". Rejected: ciproxy's own instabox
  account for every workspace, which the brainstorm recommended, and both
  models together. So the decisions in force on the customer's own account
  and against reselling overflow minutes stay. The brainstorm notes are in
  `.context/overflow-providers-and-instabox/brainstorm-ciproxy-account.md`.
- **Item 1, decided on 22 November 2026:** while instabox offers no
  snapshot sharing, the first step after a customer saves an instabox key
  builds the account image in the customer's instabox workspace. The
  user's words: "if there is no template sharing we can just have the first
  step after the key is added to create the template on their account".
  Rejected: options A, F, and G, which run jobs without the pinned image.
  Options B and H stay the goal, but instabox does not offer them today.
  instabox calls the result a snapshot, because only instabox staff can
  create a template.
- My call, 22 November 2026, inside the item 1 decision: the build imports
  the published job image, as option D says. Option E, the full recipe in
  each workspace, is the fallback when Milestone 3 shows that the import
  fails. Reason: D keeps full parity, takes minutes instead of hours, and
  gives every customer the same files. Option C would change the
  requirement on pinned tool versions.

### Design calls

My calls, 22 November 2026:

- Generic code, data, and copy use provider-neutral names. Boat names stay
  only for Boat-only parts: wallets, the free-trial lifetime, start fees,
  and the Boat image build. Reason: the user asked for one interface, and
  the `plans` skill moves every consumer of a replaced way.
- "Overflow provider" replaces "backend" in docs and copy. Product copy
  calls an instabox workspace an "instabox account". Reason: `AGENTS.md`
  reserves "workspace" for the ciproxy workspace and "machine" for own
  hardware.
- The instabox adapter goes in `futex-ai/sandbox` as `sandbox-instabox`.
  The provider seams go in `crates/ciproxy-overflow/src/instabox/`. Reason:
  item 49 and Milestone 12 of `plans/ciproxy-mvp.md` split Boat this way.
- A pinned price table gives the hourly price of each instabox size.
  Confirmed costs come from `GET /v1/usage/machines` at list prices.
  Reason: the instabox API gives no prices, and Boat uses the same
  published-price model.
- Each instabox sandbox gets `deleteAfterSeconds` of 21,600, so it ends
  after 6 hours. Reason: instabox then deletes it with no key and no
  control plane, as the backend contract requires.
- Each instabox sandbox gets `autoSuspendAfterSeconds` of 0. Reason: an
  idle pause can stop a runner during a quiet job step.
- Each instabox create sends `secrets` as an empty list, no `envSecrets`
  and no `directSecrets`, and `vault` as false. Reason:
  [Key custody](../docs/protocol/security.md#key-custody) lets a sandbox
  hold only the token that reports the end of its runner. The Free-plan
  spike found that the `instabox-mcp` program in a sandbox can create
  sandboxes and run commands in other sandboxes when a key is present.
- Each instabox create sends `failFast`. Reason: a typed refusal then
  proves that no sandbox started, as `SandboxLifecycle` requires.
- The marks go in instabox `metadata`, and the orphan cleanup lists by a
  metadata filter. Reason:
  [Orphan cleanup](../docs/protocol/overflow.md#orphan-cleanup) needs
  marks, and instabox filters by metadata.
- The size rule stays: the cheapest size that covers the label. So the
  `ciproxy` label of 2 vCPU and 8 GB runs on an instabox size of 4 vCPU
  and 8 GiB. Reason: the rule is in force, and instabox has no 2/8 size.
- The job disk of an instabox sandbox is `/data`. The launcher grows `/data`
  to the label's free job disk after the create, and the runner work folder
  and the Docker data go there. Reason: instabox fixes the root disk to the
  template and grows `/data` online. Milestone 3 confirms it.
- The KMS key `boat-account-keys` stays, and a neutral setting names it for
  every provider. Reason: Terraform protects the key from destroy, and GCP
  cannot rename a key.
- The savings report has one overflow place with a row for each provider.
  Reason: a provider switch in a month gives costs from two providers.
- Every provider uses the existing exit callback route and token. Reason:
  the route is already scoped to one sandbox and names no provider.
- ciproxy does not use instabox webhooks. Reason: the reads and the orphan
  cleanup already reconcile each sandbox, and a webhook adds a public
  endpoint for each account.
- An account image build starts when a key is saved and when an image
  release is new. The old account image serves starts until the new one
  passes its checks. Reason: a failed rebuild must not stop overflow.
- ciproxy deletes an old account image after its last sandbox ends. It
  builds again when the snapshot of a ready image is gone, for example
  after the customer deletes it. Reason: the customer can see and delete
  snapshots in its instabox account.
- The spend limit counts each builder sandbox. Reason:
  [Spend limit](../docs/protocol/overflow.md#spend-limit) counts each
  sandbox that ciproxy starts for the workspace.
- The build saves one account image for each allowed size. It forks the
  builder at each size and saves each fork. Each job then creates from the
  image of its size, with one start. When Milestone 3 shows that a fork
  cannot set the size, each start uses a stopped create, a resize, and a
  start instead. Reason: a create from a snapshot fixes the vCPU and
  memory, and a resize for each job doubles the starts against the plan's
  limit of starts a minute.
- No placeholder file sizes a disk. A create from a snapshot gets at least
  the logical `/data` size of the snapshot, and the launcher grows `/data`
  online. Reason: instabox sizes a disk by its allocation, not by the bytes
  in it. A file cannot change the vCPU or memory.
- An instabox key needs the snapshot scopes `images:read`, `images:write`,
  and `images:delete`, beside the machine scopes. Reason: the build saves
  and deletes snapshots.

### Risks

- A rebuild runs on instabox's current `ci` template. A template change
  can break a rebuild. The old account image then keeps serving starts.
- Each image release rebuilds the account image in every connected
  instabox account. Each build runs in the customer's account and within
  its start limits.
- The upload cap can slow jobs that push images or upload artifacts. Item 5
  decides how ciproxy handles it.
- Job code runs as root in the sandbox, as on a GitHub-hosted runner.
  Milestone 3 checks that job code cannot reach an instabox credential.
- The instabox terms must allow this use. The Boat spike checked the same
  question for Boat.
- An instabox abuse check can pause a busy build and lock its sandbox for
  24 hours. The job then fails. Milestone 3 runs heavy jobs to check.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The overflow screens show a provider choice. Boat keeps its wallet and
start-fee states. instabox gets its connect flow and its states. Copy that
names Boat for a rule of every provider names the connected provider.

- [ ] Change the overflow step of Setup to a provider choice: Boat,
      instabox, or Skip. Turn `app/onboarding/connect-boat` into a
      provider-neutral entry with a Boat branch and an instabox branch.
- [ ] Add the instabox connect states: the key field, a checked key with
      the account name and plan, a rejected key, and the spend limit. Add
      the refused keys that item 3 chooses. Waits for item 3.
- [ ] Update Settings under Overflow: the connected provider and its
      account name, Change key, Disconnect, and the spend states. Add the
      provider switch that item 2 chooses. Waits for item 2.
- [ ] Turn `app/settings/no-boat` into a state with no connected provider.
- [ ] Change the label switch and "Run on the largest Boat size" to copy
      that names the connected provider. Show instabox sizes in the label
      dialog and the live queue.
- [ ] Update the Jobs notices and the job page for instabox: the size that
      ran the job, the spend limit, a rejected key, billing, refused
      starts, and a sandbox that an abuse check paused.
- [ ] Add the upload-cap line of item 5 to the job page of an instabox
      job. Waits for item 5.
- [ ] Add the account image states after an instabox key is saved:
      building with its progress, ready, failed with a retry, and
      rebuilding after a release. Show them in Setup, in Settings under
      Overflow, and on the Machines page.
- [ ] Add a Jobs notice for jobs that wait for the first account image.
- [ ] Update the overflow panel of the Machines page and the savings rows,
      with a row for each provider.
- [ ] Update the user flows `peak-overflow-to-boat`,
      `admin-changes-boat-key`, `admin-skips-boat`, and `first-green-job`.
      Add `admin-connects-instabox`.
- [ ] Update the shared components and fixtures: `boat-fields.tsx`,
      `boat-spend.tsx`, `boat-wallets.tsx`, `fixtures/sizes.ts`, and
      `fixtures/boat-spend.ts`. instabox has no wallet and no start fee.
- [ ] Run the mockup check named in `AGENTS.md`. Smoke-test the changed
      pages.
- [ ] Update the design and the TODOs of this plan for each problem that
      the mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

The protocol and architecture docs state the provider rules and the
decisions of this plan. A Boat-only rule stays under Boat.

- [ ] In `docs/protocol/overflow.md`, use the provider terms. Replace
      "Decision: Boat is the only backend" with the decision of this plan
      and its date. Keep the contract as the provider contract.
- [ ] In `docs/protocol/overflow.md`, add "The instabox account" beside
      "The Boat account". State the key, the paid plan, the checks of
      item 3, and the account name. State one or several providers, as
      item 2 says.
- [ ] In the same section, state the account image: the build after the key
      is saved, the first wait, the rebuild after each release, and the
      builder sandbox in the spend.
- [ ] In `docs/protocol/overflow.md`, make "Overflow runners", "Orphan
      cleanup", "After an App removal", "Spend limit", and "Refused
      sandboxes" provider-neutral. Keep the trial lifetime and the start
      fees under Boat. State the instabox egress rule of item 4.
- [ ] In `docs/protocol/labels.md`, make "Own machines, Boat, or both" and
      "A size that nothing can run" provider-neutral. State the disk rule
      of each disk model.
- [ ] In `docs/protocol/job-routing.md`, make "When Boat cannot take a
      job", "What the dashboard shows", and "Boat fails" provider-neutral.
- [ ] In `docs/protocol/agent-images.md`, add "The instabox image": the
      account image, its import over the `ci` system, its rebuild after
      each release, and each difference from the machine image.
- [ ] In `docs/protocol/security.md`, state key custody for every provider,
      the instabox key scopes, and no instabox Secrets in a sandbox.
- [ ] Make the Boat text of `metering.md`, `billing.md`, `app-removal.md`,
      `cache.md`, `docker-disks.md`, `agent.md`, and `workspaces.md`
      provider-neutral where the rule holds for every provider.
- [ ] Update `docs/architecture.md` and `docs/architecture/data-flow.md`:
      Boat and instabox as external services, the provider registry, and
      "A job on an overflow provider".
- [ ] Update the overflow lines of `README.md`.

## Milestone 3: instabox spike

Depends on: none

A spike on a paid instabox workspace confirms the mechanisms before the
adapter work. It needs a paid instabox workspace and a workspace API key
from the user. It saves its evidence under
`.context/overflow-providers-and-instabox/spike/`.

Free-plan run, 22 November 2026:
`.context/overflow-providers-and-instabox/spike/RESULTS.md`. The other checks
need a paid instabox plan.

- [ ] Check that the instabox terms allow ciproxy to run GitHub runners in
      the instabox workspace of a customer. If they do not, stop and give
      the user a decision.
- [ ] Create a sandbox from the `ci` template with `failFast`, an
      `Idempotency-Key`, `metadata` marks, `deleteAfterSeconds`,
      `autoSuspendAfterSeconds` of 0, no Secrets, and `vault` false.
      Measure the time to `running`.
- [ ] Build an account image with the `snapshot import` steps on the `ci`
      base, from a test OCI build of the job image. Measure the pull, the
      layering, and the snapshot save.
- [ ] From that account image, start one just-in-time runner with
      `ciproxy-guest`. Run a Docker job of the smoke workflow in the
      private `Firna-AI/ciproxy-spike-cache` repository with its
      `runner_label` input. Measure the time from the queue to the first
      step.
- [ ] When the import fails, run the pinned recipe on the `ci` base instead,
      and measure its time.
- [ ] Prove the lifetime. A sandbox with a 10-minute `deleteAfterSeconds`
      ends with no API call, also when it is stopped or paused.
- [ ] Record the bodies of `429 rate_limited`, `429 quota_exceeded`,
      `503 no_capacity`, each `402` code, `403 spec_not_allowed`,
      `403 template_job_only`, and the `401` key codes.
- [ ] Measure the upload speed of a sandbox that runs a GitHub job. Record
      whether the 50 Mbit/s cap or the 1000 Mbit/s rate applies.
- [ ] As root in a sandbox, list what job code can reach: the files and
      tokens of the instabox guest agent, local endpoints, and the outbound
      proxy. Stop and give the user a High security decision when job code
      can act on the instabox workspace.
- [ ] Check the disk: the free space on `/` of the `ci` template, the online
      growth of `/data`, and the runner work folder and Docker data on
      `/data`.
- [ ] Check when the usage of a deleted sandbox appears in
      `GET /v1/usage/machines` with its metadata filter.
- [ ] Check which harmless read refuses a key with only machine scopes and
      accepts a key with `workspace:admin`, for item 3.
- [ ] Run a CPU-heavy build and a network-heavy job for 30 minutes each.
      Record whether an instabox abuse check pauses the sandbox.
- [ ] Fork the builder at two sizes, save a snapshot of each fork, and
      create a sandbox from each snapshot. Also try a stopped create and a
      resize, for the fallback.
- [ ] Check that a key with the snapshot scopes can save and delete a
      snapshot, and that a key without them gets `403 missing_scope`.
- [ ] Ask instabox whether ciproxy can publish an OAuth app, for item 3
      option F.
- [ ] Update the design and the TODOs of this plan with the results. Add a
      decision when a result changes a requirement.

## Milestone 4: Store and schema: provider-neutral overflow records

Depends on: 2

The store keeps the same data under provider-neutral names, with a
`provider` column. Boat stays the only provider. Its behaviour does not
change.

- [ ] Add a migration that renames `boat_accounts` to
      `overflow_provider_accounts` and adds `provider`. Existing rows get
      `boat`. A check allows `boat` and `instabox`.
- [ ] In the same migration, rename `boat_runner_intents` and the view
      `boat_registration_ownership` to overflow names.
- [ ] In the same migration, rename `settings.boat_spend_limit_usd_micros`,
      `labels.allow_boat`, `runners.boat_key_id`, `runners.boat_size_name`,
      and the index `runners_boat_key`.
- [ ] Change the stored value `largest_boat` of `labels.when_nothing_fits`
      to `largest_overflow` with a data migration.
- [ ] Rename the metering tables `metering_boat_costs`,
      `metering_daily_boat_costs`, and `metering_boat_cursors`. Add
      `provider` to the cost tables. Rename `boat_starts` and
      `boat_minutes` of `metering_daily_sizes`.
- [ ] Add the provider to the JSON records of `overflow_accounts`,
      `overflow_starts`, and the runner intents. Read a record without it
      as Boat.
- [ ] Rename the `boat`, `boat_delivery`, and `boat_runtime` modules of
      `ciproxy-store-interface` and their types. Add `OverflowProvider` to
      the account rows.
- [ ] Update `ciproxy-store-pg`, its tests, and each consumer that the
      rename touches, so the workspace builds.
- [ ] Update the READMEs of `ciproxy-schema`, `ciproxy-store-interface`,
      and `ciproxy-store-pg`.

## Milestone 5: Overflow service and scaler: the provider interface

Depends on: 4

`ciproxy-overflow` gets the provider interface of the design, with Boat as
its only provider. All Boat tests pass with no change of behaviour.

- [ ] Add `OverflowProvider` and the provider descriptor to the lowest
      crate that the store, the overflow service, metering, and the API
      share.
- [ ] Replace `BoatAccountApi` with `ProviderAccountApi`. Replace
      `KeyWallets` and `AccountWallet` with `KeyAccounts` and
      `BillingAccount`. Add the lifetime to `AccountFacts`.
- [ ] Replace `BackendFactory` with `ProviderFactory` and the provider
      registry. Replace `OverflowConfig.shared_snapshot` with an image
      source for each provider.
- [ ] Add the disk model to the size offers, and use it in the fit rule.
- [ ] Change `Overflow::wallets` to a key check that names the provider.
      Add the provider to `Overflow::connect`.
- [ ] Rename the Boat types of `ciproxy-scaler` to provider-neutral types.
      Rename `select_boat_size`.
- [ ] Move the Boat code behind the new seams in `src/boat/`. Keep its
      tests, the Boat live examples, and `LIVE.md` working.
- [ ] Add "Add a provider" to the `ciproxy-overflow` README with the list
      of the design. Update the READMEs of the overflow crate and the
      scaler.

## Milestone 6: Control plane, guest, and metering

Depends on: 5

The control plane composes the provider registry with Boat only. The guest
and metering use provider-neutral names.

- [ ] Rename the `boat_*` modules of `ciproxy-control-plane` to `overflow_*`
      names. Compose the provider registry at the composition root.
- [ ] Replace `BOAT_KMS_KEY` with a neutral setting that names the same KMS
      key. Keep `BOAT_SHARED_SNAPSHOT` as the Boat image setting. Update
      Helm, the Terraform outputs, the deploy scripts, and their tests.
- [ ] Rename the Boat mode of `ciproxy-guest` to a sandbox mode. Keep the
      private launch file and the exit callback. Update `image/boat-run.sh`,
      the Boat units, and the image checks that start the guest.
- [ ] Change the Boat place of metering to one overflow place with the
      provider. Give the daily costs and the savings report a row for each
      provider.
- [ ] Update `crates/ciproxy-control-plane/overflow.md` and the READMEs of
      the control plane, the guest, metering, and deploy.

## Milestone 7: Dashboard API: provider-neutral wire

Depends on: 6

The dashboard API sends the provider in every overflow body. The paths stay.
The app's API layer moves in this milestone, so that the gate stays green.
Its screens and copy move in Milestone 11.

- [ ] Add `provider` and a provider-neutral account view to
      `OverflowSettings`.
- [ ] Replace `POST /workspaces/{id}/overflow/wallets` with a key check that
      names the provider and returns its billing accounts.
- [ ] Replace `BoatConnection` with a connection that names the provider.
      Keep the Boat wallet in the Boat case only.
- [ ] Rename the Boat wire types, fields, and values: `BoatLabelFit`,
      `NothingFits::LargestBoat`, `allow_boat`, `JobLocation::Boat`,
      `HistoryPlace::Boat`, `SetupPlace::Boat`, `LabelPlacement`, and the
      savings fields. Add the provider where the view names a provider.
- [ ] Replace `ProblemCode::BoatKeyRejected` with a key problem that names
      the provider.
- [ ] Regenerate the OpenAPI and `ts/api/generated`. Move
      `ts/app/src/api` to the new types with no visible change.
- [ ] Update the README and the docs files of `ciproxy-api`.

## Milestone 8: instabox adapter in futex-ai/sandbox

Depends on: 3

A pull request in `futex-ai/sandbox`, from its own workspace, adds
`sandbox-instabox`. ciproxy then pins the reviewed revision.

- [ ] In `futex-ai/sandbox`, write the plan of the change, as its
      `AGENTS.md` says.
- [ ] Implement `SandboxLifecycle`: the create with the fields of the design
      calls, recovery by the metadata list, inspection, and deletion with
      `X-Confirm-Delete`. Poll the create operation.
- [ ] Implement `SandboxAccountCapacity` from `allowedSpecs` and the pinned
      price table. Add the data disk model to `sandbox-interface` when it
      needs one.
- [ ] Implement `SandboxFiles` with the file mode and the SHA-256 check, and
      `SandboxProcesses` over exec.
- [ ] Implement `SandboxSnapshots`: save, list, inspect, and delete, for the
      account images.
- [ ] Map the refusals. `429 rate_limited` becomes a temporary refusal with
      its retry time.
- [ ] Map `429 quota_exceeded` and `429 concurrency_limit` by their
      `retryable` field. A create at the machine or running limit becomes an
      account limit. A retryable exec refusal waits and retries.
- [ ] Map `503 no_capacity` to no capacity.
- [ ] Map `spec_not_allowed`, `template_not_allowed`, and
      `template_job_only` to a plan refusal. Map the `402` codes to a
      billing refusal.
- [ ] Map the `401` codes to a rejected key, and the `5xx` codes to a
      retryable outage.
- [ ] Test the adapter against a fake instabox server over TLS and the
      conformance harness.
- [ ] Merge it. Pin the new revision of `sandbox-interface`,
      `sandbox-boat`, and `sandbox-instabox` in ciproxy.

## Milestone 9: instabox job image

Depends on: 3

Each image release publishes the job image as an OCI image. A build script
lays it over the `ci` system in a builder sandbox. A launch script starts the
guest in each job sandbox.

- [ ] Publish the job image of each release as a public OCI image, built
      from the machine build. Record its digest in the release.
- [ ] Add `image/instabox-build.sh`. It pulls the image by digest and lays
      its files over the `ci` system. It keeps instabox's kernel, guest
      agent, and network setup, and runs the image checks.
- [ ] When Milestone 3 shows that the import fails, make the script run the
      pinned recipe instead, as the Boat build does.
- [ ] Add `image/instabox-run.sh` and its units. The script moves the work
      folder, the runner temp folder, and the Docker data to `/data`. It
      starts the guest with the private registration file.
- [ ] Keep a root-owned marker for each delivery, so a launch runs once.
- [ ] Add image checks for both scripts that need no instabox account, as
      the Boat checks do.
- [ ] Add `image/instabox.md` and update `image/README.md`.

## Milestone 9a: Shared account image lifecycle

Depends on: 5

The overflow service builds, rolls out, and cleans up the account image of
each provider account that has no shared snapshot. The code is shared for
every such provider. Mocks stand in for the provider in this milestone.

- [ ] Add a store table for account images: the key, the image release, the
      size, the state, the snapshot, the builder sandbox, the failure, and
      the times.
      Add its migration.
- [ ] Start a build when a key is saved, when an image release is new, and
      when the snapshot of a ready image is gone.
- [ ] Run each build in a builder sandbox with the marks, a lifetime, and no
      Secrets. Save the snapshot, then delete the builder.
- [ ] Keep the old ready image for starts until the new one is ready.
      Delete an old snapshot after its last sandbox ends.
- [ ] Retry a failed build with a backoff. Keep the failure for the
      dashboard view.
- [ ] Hold the starts of an account until its first image is ready. Report
      the wait in the overflow view.
- [ ] Count each builder sandbox in the spend limit.
- [ ] Add service tests over a real `PgStore` for each state change.
- [ ] Update the README of the overflow crate.

## Milestone 10: instabox provider in ciproxy

Depends on: 5, 6, 7, 8, 9, 9a

ciproxy adds the instabox seams and registers instabox as a provider.

- [ ] Add the factory in `crates/ciproxy-overflow/src/instabox/` with one
      verified HTTPS origin.
- [ ] Add the account API. `GET /v1/orgs` gives the name, the plan, the
      always-on choice, the sizes, and the billing state. The usage reads
      give the cost of each sandbox.
- [ ] Add the launcher: a mode-0600 file write, then one `sudo` exec of the
      fixed launch script under the marker. Add the read-only inspection
      of the marker.
- [ ] Add the connect checks that item 3 chooses. Waits for item 3.
- [ ] Add the instabox build steps: a builder sandbox from the `ci`
      template, `image/instabox-build.sh` over exec, and the snapshot save.
- [ ] Save one account image for each allowed size: fork the builder at
      each size, and save each fork. Start each job from the image of its
      size. When Milestone 3 shows that a fork cannot set the size, use a
      stopped create, a resize, and a start instead.
- [ ] Add the spend bound: the hourly price for the whole lifetime, no start
      fee, and the egress rule of item 4. Waits for item 4.
- [ ] End a delivery whose sandbox instabox pauses with a
      `cooldownReason`, and delete the sandbox.
- [ ] Register instabox. Add the settings for the instabox origin and the
      published image of each release.
- [ ] Add service tests over a real `PgStore` with instabox mocks. Cover
      connect, start, refusals, the end report, and orphan cleanup.
- [ ] Also cover the spend, a key change, Disconnect, and an App removal.
- [ ] Add `examples/instabox_live.rs` and its section in `LIVE.md`.
- [ ] Update the READMEs of the overflow crate and the control plane.

## Milestone 11: Dashboard UI: providers and instabox

Tags: ui

Depends on: 1, 7, 10

The app shows the approved mockups for both providers.

- [ ] Move the provider-neutral parts of `ts/app/src/screens/boat` to
      overflow components. Keep the wallet choice for Boat only.
- [ ] Add the provider choice and the instabox connect form to Setup.
- [ ] Update Settings under Overflow for both providers, with the provider
      switch of item 2.
- [ ] Update the labels, the Jobs notices, the job page, the Machines page,
      and the savings report.
- [ ] Add the account image states and the first-wait notice of the
      mockups.
- [ ] Update the app tests, the harness scripts, and the browser checks.
      Add instabox fixtures.
- [ ] Run the app typecheck, lint, tests, and export. Compare the screens
      with the mockups.

## Milestone 12: Site: instabox guide and provider-neutral docs

Depends on: 2

The site gets a connect guide for instabox. Pages that name Boat for a rule
of every provider name both providers.

- [ ] Add `guides/connect-instabox.md`: the paid plan, and a key with the
      machine and snapshot scopes only. Add the first image build and its
      wait, and the builder sandbox and snapshot in the customer's account.
      Add the spend limit, the lifetime, the sizes, and the upload rate of
      item 5.
- [ ] Update `connect-boat.md`, `set-a-spend-limit.md`,
      `how-ciproxy-works.md`, `reference/labels.mdx`, `plans.mdx`, the
      security pages, the quickstart, and the index.
- [ ] Update `tiers.ts` and `plan-compare.ts`.
- [ ] Add instabox to the legal pages as a recipient beside Boat. Waits for
      the user's approval of the legal text, as
      `plans/follow-ups/legal-pages-approval.md` says.
- [ ] Run the site lint, typecheck, and build.

## Milestone 13: Live checks before the merge

Depends on: 10

The live checks use a paid instabox workspace and the Boat test account.
They need no deployment of the branch.

- [ ] Run the probes of `examples/instabox_live.rs`: the key check, the
      sizes, an account image build, a start with a real runner, the end
      report, the cleanup, and the usage read.
- [ ] Run the Boat live probes, to prove that Boat still works.
- [ ] Save the results under `.context/overflow-providers-and-instabox/`.

## Milestone 14: Verification, commit, review, and pull request

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

## Post-merge follow-up (non-blocking)

- [ ] On the staging deployment, connect instabox in the dashboard, fill
      the own machines, and run a job of a ciproxy label on instabox. Check
      the job page, the sandbox deletion, and the spend.
- [ ] Repeat the staging job on Boat.
- [ ] Ask instabox for snapshot sharing across workspaces or a ciproxy
      template. Either one replaces the account images, as the item 1
      decision says.
- [ ] Ask instabox for OAuth app publishing, for item 3 option F, and for
      the CI upload rate, for item 5.

## Open decisions

Four items wait for the user: items 2, 3, 4, and 5. Each item uses the
layout of the `decisions-and-findings` skill. The user decided items 1 and 6
on 22 November 2026, as [User decisions](#user-decisions) says. The git
history of this plan keeps the full text of item 1. The options check of 22
November 2026 added options F, G, and H to item 1, E and F to item 3, E to item
4, and C to item 5. It changed the recommendations of items 1 and 5. Its report:
`.context/overflow-providers-and-instabox/options-check.md`. On 22 November
2026, the user asked for other ways to load a snapshot. The answer added option
I to item 1 and lowered the costs of its options C and D.

### 2. No rule for several providers

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/overflow.md`, "The Boat account", and the
  index `boat_accounts_active` of
  `crates/ciproxy-schema/migrations/202611040905_boat_accounts/up.sql`.
- **Background:** A workspace connects one Boat account today. The store
  allows one active key for each workspace. The spend limit, Settings under
  Overflow, and the job routing assume one provider. Each option keeps one
  monthly spend limit for each workspace, as "Spend limit" says.
- **Problem:** With two providers, no rule says whether a workspace can
  connect both, or how ciproxy chooses between them.
- **Impact if no action:** The connect flow, the Settings mockups, and the
  routing rules wait.
- **Root question:** Where do the overflow jobs of a workspace run when
  ciproxy supports several providers?
- **Options:**
  - **A.** One provider at a time. A switch connects the other provider.
    Running sandboxes of the old provider finish, and ciproxy deletes its
    key after the last one ends, as for a key change. Effort: Small.
  - **B.** Several providers in an order that the customer sets. ciproxy
    tries the next provider after a refusal or when no size fits. One spend
    limit covers all of them. Effort: Large.
  - **C.** Several providers, and ciproxy picks the cheapest fitting size of
    all of them for each job. Effort: Large.
  - **D.** Several providers, and each label names its provider. Effort:
    Large.
- **Left out:** A provider for each organization. Reason: one provider
  account serves every organization of the workspace, as "The Boat
  account" says.
- **Wider fix:** None. B, C, and D later need the unique index on the
  workspace and the provider. D also needs a provider on each label.
- **Previous examples:** None found.
- **Recommendation:** A. It meets the request with the smallest change, and
  the provider column keeps B, C, and D open.
- **Action:** ask the user, because B and C give failover and lower cost,
  so no option is clearly best for the product.

### 3. Valid instabox keys can be unusable

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Design.
- **Category:** Security.
- **Location:** `docs/protocol/overflow.md`, "The Boat account", and
  `docs/protocol/security.md`, "Key custody".
- **Background:** ciproxy checks a Boat key when the customer saves it. It
  does not save a key that Boat refuses. An instabox key reaches one
  instabox workspace. By default, a workspace key gets `workspace:admin`
  rights over members, invites, and Secrets. Only a sign-in can create a
  key. The account image build of item 1 also needs the snapshot scopes.
  instabox also offers agent sign-up and OAuth apps, which give a credential
  with chosen scopes. Today every paid plan allows the idle pause off.
- **Problem:** A key of a Free workspace works, but it cannot create a `ci`
  sandbox. No requirement says what ciproxy does with such a key, or with a
  key that has more rights than ciproxy needs.
- **Impact if no action:** A customer can connect a key that never starts
  a sandbox. ciproxy can hold a key that manages the customer's instabox
  members and Secrets.
- **Root question:** How does ciproxy get an instabox credential that runs
  sandboxes in a paid workspace and does nothing more?
- **Options:**
  - **A.** Accept each key that instabox accepts. Starts then fail with plan
    refusals, and the Jobs page shows them. Effort: Small.
  - **B.** At connect, check the paid plan, the always-on choice, and the
    machine and snapshot scopes. Refuse a key that fails a check, with a
    message that names the check. A later downgrade gives plan refusals,
    and jobs wait. Effort: Medium.
  - **C.** B, and also refuse a key with `workspace:admin` rights. A
    harmless read that only such a key can make finds it. Milestone 3
    confirms the read. Effort: Medium.
  - **D.** Accept the key, and show a "not ready" state with the failed
    checks until the account passes them. Effort: Medium.
  - **E.** Connect with instabox agent sign-up. ciproxy asks for the
    machine scopes and shows the claim link and code. The person can only
    narrow the scopes. The key belongs to the approver's own account, not
    to a shared instabox workspace. Effort: Medium.
  - **F.** Connect as an instabox OAuth app. The person picks the workspace
    and the scopes, and ciproxy keeps the refresh token encrypted. instabox
    must let ciproxy publish an app. The grant ends when the approver leaves
    the workspace. Effort: Large.
- **Left out:** None.
- **Wider fix:** F. A grant with the scopes that ciproxy asks for needs no
  scope probe and no scope name.
- **Previous examples:** The Boat key check in "The Boat account" of
  `docs/protocol/overflow.md`.
- **Related:** Item 1. Its decision needs the snapshot scopes
  `images:read`, `images:write`, and `images:delete`.
- **Recommendation:** C. Only C serves every paid workspace now. Ask
  instabox for app publishing, so that F can replace C later.
- **Action:** ask the user, because B, C, and D add new refusals or a new
  state at the connect boundary, and no requirement covers them.

### 4. The spend bound cannot cover egress

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/overflow.md`, "Spend limit".
- **Background:** The spend limit counts confirmed costs, and a bound for
  each start whose cost is not confirmed. The bound is the hourly price
  for the whole lifetime, plus the start fee. A decision in force says that
  uncounted costs never pass the limit. instabox bills egress beyond the
  plan's free egress at $0.02 per GiB, and reports it for each hour.
  ciproxy counts egress at list price from the first byte, because it
  cannot split the plan's free egress with other use.
- **Problem:** Egress has no price for each hour, so the size alone cannot
  bound it.
- **Impact if no action:** The instabox spend rule waits.
- **Root question:** How does the spend limit count the egress of an
  instabox sandbox before instabox reports it?
- **Options:**
  - **A.** Count egress when instabox reports it, with no bound before.
    The spend can pass the limit by the unreported egress of the running
    sandboxes. Effort: Small.
  - **B.** Add a fixed egress allowance to each bound, for example 10 GiB,
    or $0.20. The hourly egress readings replace the allowance, as measured
    runtime replaces its bound. Egress above the allowance can still pass
    the limit. Effort: Small.
  - **C.** Bound egress by the upload cap for the whole lifetime: about
    126 GiB, or $2.50, for each start. No cost stays uncounted. The bound
    is then about 12 times the compute bound of a 4 vCPU size, so small
    limits stop overflow early. Effort: Small.
  - **D.** Leave egress out of the spend. The instabox bill can then pass
    the spend that ciproxy shows. Effort: Small.
  - **E.** B, and the connect step reads the instabox spend cap. The guide
    asks the customer to set that cap as a hard stop. At the cap, instabox
    stops every sandbox of the instabox workspace, also running jobs and
    other use. Effort: Small.
- **Left out:** A limit inside the sandbox. Reason: job code runs as root,
  so it can remove any limit that the launcher sets.
- **Wider fix:** B, when the allowance is a field of the provider
  descriptor. A later provider, such as AWS EC2, bills egress with no
  upload cap, so C cannot serve it.
- **Previous examples:** The start-fee decision in "Spend limit", where the
  owner said, "let's not over engineer this feature, it's a nice to have,
  if we block a bit early 0.07 early that's really not a problem".
- **Recommendation:** B. It covers the egress of normal jobs, and the hourly
  readings keep an overshoot near one hour of upload for each running
  sandbox.
- **Action:** ask the user, because A, B, D, and E change the decision that
  uncounted costs never pass the limit.

### 5. Uploads from instabox can be slow

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Performance.
- **Location:** None.
- **Background:** instabox caps what a sandbox sends at 50 Mbit/s after a
  burst of 512 MiB to 1 GiB. A `ci` sandbox that runs a GitHub Actions job
  for a paid workspace gets 1000 Mbit/s. Those words can also cover a
  ciproxy sandbox. Milestone 3 measures the rate of a ciproxy sandbox.
- **Problem:** At 50 Mbit/s, each 100 MB of upload takes about 16 seconds.
  Jobs that push images, upload artifacts, or save large caches then take
  longer than on own machines or Boat.
- **Impact if no action:** Customers can see slow jobs on instabox and blame
  ciproxy.
- **Root question:** How do customers avoid slow uploads on instabox, or see
  their cause?
- **Options:**
  - **A.** Launch with the cap. State it in the connect step and the guide.
    Ask instabox for the CI rate for ciproxy sandboxes. Effort: Small.
  - **B.** Hold the instabox launch until instabox gives ciproxy sandboxes
    the CI rate. Effort: Small, after instabox agrees.
  - **C.** A, and the job page of each instabox job names the upload cap.
    Effort: Small.
- **Left out:**
  - A proxy for uploads. Reason: the cap applies to each byte that the
    sandbox sends.
  - A label rule. Reason: the overflow switch of a label already keeps
    upload-heavy jobs on own machines, and the guide of A says so.
- **Wider fix:** None.
- **Previous examples:** None found.
- **Recommendation:** C. The options check changed the recommendation from
  A. C shows the cause where the customer sees the slow job. This item
  applies only when Milestone 3 measures the cap.
- **Action:** ask the user, because B trades the launch date for job speed,
  and only the user can weigh that.
