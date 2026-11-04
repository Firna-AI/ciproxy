# ciproxy MVP

Status: Active. On branch `calummoore/ci-proxy-load-balancer`, no pull
request yet. The user approved this plan on 4 November 2026, in the words
"plan is approved". The pull request that completes the work deletes this
plan, as the `plans` skill says.

This plan builds and launches the MVP of ciproxy: the agent, the control
plane, the dashboard, billing, the site, and the deployment. A job names a
ciproxy label in `runs-on`. The control plane gets it through the GitHub
scale set API, and starts its runner in a Firecracker VM on an own machine,
or in the Boat account of the customer. macOS, AWS EC2, and Blacksmith come
later.

The protocol docs hold the requirements, and each milestone names the
sections that it builds. `plans/ciproxy-mvp-foundation.md` holds the
decisions, the pricing, the answers of 4 November 2026, the design calls for
the roles, and the answers to items 8 to 20 and 28. Items 29 to 45 are new,
at the end. Items 32 and 37 still wait for the user.

Contract owners: the [architecture doc](../docs/architecture.md), the
protocol docs under `docs/protocol/`, the
[mockups](../docs/mockups/README.md), the [README](../README.md),
`AGENTS.md`, and the README of each crate and package.

Design calls, my calls, 4 November 2026:

- **Parallel work:** a milestone depends only on the code that it needs. An
  open item blocks only the TODOs that end with "Waits for item N." Reason:
  an answer then blocks only the work that uses it.
- **Spikes:** all five spikes confirm their mechanisms. The Boat spike ran
  on 4 November 2026 with the paid plan: two jobs ran with Docker, 17.6 to
  20.6 seconds from the queue to the first step. It found that a sandbox
  cannot end itself without the account key. Item 47 settles it.
- **Actions:** `Tags:` milestones go to the `ui implementor`, Milestones 2
  and 24 to the `docs implementor`, and the rest to the `non-ui implementor`.
- **Open parts:** `ciproxy-protocol`, `ciproxy-agent`, `ciproxy-guest`,
  `ciproxy-cache-sidecar`, the runner patch, and `image/` are Apache-2.0, as
  item 20 decided. They depend only on each other, so that their public copy
  builds alone. Other crates are closed.
- **One host:** the control plane serves `app.ciproxy.dev/api/`, `/agent/`,
  and `/hooks/`. Reason: three hosts in all, and no cross-origin calls.
- **Job facts:** the repository of a job comes from the scale set messages,
  and its ref, event, and head repository from the GitHub API, as item 32
  says. Never from the VM. Reason: job code runs in the VM.
- **Docs of the code:** each milestone updates the READMEs and the
  `AGENTS.md` commands that it changes.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

Milestone 1 of the foundation plan applies the answers to items 8 to 20.
This milestone adds the states of the new items, and the legal pages.

- [x] Add a removed App, an ended sign-in, and a personal account.
- [x] Add the install command, a refused label name, too little disk, and
      mixed machines.
- [x] Add a payment that Stripe still confirms, and two banners at once.
- [x] Add the workspace of item 48: the switcher, the Organizations tab with
      "Add an organization", and the organization of each job. Show the
      third App permission of item 32.
- [x] Add the terms and privacy pages. Run the mockup check, and smoke-test
      the changed pages.
- [x] Update this plan for each problem that the mockups show.
- [ ] Add the rename of a workspace, the banner of a Free workspace with
      more than one organization, the job page with the mixed cache source
      and with no entries restored, a `labels` key in `related-docs.ts`,
      the warning when an Admin gives the Admin role, no state without the
      runner right after item 50, and the take-out of item 51: the dialog
      says that the App is removed, and the last organization has no
      take-out.

## Milestone 2: Documentation and protocol

Depends on: 1

Each answer goes into the protocol doc that it changes, and its `Open:`
line goes.

- [x] Write the disk rule of item 34, and each answer up to item 49.
- [x] Write the answers to items 50 and 51 into the docs.
- [x] Write the design calls of items 33 to 36 and 41 to 44 into the docs.
      Item 44 contradicts "Cache tiers" in `cache.md`, and item 44 wins.

## Milestone 3: Protocol crate

Depends on: none

`ciproxy-protocol` holds the types of the agent connection. My call: one
WebSocket over HTTPS with JSON. Reason: machines open no inbound port.

- [x] Add the rules of "Slots" in `job-routing.md`, tested on its examples.
      Branch: `conductor/ciproxy-mvp-milestone-3-protocol-crate`.
- [x] Add the registration, the hello, the report every 10 seconds, and the
      commands for runners, job facts, drain, remove, cache, and images.
      Branch: `conductor/ciproxy-mvp-milestone-3-protocol-crate`.
- [x] Add the vsock messages of `ciproxy-guest`. Use tagged enums, hide
      secrets from `Debug`, and snapshot each message with `insta`.
      Branch: `conductor/ciproxy-mvp-milestone-3-protocol-crate`.
- [x] Add the disk rule of "Slots", after Milestone 2 writes it into
      `job-routing.md`.
      Branch: `conductor/ciproxy-mvp-milestone-3-disk-rule`.

## Milestone 4: Store

Depends on: none

Postgres through Diesel, as the `databases` skill says: `ciproxy-schema`,
`ciproxy-store-interface`, and `ciproxy-store-pg`. My call: one schema file
for each area. Reason: parallel branches then do not conflict.

- [x] Add the core tables: workspaces, organizations, labels, scale sets,
      machines, tokens, runners, jobs, settings, and Boat accounts. A
      workspace owns the rest, as item 48 says. Branch:
      `conductor/ciproxy-mvp-milestone-4-store`.
- [x] Keep tokens as hashes and Boat keys as ciphertext. Add an advisory
      lock for each scale set session. Branch:
      `conductor/ciproxy-mvp-milestone-4-store`.
- [x] Test the queries on the Postgres of `DATABASE_URL`, which CI starts.
      Branch: `conductor/ciproxy-mvp-milestone-4-store`.

## Milestone 5: Deployment: hosting and DNS

Depends on: none

Production runs in the GCP project `ciproxy`, and staging and tests in
`internal-498318`. My calls: `europe-west1`, Terraform under `deploy/`, and
Workload Identity Federation. Reason: EU data, reviews, and no stored keys.

- [ ] Add Cloud SQL, Secret Manager, and Cloud KMS for the Boat keys.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define both projects, private SQL, secret containers, KMS, and IAM.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: apply both foundations with `deploy/scripts/bootstrap.sh`.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [ ] Run the control plane and the `futex-ai/cli-release` server on GKE
      Autopilot with Helm, as item 31 says.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define Autopilot, Helm charts, SQL proxy, and runtime identities.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: deploy Helm. Enable the control plane after its runtime exists.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [ ] Set the release server to `get.ciproxy.dev`, `ciproxy-agent`, the tag
      prefix `ciproxy-agent-v`, and a token for this repository only.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Configure release hosts, binary, prefix, and an existing secret.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: prepare the pinned image and store the repository-only token.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [ ] Serve the site, the dashboard, and signed image URLs from a Cloud
      Storage bucket, through Cloud CDN and one HTTPS load balancer. My call:
      they are static files, so they need no runtime.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define public buckets, private images, CDN, routes, and publishing.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: deploy the edge and register a CDN signing key outside Terraform.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [ ] Create the DNS zone, which the user delegates.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define production and staging DNS zones and HTTPS records.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: apply the zones and delegate the domain and staging subzone.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [ ] Deploy staging on each push, and production on each `deploy-` tag. Add
      uptime alerts.
- [ ] Rename the staging environment to the main preview of item 53, with
      its hosts under `br-main.preview`, before the first apply.
- [ ] Set up the main preview from the checklist of `deploy/README.md`, as
      item 52 says.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Add both deploy workflows, restricted federation, and uptime alerts.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: set GitHub variables, deploy, check HTTPS, and enable alerts.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [x] Keep runtime routing when Autopilot moves pods between zones.
      My call, 4 November 2026: use Preview NEG preparation on GKE 1.36.4,
      with matching pod placement. Reason: each zone then has a backend.
      Evidence: `.context/ciproxy-mvp/zone-finding.md`.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [x] Keep signing key names and values together during secret rotation.
      My call, 4 November 2026: require a pod rollout for secret updates.
      Reason: a running pod then keeps its key and name together.
      Evidence: `.context/ciproxy-mvp/secret-rotation-finding.md`.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [x] Bind each pod revision to a numbered signing key.
      My call, 4 November 2026: derive names from versions and keep the
      provider objects. Reason: replacements then keep matching keys.
      Evidence: `.context/ciproxy-mvp/fix-round-1-report.md`.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [x] Match deploy identities with repository IDs and environment claims.
      My call, 4 November 2026: avoid a subject format match and map each
      condition claim. Reason: valid jobs then keep the same trust limits.
      Evidence: `.context/ciproxy-mvp/fix-round-1-report.md`.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.

## Milestone 6: GitHub App and scale sets

Depends on: 4

`ciproxy-github` acts as the App, with one scale set and one listener for
each label. The scale set spike confirms it.

- [ ] Add the App tokens, the repositories, the default branch, and the
      installation events. Create the default labels of `labels.md`.
- [ ] Handle a removed App, a suspended App, and a personal account, as
      item 29 says.
- [ ] Use the crate `scaleset`, with one session for each label under the
      lock of Milestone 4.
- [ ] Limit the repositories with one runner group, because GitHub assigns
      jobs with no acquire step. Check that a Free organization can do it,
      or propose the requirement change.
- [ ] Store each message before you delete it, by job ID. Count the wait
      from `scaleSetAssignTime`. My call: `queueTime` is empty, and the
      spike found this time within about a second of the queue time.
- [ ] Read the ref, the event, and the head repository of each run with
      the "Actions" read permission, and forward them to the agent, as item
      32 says.
- [ ] Link each installation to the workspace of the Admin who started it,
      by the `requester` of the `installation` event, because GitHub sends
      an approving owner to the setup URL without the state of ciproxy.
      Keep one scale set for each organization and label.
- [ ] Create and remove the just-in-time registrations.

## Milestone 7: Scaler

Depends on: 3

`ciproxy-scaler` turns a state snapshot into actions: start a runner on a
machine or on Boat, stop a runner, or delete a sandbox.

- [x] Apply `job-routing.md` from "Where a job runs" to "Failures", and the
      size changes of "Custom labels" in `labels.md`. Give each start a typed
      reason.
      The snapshot builder supplies the recent-job warm allocation under
      "Warm runners" in `job-routing.md`. Milestone 13 applies that policy
      when it builds the snapshot.
      Branch: `conductor/ciproxy-mvp-milestone-7-scaler`.
- [x] Unit-test each rule with a hand-worked state. The tests enforce the
      supplied allocation. The snapshot builder's history and allocation
      tests belong to Milestone 13.
      Branch: `conductor/ciproxy-mvp-milestone-7-scaler`.
- [x] Use the shared protocol fit rule for slots and job disk. Check disk
      pressure and warm runner disk use.
      Merge notes: `.context/ciproxy-mvp/scaler-merge-notes.md`.
      Branch: `conductor/ciproxy-mvp-milestone-7-scaler`.

## Milestone 8: Machine gateway

Depends on: 3, 4

`ciproxy-gateway` is the control plane side of "Install", "Connection", and
"Machine identity" in `agent.md`.

- [ ] Issue tokens for one hour and any number of machines, which an Admin
      can revoke. Register machines, and accept the agent WebSockets.
- [ ] Store the reports, mark machines offline after 60 seconds, send the
      commands, and revoke a removed machine at once.
- [ ] Accept this protocol version and the one before. My call: agents
      update themselves.
- [ ] Test the gateway with a fake agent over a real WebSocket.

## Milestone 9: Agent

Depends on: 3

`ciproxy-agent` runs as root under systemd. It starts each runner in a fresh
Firecracker VM, as `agent.md` says, which the Firecracker spike confirms. It
ships through `futex-ai/cli-release`.

- [ ] Add `install --token` and `uninstall`. My call: "Remove a machine" needs
      `uninstall`.
- [ ] Report the hardware and the slots every 10 seconds. Reconnect after a
      growing wait, and apply "Drain and remove".
- [ ] Run each VM under the jailer in its own systemd unit, on a reflink
      copy of the image, with the internet and the machine cache only. My
      call: an agent restart then keeps the VMs.
- [ ] Pass registrations over vsock only. Give each VM an empty drive for a
      Docker disk.
- [ ] Fetch and check the newest image. Update the agent with `cli-updater`
      every hour.
- [ ] Release with release-plz and `scripts/cli-release.sh package`, as
      `x86_64-unknown-linux-gnu` for old glibc versions.
- [ ] Test with fakes. Smoke-test a Docker job, a warm runner, and a restart
      during a job.

## Milestone 10: Job VM image

Depends on: 3

One image serves the job VMs and Boat, as "The image" in `agent.md` says.
The Firecracker spike confirms it for VMs. The Boat spike ran the runner
from a minimal snapshot, not from the full image.

- [ ] Measure the full image first. If it fits no size, propose a smaller
      image as a decision.
      Orchestrator: run `bash image/build.sh .context/image-build` on a
      KVM host. Read `artifacts/measurement.json` in that build directory.
      The full image size remains unmeasured here.
      Evidence: `.context/ciproxy-mvp/image/measurement-preflight.json`.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.
- [x] Define it under `image/` from a pinned `actions/runner-images`, with
      Docker, the runner, and a kernel.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.
- [x] Add `ciproxy-guest`: it takes the registration over vsock, starts the
      runner, and powers off the VM after the runner exits. On Boat, it
      tells the control plane when the runner exits, as item 47 says.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.
- [ ] Publish it with its SHA-256, and as a shared Boat snapshot. Rebuild
      before GitHub stops accepting the runner.
      Orchestrator: run `image/publish.sh` and the create, prepare, and
      snapshot commands of `image/boat.py` from the
      [image guide](../image/README.md#boat-shared-snapshot).
      Use `image/check-runner.sh` and `image/github.py runner-support`
      to plan the next rebuild before runner support ends.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.
- [ ] Run a job from the shared snapshot on Boat `small` and `default`, and
      measure the free disk. The label `ciproxy` asks for 60 GB, and Boat
      gives 51 GB for files. Propose the disk of the default labels as a
      decision.
      Orchestrator: run the registration and result commands of
      `image/github.py`, and the create, run, status, and delete commands
      of `image/boat.py`, for each size. The
      [smoke guide](../image/README.md#boat-small-and-default-smoke) gives
      the arguments. Supply the overflow callback and its scoped token.
      Compare `image/boat.py metadata` before and after snapshot restore.
      Bring the measured free bytes to the label disk decision.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.

## Milestone 11: Cache sidecar and runner patch

Depends on: 3

A patched runner sends the cache calls to a sidecar in the VM, which reads
and writes through to GitHub. The machine cache keeps entries after GitHub
evicts them. Boat runs the stock runner. The cache spike confirms it.

- [ ] Rename `ACTIONS_CACHE_URL`, `ACTIONS_CACHE_SERVICE_V2`, and
      `ACTIONS_RESULTS_URL` in `Runner.Worker.dll` at equal length.
- [ ] Add `ciproxy-cache-sidecar`. Pass other results calls through, and
      fall back to GitHub on any error.
- [ ] Scope each entry from the job facts, as "Cache scopes" says.
- [ ] Add "Clear cache" for one machine, eviction, and the reports.
- [ ] Test the sidecar against a fake GitHub cache.

## Milestone 12: Boat backend on the shared sandbox interface

Depends on: 4

Item 49 puts the Boat backend in `futex-ai/sandbox`, behind a split of its
trait into capabilities. A pull request in that repository, from its own
workspace, does the first four TODOs. ciproxy then uses `sandbox-interface`
and `sandbox-boat` at the reviewed revision. `ciproxy-overflow` keeps the
rules of `overflow.md`: the size choice, the refusals, the spend, the end
report, and the orphan cleanup. The Boat spike confirms the runner in a
sandbox, the refusals, and the limits. **Fallback:** AWS EC2 behind the same
interface, which needs a user decision.

- [ ] In `futex-ai/sandbox`, write the plan of the change, as its
      `AGENTS.md` says.
- [ ] In `futex-ai/sandbox`, split `SandboxBackend` into capability traits:
      the lifecycle, snapshots, image preparation, processes, files,
      terminals, ports, and screens. Move `sandbox-e2b`, the mocks, and the
      conformance harness to them. Record the adoption steps for Juno.
- [ ] In `futex-ai/sandbox`, add a VM size request, the sizes that an
      account can start with their prices, a start refusal with a retry
      time, and one-shot lifetimes up to the limit of each provider.
- [ ] In `futex-ai/sandbox`, add `sandbox-boat` with the lifecycle,
      snapshots, processes, and files. Use marks, `noEnv`, `ttlSeconds`,
      and an `Idempotency-Key`. Join `machineTypes` of `GET /limits` with
      the published sizes and prices, because the API gives no dimensions.
      Map HTTP 429 `rate_limited` to a refusal with its `Retry-After`, 403
      `machine_class_plan_required` to a size that the plan lacks, and 402
      `billing_required` to a billing refusal. Test it against a fake Boat
      server and the conformance harness.
- [ ] Add `ciproxy-overflow` on `sandbox-interface`, with the Boat adapter
      of the reviewed revision. Read the sizes of each account every 5
      minutes. My call: a Boat plan can change at any time.
- [ ] Pick the cheapest size that fits, or the largest.
- [ ] Start from the shared snapshot with marks, a private registration
      file, a token for the end report, a lifetime of 6 hours, or 2 on a
      free trial, and a delivery key for each start.
- [ ] Delete a sandbox at once when its guest reports the end of its
      runner. Accept one report for each token.
- [ ] Delete idle marked sandboxes. Wait out a refusal with a retry time,
      drop a size that the plan lacks, and stop Boat starts at a billing
      refusal. Treat `canStart` as a hint, as the spike saw a refusal after
      `canStart: true`. Count the spend as run time and start charges. My
      call: Boat bills these two parts.
- [ ] Encrypt the keys with Cloud KMS. Test with the mock of
      `sandbox-interface`.

## Milestone 13: Own machines end to end

Depends on: 6, 7, 8, 9, 10, 11

This milestone builds "A job on an own machine" of `architecture.md`, on a
test host in `internal-498318`.

- [ ] Wire the control plane and the agent. Add the patch and the sidecar to
      the image.
- [ ] Stop a warm runner only after GitHub removed it. My call: a runner
      that just took a job keeps it.
- [ ] Smoke-test warm starts, slots, drain, remove, offline, a cache hit, a
      restart, and a disallowed repository.

## Milestone 14: Dashboard API: sign-in, setup, machines, and settings

Depends on: 4, 6, 8

`ciproxy-api` serves the dashboard with axum, and `utoipa` with
`openapi-typescript` generates its types. My call: money in cents, times in
UTC RFC 3339, and IDs as strings. Reason: JavaScript loses 64-bit numbers.

- [ ] Name the generator and the wire forms in `AGENTS.md`. Add a gate phase
      that regenerates the types and fails on a diff, as item 45 says.
- [ ] Sign in with the GitHub App. Apply "Roles" in `security.md` for each
      workspace, with the access requests.
- [ ] Add an organization to a workspace, take one out, and keep the
      repository access of each one. A take-out deletes the scale sets and
      removes the install through the GitHub API, as item 51 says. ciproxy
      tells its own removal apart from a removal by an owner.
- [ ] Give the build of the API with each response, for the reload of an
      old dashboard tab in Milestone 19.
- [ ] Give the install command to an Admin.
- [ ] Serve Setup, Machines, and Settings, with "Clear cache". Milestone 15
      adds the Boat parts.
- [ ] Test each route at the HTTP level with a real store.

## Milestone 15: Overflow to Boat end to end

Depends on: 12, 13, 14

This milestone builds "A job on Boat" of `architecture.md`.

- [ ] Wire Boat into the scaler loop with every rule of `overflow.md`.
- [ ] Serve the Boat step of Setup and Settings under Overflow, with the key
      check and the spend limit at each connection.
- [ ] Smoke-test an overflow, an idle delete, and the spend limit.

## Milestone 16: Metering and the savings report

Depends on: 4

`ciproxy-metering` computes `metering.md` from the stored job events.

- [x] Count the starts of each period, the machines, the soft limits, and
      the offer.
      My call, 4 November 2026: keep the current paid tier when offers tie.
      Reason: the same price needs no tier move.
      Branch: `conductor/ciproxy-mvp-milestone-16-metering`.
- [x] Count the starts and the machines of each workspace across its
      organizations.
      Branch: `conductor/ciproxy-mvp-milestone-16-metering`.
- [x] Compute the savings report, and keep the daily totals for 90 days, as
      item 37 says.
      My call, 4 November 2026: accept dated price lists and the monthly
      divisor as explicit inputs, because the docs provide no defaults.
      Missing or ambiguous costs stay explicit and suppress savings.
      Branch: `conductor/ciproxy-mvp-milestone-16-metering`.
- [x] Unit-test each rule with hand-worked numbers.
      Branch: `conductor/ciproxy-mvp-milestone-16-metering`.

## Milestone 17: Billing with Stripe

Depends on: 4

`ciproxy-billing` keeps the tier and the add-ons in step with Stripe, as
`billing.md` says, after `futex-ai/accounting`. My call: the return page and
the event both read the session, so the tier changes once.

- [ ] Create the prices, and open Checkout, with no tax, as item 39 says.
- [ ] Take each signed event once. Change tiers and add-ons with proration,
      and open the portal.
- [ ] End the subscription of a workspace at the end of its period when
      its last organization leaves.
- [ ] Test each state with recorded events and the Stripe test mode.

## Milestone 18: Dashboard API: jobs, usage, savings, and billing

Depends on: 14, 16, 17

These routes serve the other pages of the dashboard.

- [ ] Serve Jobs, Savings, Plan and usage, Plans, Checkout, and one banner
      for each page.
- [ ] Test each route at the HTTP level.

## Milestone 19: Dashboard UI: shell, sign-in, and setup

Tags: ui

Depends on: 1, 14

The Expo app under `ts/app` builds each screen and state of its mockup, with
`@firna/ui`, `@ciproxy/tokens`, and the generated types. Each UI milestone
smoke-tests its pages at 1440, 720, and 390 pixels.

- [ ] Build the shell, sign-in, the App install, Setup without its Boat
      step, the page for a member with no role, and the page of a removed
      App.
- [ ] Reload a dashboard tab that is older than the API. My call: the API
      gives its build with each response, and the dashboard reloads at the
      next page change after the build changes. Reason: an old tab must not
      call a newer API with old types, and a page change loses no input.

## Milestone 20: Dashboard UI: Machines and Settings

Tags: ui

Depends on: 14, 19

Each screen follows its mockup, as Milestone 19 says.

- [ ] Build Machines, and Settings with the members and roles but without
      Overflow.

## Milestone 21: Dashboard UI: Jobs, Savings, and billing

Tags: ui

Depends on: 18, 19

Each screen follows its mockup, as Milestone 19 says.

- [ ] Build Jobs, Savings, Plan and usage, Plans, Checkout, and the banners.
- [ ] Run one checkout in the Stripe test mode.

## Milestone 22: Dashboard UI: Boat

Tags: ui

Depends on: 15, 19

Each screen follows its mockup, as Milestone 19 says.

- [ ] Build the Boat step of Setup, and Settings under Overflow, with their
      states.

## Milestone 23: Site: marketing pages and the docs look

Tags: ui

Depends on: 1

`ts/site` serves `ciproxy.dev` from the marketing and docs mockups.

- [x] Build the landing page, the pricing page, the footer, and the legal
      page frames.
      Branch: `conductor/ciproxy-mvp-milestone-23-site`.
- [x] Theme the docs. Restyle the Starlight search to its mockup as far as
      CSS allows, and record each gap as a follow-up item.
      Branch: `conductor/ciproxy-mvp-milestone-23-site`.
- [x] Smoke-test each page at 1440, 720, and 390 pixels.
      Branch: `conductor/ciproxy-mvp-milestone-23-site`.

## Milestone 24: Site: the 14 launch docs pages

Depends on: 13, 15, 23

Each page of the docs sidebar mockup states only what the protocol docs
require and the product does.

- [ ] Start here: Overview, Quickstart, and How ciproxy works.
- [ ] Guides: Add a machine, Connect Boat, Move your workflows, Set a spend
      limit, and Remove a machine.
- [ ] Reference: Labels and VM sizes, Agent install options, GitHub App
      permissions, and Plans and starts.
- [ ] Security: Job isolation, and The agent connection. Check each page
      against the product on the test host.

## Milestone 25: Tier features: backend

Depends on: 13, 18

This milestone builds the Docker disks of Team and Business, which ship at
launch, as item 8 says. Several organizations come with the workspace of
item 48 in Milestones 6, 14, and 20. The job hooks spike confirms the Docker
disks.

- [ ] Attach the Docker disk in the job-started hook, as "Docker disks" in
      `cache.md` says.
- [ ] Give the disk to the first default-branch job, and a reflink copy to
      each other job.
- [ ] Use drive IDs of letters, digits, and underscores. Keep each Docker
      image store on the disk.
- [ ] Discard a disk that fails to mount. My call: a rebuild loses only
      time.

## Milestone 26: Dashboard UI: tier features

Tags: ui

Depends on: 20, 21, 25

Each screen follows its mockup, as Milestone 19 says.

- [ ] Build the Docker disks of the machine page.

## Milestone 27: Launch

Depends on: every milestone from 1 to 26

The public copy goes live before the site, because the site links to it.

- [ ] Run the CI of ciproxy on ciproxy, and fix or record each problem.
      Close the follow-up item `expo-56-npm-advisories.md`.
- [ ] Draft the terms and the privacy policy, which the user writes or
      approves. They state the limits of "Data retention" in `security.md`.
- [ ] Register the production App and the Stripe live mode, which need the
      accounts of the user.
- [ ] Deploy production. Smoke-test a first job, an overflow, and a live
      checkout that the user approves.
- [ ] Publish the open parts in a public repository that the user creates,
      and check that it builds alone. Check the licence of the installer
      of `futex-ai/cli-release`.
- [ ] Ask Boat to confirm in writing that a customer may run ciproxy
      runners on its own Boat account. The user sends the request. My call:
      the terms allow this use by inference only.
- [ ] Check the three prices of the cost claim and the GitHub charge for
      self-hosted runners again. Update the price list of the site and its
      date. Reason: GitHub only postponed a charge of $0.002 a minute, which
      would lower the claim to about 54%.
- [ ] Publish the site. Then make the GitHub App public.

## Milestone 28: Verification, commit, review, and pull request

Depends on: every other milestone

The gate, the review, and the pull request stay with the orchestrator.

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

- [ ] Check the prices of the cost claim each month after the launch, and
      update the price list of the site and its date. My call of 4 November
      2026: the claim names its date, so a monthly check keeps it true.

## Open decisions

Design calls on the draft's questions, my calls, 4 November 2026. Each one
had a clearly best option:

- Item 30: The `cli-release` installer gains a generic step that runs the binary
  with the arguments of the install script, as a change in
  `futex-ai/cli-release`. Reason: the install command of the mockups stays one
  line.
- Item 33: Label names follow the GitHub label rules, and labels have no rename.
  Reason: a new label and a delete give a safe rename.
- Item 34: Slots also count disk, and the machine cache gets the rest of the
  disk. Reason: the machine page already says why a machine has no slots.
  The rule, my call of 4 November 2026: a size fits an own machine when the
  machine has enough free slots and enough free disk for it. The host
  reserve also keeps a quarter of the disk, for the system and the images.
  Each running job holds the disk of its size, and the machine cache uses
  the rest and frees space when a job needs it. Reason: one rule for every
  resource, as for the threads and the memory. Milestone 2 writes it into
  "Slots" of `job-routing.md`, and the protocol crate follows.
- Item 35: A sign-in lasts 30 days, with a membership check each hour. Reason:
  access ends soon after a person leaves the organization.
- Item 36: On a personal account, ciproxy shows a page that says it serves
  organizations only, with the link to remove the App. Reason: a GitHub
  organization costs nothing.
- Item 38: Checkout takes cards only, and its return page says when Stripe still
  confirms. Reason: the page never asks for a second payment.
- Item 40, withdrawn on 4 November 2026: the disk of the default labels is a
  requirement in `labels.md`, so a change goes to the user. Milestone
  10 measures the free disk on Boat with the shared image, and then
  proposes the disk as a decision.
- Item 41: A size that fits a different number of jobs on different machines
  shows a range, such as "1 to 3 jobs". Reason: it keeps the rule of one
  machine.
- Item 42: At a failed payment, the payment banner replaces the soft-limit
  banner. Reason: a page keeps one banner.
- Item 43: A Free period that starts on day 29 to 31 follows the Stripe rule:
  the last day of a shorter month. Reason: one rule serves every period.
- Item 44: A job that restored from both caches shows "This machine and the
  GitHub cache". Reason: it is true for every job.

Answers of 4 November 2026:

- Item 29, B: when an organization removes the App, its subscription ends
  at the end of the paid period, as "Move to Free" does. ciproxy revokes
  its machines and deletes its data after 30 days. Rejected: no change, an
  end at once with a refund, a pause of 30 days, and deletion at once.
- Item 31, B: GKE Autopilot runs the control plane and the release server,
  with Helm. Rejected: one Cloud Run service, Cloud Run in two parts, and a
  Compute Engine VM.
- Item 39, A: ciproxy collects no tax at launch. The user said "for now",
  so the follow-up item `taxes-after-launch.md` comes back to it. Rejected:
  Stripe Tax, prices that include tax, Stripe Managed Payments, and own tax
  rates.
- Item 45, B: a gate phase regenerates the API types and fails on a diff.
  Rejected: review only.
- Item 37, B: ciproxy keeps 90 days of job details on every tier, and each
  page shows only the window of its tier. Rejected: the window of the
  current tier, 13 months of totals, and a setting for a shorter window.
  My call: the daily totals also stay 90 days, so the privacy policy states
  one limit.
- Item 32: the user asked for a spike of option B, the first report of the
  job-started hook, before the choice. It passed on 4 November 2026.
- Item 47, G: the guest in each sandbox tells the control plane when its
  runner exits, with a token for that sandbox only, and the control plane
  deletes the sandbox at once. A Boat TTL of 6 hours, or 2 hours on a free
  trial, is the backstop. Rejected: the TTL alone, a request to Boat, AWS
  EC2 first, Boat's in-sandbox key, a renewed TTL, and a key that ciproxy
  creates. The user proposed this option.
- Item 50, in the user's words: "reverse the previous decision and have a
  new machine be addable with a admin/runner role inside of ciproxy for
  any of the installed installations". So the Admin role alone adds a
  machine, connects Boat, and adds an organization, in a workspace with
  any number of organizations, and ciproxy checks no GitHub runner right.
  This reverses item 28 of the foundation plan. When an Admin gives the
  Admin role, the dialog says that it covers the runners of every
  organization, and an organization that wants other people to control its
  runners uses its own workspace. Rejected: options A to F. Later: a
  narrower right for chosen organizations or repositories, kept in the
  follow-up item `scoped-runner-rights.md`.
- Item 51, D: a take-out removes the App from the organization. ciproxy
  deletes its scale sets, lets its running jobs finish, and removes its own
  install through the GitHub API. The organization can join any workspace
  again as a new install, and the last organization cannot be taken out.
  Rejected: options A, B, C, E, and F.
- Item 52, A: staging comes up now. The orchestrator runs the GCP steps of
  `deploy/README.md` from the user's Mac, with the user's approval of each
  command. The user creates the release token and sets the name servers at
  the registrar.
- Item 53, D: the staging environment becomes the main preview, as in
  `mokly-cloud` and `futex-ai/accounting`: `br-main.preview.ciproxy.dev` for
  the site, `br-main.preview.app.ciproxy.dev` for the dashboard and the API,
  and `br-main.preview.get.ciproxy.dev` for the downloads. It deploys from
  the primary branch until that branch merges into `main`, then from
  `main`. Previews of labelled pull requests at `pr-<number>.preview.*`
  wait for the follow-up item `pr-preview-environments.md`. Rejected:
  `staging.ciproxy.dev`, and pull request previews now.
- Item 49, C: the Boat backend lives in `futex-ai/sandbox`. Its trait
  `SandboxBackend` splits into capabilities, and `sandbox-boat` starts with
  the lifecycle, snapshots, processes, and files. The interface gains VM
  sizes, the size list with prices, refusals with a retry time, and longer
  lifetimes. Juno adopts the split. Rejected: ciproxy's own trait until
  after launch, and the full trait for Boat. The user first wrote A, and
  then C, which stands.
- Item 32, A: the App asks for a third permission, "Actions" read, and the
  control plane reads the ref, the event, and the head repository of each
  run from the GitHub API. Rejected for now: the first report of the
  job-started hook, which the spike proved and which the follow-up item
  `job-facts-from-the-hook.md` keeps; trust in no ref; and an organization
  webhook.
- Item 48, in the user's words: "multiple github orgs just count as
  multiple orgs and can be in the same workspace/team on ciproxy", an
  Admin adds an organization at any time, and Team also allows several
  organizations. So a ciproxy workspace holds one or more GitHub
  organizations, close to option G. Rejected: options A to F.

Design calls for the parts that answer B to item 29 leaves open, my calls,
4 November 2026. The options check named these parts:

- The revoke covers every credential: the machine keys and the Boat key go
  at once. Reason: no credential of an organization that left stays usable.
  After item 48, the machines and the Boat key belong to the workspace, so
  they go when the last organization of the workspace leaves. A removal of
  one organization stops its runners, and deletes its data after 30 days.
- A removal takes only that organization out of its workspace. The
  subscription ends when the last organization of the workspace leaves.
  Reason: the other organizations still use it. Item 48 replaced the
  Business pool with the workspace.
- A new install within 30 days finds the labels, the settings, and the job
  history. Each machine needs the install command again. Reason: the data
  stays for 30 days, and the machines are revoked.
- A suspension of the App stops the jobs and changes nothing else. An
  unsuspend starts them again. Reason: the owner can undo a suspension at
  any time.
- After a removal, the dashboard says that the App was removed, and how to
  install it again. Reason: a GitHub App user token probably cannot read an
  organization without the App.
- The scale sets and the runners of the organization stay on GitHub. The
  docs say so. Reason: ciproxy has no GitHub access left to delete them.

Design calls for the workspace of item 48, my calls, 4 November 2026:

- The customer entity is a "workspace". Reason: "team" would clash with the
  Team tier.
- Free allows one organization. Team and Business allow several, and
  "several GitHub organizations" moves from Business to Team. Reason: the
  user named Team.
- An Admin adds an organization with "Add an organization", which installs
  the App on it. GitHub decides who may finish the install. ciproxy links
  the installation to the workspace of the Admin who started it.
- The workspace owns the tier, the members and their roles, the machines,
  the install tokens, the labels, the settings, and the Boat account. Each
  organization keeps its own repository access.
- A machine runs the jobs of every organization of its workspace, each in
  a fresh VM. Tenant isolation holds between workspaces. Reason: one fleet
  serves the customer, and the cache scopes already separate repositories.
- A person joins a workspace when the person is a member of one of its
  organizations, through the access request. An owner of any organization
  of the workspace can claim the Admin role. Reason: the access model stays
  as it was.
- An Admin can take an organization out. Its runners stop, and the App
  stays installed until an owner removes it on GitHub.
- Starts, machines, the soft limits, and the savings report count per
  workspace.

Design calls on the docs work of items 32 and 48, my calls and those of the
docs agent, 4 November 2026:

- The model lives in a new doc, `workspaces.md`, with the App install flow.
- An install that no Admin started makes a new workspace. A new workspace
  takes the name of its first organization, and an Admin can rename it.
- A take-out stops the runners without a job, lets running jobs finish, and
  deletes no data.
- Team and Business allow any number of organizations.
- One warm runner count for each machine, shared by the organizations of
  the workspace by their recent jobs. Reason: the warm runners go where the
  jobs arrive, and the machine page keeps one control.
- On Free, the number of organizations is a soft limit, with the banner and
  the offer, and "Move to Free" stays possible. Reason: it treats
  organizations as the machine count is treated.
- When the last organization of a workspace leaves, the Boat key goes when
  the last running sandbox ends, as after Disconnect. Reason: ciproxy needs
  the key to delete those sandboxes.
- A `pull_request_target` job counts as a pull request for trusted
  write-back. Reason: its job often runs code from the pull request.
- `AGENTS.md` gets the copy rule "workspace, never team".
- Asked by the scaler workspace: the recent jobs that share the warm
  runners are the jobs of `ciproxy` of the last 24 hours, with equal
  weight. Each organization gets its share, rounded down, and each runner
  left over goes to the organization with the most recent jobs and the
  fewest warm runners. With no recent jobs, the runners go round in turn.
  Reason: a short window follows the work of the day, and the rule is
  simple to test.
- Changed after the options check of items 50 and 51: an owner of an
  organization claims the Admin role only when the workspace has no Admin,
  or when the person owns every organization of the workspace. Reason: an
  owner of a later organization must not take over the workspace of
  another. With one organization, the rule stays as it was.

Design calls of the docs work of Milestone 2, my calls and those of the
docs agent, 4 November 2026:

- The label sections move to a new doc, `labels.md`, so that
  `job-routing.md` stays near its length target.
- A warm runner holds the disk of its size, as it holds its slots. "Fits"
  means enough capacity, and "starts" means enough free slots and free job
  disk. The space of the machine cache counts as free job disk, because the
  cache gives it back.
- The new term "job disk" names the three quarters of the disk that the
  host reserve leaves, because "free disk" already names a health check.
- A person who fails the hourly membership check loses the role.
- The Docker disks and their copies share the free job disk with the
  machine cache. When a job needs space, ciproxy deletes the Docker disk
  that no job uses and that a job used least recently. Reason: a Docker
  disk is a cache, so its loss costs only time.
- The hardware minimum adds 80 GB of disk. Reason: such a machine then fits
  one job of `ciproxy`, as the memory minimum ensures one slot.
- A job that restored no entry shows "No entries restored". Reason: it is
  true for every such job.

Design calls on the problems that the mockups of Milestone 1 showed, my
calls and those of the mockups agent, 4 November 2026:

- While Stripe confirms a payment, Plan and usage says so, and the Plans
  page opens no second checkout. Reason: design call 38 says that no page
  asks for a second payment.
- A label name follows the label rules of GitHub, checked with GitHub when
  the label is saved. Reason: design call 33 names the GitHub rules, so
  ciproxy adds no rule of its own.
- Every label except `ciproxy` can be deleted. Its waiting jobs stay queued,
  its running jobs finish, and ciproxy removes its scale sets. Reason: the
  warm runners serve `ciproxy`.
- The fit line of the label dialog counts the connected machines that can
  run jobs, and says "1 job on 3 of your 4 machines" when some fit none.
  Reason: design call 41 covers different counts, and this covers none.
- For 30 days, the Organizations tab shows a removed organization, with the
  last day of a new install that keeps its history. Reason: an Admin
  manages the organizations there.
- A suspension shows a notice on Jobs, with no waiting count for that
  organization. Reason: the other reasons why jobs wait show there.
- The install command reads `curl -fsSL https://get.ciproxy.dev | sudo sh
  -s -- install --token …`. Reason: design call 30 runs the agent with the
  arguments of the install script.
- The page of a removed App shows no data of the workspace, because ciproxy
  cannot check a membership after a removal.

Design calls on the questions of the docs work, my calls, 4 November 2026:

- A move up between paid tiers changes the tier only after its charge
  succeeds. A failed charge keeps the old tier, and the page offers to try
  again or to update the card. When the bank asks for a confirmation, the
  Stripe payment page opens. Reason: the tier already changes only after
  Stripe confirms a checkout, and Stripe supports this for a subscription
  change.
- The savings report prices a job whose VM size an alternative does not
  sell at the smallest size of that alternative with at least its vCPU and
  memory. Reason: the job would need that size there, and each price stays
  a real list price.
- Withdrawn after item 48: the share of a pool's price in each savings
  report. The savings report now covers the whole workspace, which pays
  one price.

No item of this plan waits for the user. The options check of items 50 and
51 ran, and its report is in
`.context/ciproxy-mvp/options-check-50-51.md`.

The options check of the `plans` skill ran over these items on 4 November
2026. Its reports are under `.context/ciproxy-mvp-foundation/`:

- `options-check-3.md`, over items 29 to 45. It changed the
  recommendations of items 29, 32, 37, and 39.
- `options-check-4.md`, over item 47. It kept the recommendation and added
  options D, E, and F.
- `options-check-5.md`, over item 48. It kept the recommendation and added
  options E, F, and G.
- The check of item 49 stopped when its session ended. The user answered
  before it, so it did not run again.

### 29. Removing the App leaves billing running

**Answer, 4 November 2026:** B.

- **Severity:** High. **Effort:** Medium.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/billing.md`, "Subscription states", and
  `docs/protocol/overflow.md`, "Disconnect".
- **Background:**
  - An owner can remove or suspend the App on GitHub, without opening
    ciproxy.
  - ciproxy then loses its GitHub access. It cannot start runners, or
    delete the scale sets that stay on GitHub.
  - The machines stay connected, Stripe keeps charging, and the data and
    the Boat key stay.
  - After a removal, the dashboard probably cannot check the membership of
    a person, so the payer may not reach the portal.
  - A design call lets one Business subscription pay for several
    organizations.
- **Problem:** No requirement says what stops, what stays, and what comes
  back after a new install.
- **Impact if no action:** Stripe charges an organization that left, and
  ciproxy keeps its Boat key and its data with no end.
- **Root question:** When an organization removes or suspends the App, what
  stops, what stays, and what comes back after a new install?
- **Options:**
  - **A.** Change nothing. The customer cancels in the Stripe customer
    portal, which the dashboard may no longer open. Effort: Small.
  - **B.** End the subscription at the end of the period. Revoke the
    machines, and delete the data after 30 days. Effort: Medium.
  - **C.** B, but end the subscription at once, with a refund. Effort:
    Medium.
  - **D.** Treat a removal as a pause of 30 days. Effort: Medium.
    - Every machine drains: it finishes its jobs and keeps no warm runners.
    - Boat disconnects as `overflow.md` says, so ciproxy deletes the key.
    - The subscription ends at the end of the period. In a Business pool,
      only that organization leaves the pool.
    - A new install within 30 days stops the drain, and keeps the machines,
      the labels, and the settings.
    - After 30 days, ciproxy removes the machines and deletes the data.
    - A suspension drains the machines too, and an unsuspend stops the
      drain.
  - **E.** B, but delete the data at once. A mistaken removal then loses
    the machines, the labels, the settings, and the job history. Effort:
    Medium.
- **Left out:** A "Leave ciproxy" page as the only path, because an owner
  can remove the App on GitHub without opening ciproxy.
- **Wider fix:** None, because this is one new rule.
- **Previous examples:** None found.
- **Related:** Item 37 sets how long ciproxy keeps data. Item 8 decides
  whether one Business subscription pays for several organizations.
- **Recommendation:** D, changed from B by the options check. It ends the
  payments as B does, deletes the Boat key at once, and lets a returning
  organization keep its machines.
- **Action:** ask the user, because no requirement covers a removal or a
  suspension, so each option adds new behaviour.

### 31. The runtime on GCP is open

**Answer, 4 November 2026:** B.

- **Severity:** High. **Effort:** Small.
- **Source:** Design.
- **Category:** Code structure.
- **Location:** Milestone 5.
- **Terms:**
  - **Scale set session:** a long poll through which GitHub sends the job
    messages of one label.
  - **Cloud Run worker pool:** a Cloud Run resource for background work
    with no HTTP requests. It is generally available since April 2026.
- **Background:**
  - The control plane keeps a WebSocket to each agent, and a scale set
    session for each label. The scaler and the Boat checks run all the
    time.
  - The site, the dashboard, and the image downloads are static, so a
    bucket serves them with every option.
  - `futex-ai/cli-release` also publishes its release server as a
    container image, so it runs on Cloud Run with every option.
  - With more than one process, the scaler needs a route to the process
    that holds the WebSocket of an agent. A, B, and D need it.
- **Problem:** No decision names the GCP runtime of the long-lived work.
- **Impact if no action:** The deployment cannot start.
- **Root question:** Which GCP runtime keeps the agent connections, the
  scale set sessions, and the scaler running, with the least upkeep?
- **Options:**
  - **A.** One Cloud Run service, always on, with its CPU always on. A
    WebSocket lasts up to 60 minutes, so each agent reconnects once an
    hour. Effort: Small.
  - **B.** GKE Autopilot with Helm. The team keeps the cluster and the
    charts up to date. Effort: Medium.
  - **C.** A Compute Engine VM, which the team patches. One VM is one point
    of failure. Effort: Medium.
  - **D.** Cloud Run in two parts. A service holds the API, the WebSockets,
    the webhooks, and the release server. A worker pool runs the sessions,
    the scaler, and the Boat checks. Effort: Medium.
- **Left out:** A runtime outside GCP, because the user chose GCP.
- **Wider fix:** None, because this is a one-time choice.
- **Previous examples:** None found.
- **Recommendation:** A. It needs the least upkeep, and agents reconnect
  after each deploy anyway. D is the next step when the background work
  grows.
- **Action:** ask the user, because the options trade upkeep against
  limits, and only the user knows whether the team runs GKE.

### 32. No allowed source gives the branch of a job

**Answer, 4 November 2026:** A, with B as a follow-up item.

**Status, 4 November 2026:** the spike of option B passed.

- **Severity:** High. **Effort:** Medium.
- **Source:** Implementation.
- **Category:** Security.
- **Location:** `docs/protocol/security.md`, "GitHub App permissions", and
  `docs/protocol/cache.md`, "Trusted write-back". The mockups
  `marketing/security.tsx:24`, `marketing/how-it-works.tsx:21`, and
  `auth/install-app.tsx:46`.
- **Terms:**
  - **Job-started hook:** a script that the runner runs in the job VM
    before the first step of the job.
- **Background:**
  - Trusted write-back needs the ref and the event of each job from a
    source that job code and its author cannot change. Only own machines
    need it, because the GitHub cache applies its own scopes on Boat.
  - The scale set messages carry the ref of the workflow file, which
    differs from the ref of the job in a reusable workflow.
  - The App asks for "Self-hosted runners" and "Metadata" only. The site
    promises "Runner access only".
  - The spike ran 31 real jobs on runner 2.338.2. The first report of the
    hook matched the GitHub API in each case, also for reusable workflows,
    `pull_request_target`, and environment overrides.
  - The hook ran before job containers, services, action pre-steps, steps,
    and composite actions. No tamper attempt changed a first report.
  - A fork was not tested. A pull request never writes shared entries, so
    this does not affect trusted write-back.
- **Problem:** With its two permissions, ciproxy has no trusted source for
  the ref and the event of a job unless it takes option B.
- **Impact if no action:** A pull request can poison the cache and the
  Docker disk on a machine.
- **Root question:** How does ciproxy learn the ref and the event of each
  job from a source that job code and its author cannot change?
- **Options:**
  - **A.** Add the "Actions" read permission, and read each run from the
    GitHub API. It also reads the logs of every run. The docs and three
    mockups drop "Runner access only". Effort: Medium.
  - **B.** Take the facts from the first report of the job-started hook,
    which the agent receives outside the VM. Effort: Medium.
    - The spike proved it for runner 2.338.2. The image pins the runner,
      and a test repeats the proof before each runner update.
    - A pull request event never writes shared entries, and unknown facts
      give a private cache scope.
  - **C.** Trust no ref. The cache on a machine serves only what the
    GitHub cache finds for the job. This undoes answer B to item 18, and
    no Docker disk keeps its changes. Effort: Small.
  - **D.** An organization webhook for `workflow_run` and `workflow_job`,
    signed with a secret from ciproxy. An owner adds it on GitHub. Jobs get
    no trusted writes until then. Effort: Large.
- **Left out:**
  - The runtime token of the job, because GitHub documents no way to check
    it.
  - `jobWorkflowRef` and `eventName`, because the ref is that of the
    workflow file.
  - The OIDC token of the job, because each workflow must then grant
    `id-token: write`, and the cache needs no workflow edits.
- **Wider fix:** A. It holds whatever the runner does in the VM. B depends on
  the order of the runner's setup steps, which its test checks before each
  update.
- **Previous examples:** The job hooks spike, and the job facts spike.
- **Related:** Item 8: the Docker disks also need a trusted ref. Item 18:
  option C undoes its answer.
- **Recommendation:** B, changed from A after the spike. It keeps the App at
  two permissions and the "Runner access only" promise, and a test repeats
  the proof for each runner update. A stays the fallback if an update
  breaks the order.
- **Action:** ask the user, because the options trade a wider permission
  against a dependency on the runner, and the user asked to decide after
  the spike.

### 37. Data retention has no rule

**Answer, 4 November 2026:** B.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Security.
- **Location:** `docs/protocol/security.md`, "Secrets and job data", and
  `docs/protocol/metering.md`, "Savings report".
- **Background:**
  - Free shows job details for 7 days, Team for 30, and Business for 90.
  - Every tier keeps the daily totals of at least the last 30 days, this
    month, and last month.
  - Every option keeps the last report of each machine and 7 days of its
    cache hits, because no page shows more.
  - Every option keeps a person only while the person has a role or an
    open request. Milestone 5 sets the GCP periods of logs and backups.
  - The privacy policy must state each limit.
- **Problem:** No source covers tier changes, or an upper limit for the
  job details and the daily totals.
- **Impact if no action:** The privacy policy cannot state a limit.
- **Root question:** Which data does ciproxy keep, and how long?
- **Options:**
  - **A.** The window of the current tier, and 62 days of totals. Effort:
    Small.
  - **B.** 90 days of job details on every tier. Each page shows only the
    window of the tier. Effort: Small.
  - **C.** A, with 13 months of totals. Effort: Small.
  - **D.** A or B, plus a setting where an Admin makes the window shorter.
    It adds a setting, a mockup state, and a rule for tier changes.
    Effort: Medium.
- **Left out:** Keeping everything, because the policy then sets no limit.
- **Wider fix:** None.
- **Previous examples:** None found.
- **Related:** Item 29 decides the data of an organization that leaves.
- **Recommendation:** B, changed from A by the options check. It needs no
  deletion at a tier change, and an upgrade shows the older jobs at once,
  which helps sell the paid tiers.
- **Action:** ask the user, because no requirement covers retention, and
  the privacy policy states it.

### 39. Taxes have no rule

**Answer, 4 November 2026:** A, for now.

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/billing.md`, "Tiers" and "Checkout".
- **Terms:**
  - **Merchant of record:** the company that sells to the customer in law,
    and that owes the sales tax and the VAT.
  - **Stripe Managed Payments:** a Stripe service that makes Stripe the
    merchant of record. It is generally available since April 2026.
- **Background:**
  - Prices are in US dollars, and no source says whether they include tax.
  - The duty to collect tax depends on the registrations of the seller.
  - The checkout review cannot know the tax before Checkout has the
    address, so with B or D it says that tax comes on top.
- **Problem:** No source says who owes the tax of each payment, or how the
  price shows it.
- **Impact if no action:** ciproxy can owe tax that it did not collect.
- **Root question:** Who owes the sales tax and the VAT of each payment,
  and how does the price show it?
- **Options:**
  - **A.** No tax at launch. Effort: Small.
  - **B.** Stripe Tax, for 0.5% of each payment, with "plus tax" and tax
    IDs. The company registers, collects, and files. Effort: Medium.
  - **C.** B, with prices that include tax. Effort: Medium.
  - **D.** Stripe Managed Payments, for 3.5% of each payment on top of the
    Stripe fees. Stripe collects and files the tax in over 80 countries.
    Effort: Medium.
    - Customers see Link, a Stripe brand, as the seller, and Stripe can
      refund a payment within 60 days in some cases.
    - Stripe must accept the business, in a listed country such as the US,
      the UK, or an EU country.
    - It shows local currency at checkout by default, so ciproxy turns
      that off or changes the US dollar rule.
  - **E.** ciproxy keeps its own tax rates and gives them to Checkout. It
    then carries each error. Effort: Large.
- **Left out:** Another merchant of record, such as Paddle, because the
  billing decision keeps Stripe, and D gives the same result on Stripe.
- **Wider fix:** None.
- **Previous examples:** None found.
- **Recommendation:** D, changed from B by the options check. It removes
  every tax registration and filing for about 3% more of each payment.
  B is better if the company already collects tax with Stripe Tax.
- **Action:** ask the user, because only the user knows the registrations
  of the company, and the options trade fees against tax work.

### 45. No gate check for the generated API types

**Answer, 4 November 2026:** B.

- **Severity:** Low. **Effort:** Small.
- **Source:** Design.
- **Category:** Repository rule.
- **Location:** `xtask/src/check/phases.rs`. Milestone 14.
- **Background:** The dashboard imports types that tools generate from the
  Rust API, and the repository commits them. The `shared-types` skill puts
  a drift check in the gate once the user approves it.
- **Problem:** Committed types can go stale.
- **Impact if no action:** The dashboard breaks at run time.
- **Root question:** How does a change to the Rust API types reach the
  dashboard before the change merges?
- **Options:**
  - **A.** No phase, and review. Effort: Small.
  - **B.** A phase that regenerates and fails on a diff. Effort: Small.
- **Left out:** Types generated at build time, because the `shared-types`
  skill commits them, so that the review diff shows each contract change.
- **Wider fix:** B, because it catches each stale type.
- **Previous examples:** None found.
- **Recommendation:** B. It is the drift check that the `shared-types` skill
  names.
- **Action:** ask the user, because a new gate check needs the user.

### 47. A Boat sandbox cannot end itself

**Answer, 4 November 2026:** G.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, from the Boat spike.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/overflow.md`, "Backend contract". Milestones
  10 and 12.
- **Terms:**
  - **TTL:** `ttlSeconds`, the lifetime that Boat gives a sandbox at its
    start. Boat stops the sandbox when it ends, with no call.
  - **In-sandbox key:** a key that Boat writes into each sandbox. It works
    only on that sandbox, and it can stop it.
- **Background:**
  - The backend contract says that a backend ends a VM by itself when its
    runner exits, so the end needs neither the key nor the control plane.
  - The reason: an outage, or a lost key, must not leave sandboxes billing.
  - `overflow.md` also requires that the control plane deletes each sandbox
    when its job ends, and deletes orphans after an outage. So the end
    without ciproxy is a backstop.
  - The spike used `noEnv: true`, which keeps the secrets of the customer's
    Boat account out, and also removes the in-sandbox key.
  - So `boat stop self` returned HTTP 401. A guest poweroff left the
    sandbox `idle`, and Boat kept counting its time.
  - A TTL of 90 seconds removed a sandbox after about 95 seconds.
  - Job code runs as root, so it can read every credential in the sandbox.
- **Problem:** With no credential in the sandbox, nothing ends it when its
  runner exits, so Boat fails the backend contract.
- **Impact if no action:** The Boat backend cannot meet its contract, and
  the overflow milestone cannot finish.
- **Root question:** What ends a Boat sandbox after its runner exits when
  ciproxy cannot delete it, and how long may it bill before that?
- **Options:**
  - **A.** Change the contract: a VM ends by itself when its runner exits,
    or at a lifetime that ciproxy sets at the start. Each sandbox gets a
    TTL of 6 hours, or 2 hours on a Boat free trial. Effort: Small.
    - A Boat job then stops at the TTL, as a GitHub-hosted job stops at 6
      hours. On an own machine, a job can run for up to 5 days.
    - In an outage, each idle sandbox bills until ciproxy is back or its
      TTL ends: about $7 an hour for 100 `large` sandboxes.
  - **B.** A, and the user asks Boat to end a sandbox when its guest powers
    off. The contract can then come back. Effort: Small.
  - **C.** Keep the contract, and make AWS EC2 the first backend. EC2 can
    end an instance at its own shutdown. Effort: Large.
  - **D.** Keep the contract with the in-sandbox key. Each sandbox starts in
    a Boat environment that passes only this key. The guest runs `boat stop
    self` after the runner exits, and A's TTL stays as a backstop. Effort:
    Medium.
    - Job code gets the key, and can stop, run commands, use SSH, publish a
      port, and prompt Boat's agent, on its own sandbox.
    - A spike must confirm the environment, the stop, and what the key can
      change. Large if Setup must ask the customer to create it.
  - **E.** A, and the control plane renews each TTL every hour while its
    runner runs. A Boat job then has no 6-hour cap. Effort: Small.
  - **F.** Keep the contract, with a stop-only key that ciproxy creates for
    each sandbox. Only an admin key can create keys in Boat, so ciproxy
    must hold an admin key of each customer's account. Effort: Medium.
  - **G.** A, and the guest in the sandbox tells the control plane when its
    runner exits, with a token for that sandbox only. The control plane
    then deletes the sandbox at once. This is the user's proposal of
    4 November 2026. Effort: Small.
    - It ends a sandbox seconds after its runner exits, also when GitHub's
      message comes late, or never, as for a runner that fails at start.
    - Job code can send the message too, but it can only end its own job.
    - Like the message from GitHub, it needs the control plane and the key,
      so the TTL of A stays the backstop for an outage.
- **Left out:**
  - The customer's key in the sandbox, because job code would control the
    whole Boat account, which `security.md` forbids.
  - No TTL, because a sandbox would then bill without end after ciproxy
    loses the key.
  - A separate cleanup service, because it needs the same key, and the
    same outage stops it.
  - A powered-off sandbox that stops billing, because Boat does not
    document it.
- **Wider fix:** A. Its contract text bounds every backend: Boat meets it
  with a TTL, and EC2 with its shutdown behaviour.
- **Previous examples:** None found.
- **Related:** Item 19, answered D, gives the reason that no new path leaves
  a sandbox. G adds one path, for the end of the runner only.
- **Recommendation:** G, changed from A after the user's question. It keeps
  the TTL backstop of A, and it ends each sandbox at once when its runner
  exits. D is the way back to the strict contract.
- **Action:** ask the user, because A changes a requirement of the backend
  contract in `overflow.md`.

### 48. No rule says how organizations join a pool

**Answer, 4 November 2026:** a ciproxy workspace that holds several
organizations, close to G. See the answers above.

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation, from the docs work of item 8.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/billing.md`, "Business pools", "Checkout", and
  "Subscription states", and `docs/protocol/metering.md`, "A Business
  pool". The mockup `screens/marketing/price-cards.tsx:23`. Milestones 1,
  16, 17, 25, and 26.
- **Terms:**
  - **Business pool:** the organizations that one Business subscription
    pays for. They share one count of starts and one count of machines.
  - **Paying organization:** the organization whose Stripe subscription
    pays for the pool.
- **Background:**
  - Answer A to item 8 keeps the six features in the tiers, and marks each
    feature without a screen "Coming later".
  - The plan listed "several GitHub organizations" as a launch feature,
    because a design call defined its shared counts. The docs now state it.
  - No mockup shows a pool at all: no list of its organizations, no shared
    Plan and usage, and no join.
  - Depot and Ubicloud bill an account of their own that holds several
    GitHub organizations. Blacksmith bills each organization on its own.
- **Problem:** No rule says who joins an organization to a pool, where, or
  what happens to its plan and its starts. No rule says who manages the
  shared subscription.
- **Impact if no action:** The build guesses a launch flow. As the docs are
  now, each payer of each joined organization can change the card and the
  add-ons of the paying organization.
- **Root question:** How does a customer pay one Business subscription for
  several organizations, so that no person binds or charges an
  organization where that person cannot pay?
- **Options:**
  - **A.** A person with the Admin or the Billing role in both
    organizations joins them on Plan and usage, and can take one out.
    Effort: Medium.
  - **B.** A payer of the Business organization creates an invite, and an
    Admin of the other organization accepts it. Effort: Medium.
  - **C.** The ciproxy team joins the organizations on request, with no
    screen. Effort: Small.
  - **D.** Mark "several GitHub organizations" "Coming later" too, because
    it has no screen. Effort: Small.
    - Business then sells its larger counts, 90 days of job history, and
      priority support. It costs at most $20 a month less than Team with
      add-ons, and only above 136,000 starts and 10 machines.
    - The Business card stops leading with several organizations.
    - The pool rules leave the protocol docs for a follow-up item, with
      the rules of E.
  - **E.** A, with the rules that A leaves open. Effort: Medium.
    - Only an organization on Free can join. Its starts count in the pool
      from the join.
    - Only the payers of the paying organization change the tier, the
      add-ons, and the card.
    - The payers of a joined organization can only take it out. It then
      moves to Free at once.
    - When the paying organization leaves, the subscription ends at the
      end of the period, and the others move to Free.
  - **F.** E, but an organization joins only at its first install, from "Add
    an organization" on Plan and usage. An organization that already uses
    ciproxy cannot join at launch. Effort: Medium.
  - **G.** A billing account above the organizations holds the Stripe
    customer and has its own payers, as at Depot. It changes the roles and
    every billing page. Effort: Large.
- **Left out:**
  - A pool from a GitHub enterprise account, because it fails for
    organizations outside an enterprise.
  - A join by any member, because a join changes the plan of both
    organizations, and only Admin and Billing people choose a plan.
  - The switch between organizations as the feature, because every tier
    already lets one sign-in use several organizations.
  - A price for each extra organization, because it changes the approved
    pricing.
- **Wider fix:** E. Its rule holds for every join path: only the payers of
  the paying organization manage the subscription.
- **Previous examples:** None found.
- **Recommendation:** D, kept by the options check. It applies the rule of
  answer A to a feature with no screen, and it is the only Small option. If
  the pool must ship at launch, E is better than A, because A leaves the
  card of the paying organization open to every joined organization.
- **Action:** ask the user, because D changes a launch feature that the docs
  state, and the other options add a flow that no requirement covers.

### 49. Where the Boat backend lives

**Answer, 4 November 2026:** C.

- **Severity:** Medium. **Effort:** Large.
- **Source:** Design, from the user's question of 4 November 2026.
- **Category:** Code structure.
- **Location:** Milestone 12, and the repository `futex-ai/sandbox`.
- **Terms:**
  - **`futex-ai/sandbox`:** a public Rust workspace under MIT. Its crate
    `sandbox-interface` holds a trait for remote sandboxes of any provider,
    and `sandbox-e2b` implements it for E2B. Juno uses it.
- **Background:**
  - Milestone 12 plans `ciproxy-backend-interface`, a small trait for the
    backend contract of `overflow.md`, and `ciproxy-backend-boat`.
  - `SandboxBackend` has 28 functions: the lifecycle, snapshots, image
    preparation, processes, files, terminals, ports, and desktop screens.
  - It has no VM size request, no list of the sizes of an account, no
    refusal with a retry time, and no price. A one-shot lifetime is at
    most 1 hour.
  - ciproxy needs each of these, and a lifetime of 6 hours.
  - Boat has processes, files, SSH, a desktop, ports, and snapshots, so a
    Boat adapter can implement most of the trait.
  - That repository allows breaking changes, because nothing runs on it in
    production.
- **Problem:** A shared interface for sandbox providers exists, but the
  plan does not use it, and it lacks what ciproxy needs.
- **Impact if no action:** ciproxy builds a second sandbox interface, and
  Boat support does not reach the other products.
- **Root question:** Where do the Boat backend and its interface live, so
  that other products and providers share them, without delaying overflow?
- **Options:**
  - **A.** Keep ciproxy's own small trait for the MVP, as planned. Move it
    onto `sandbox-interface` after launch, with a second provider. Effort:
    Medium.
  - **B.** Add `sandbox-boat` to `futex-ai/sandbox` on the full trait, and
    add sizes, the size list, refusals, prices, and longer lifetimes to the
    interface. Boat must first implement all 28 functions, also the
    terminals and the desktop screens. Effort: Large.
  - **C.** B, but first split `SandboxBackend` into capabilities: the
    lifecycle, snapshots, processes, files, terminals, and screens.
    `sandbox-boat` starts with the four that ciproxy needs. Juno must
    adopt the split. Effort: Large.
- **Left out:** Only the types of `sandbox-interface` in ciproxy's own
  trait, because that gives no shared interface.
- **Wider fix:** C. Each provider then implements only what it supports, and
  each product depends only on what it uses.
- **Previous examples:** The extraction of `sandbox-interface` and
  `sandbox-e2b` out of Juno, in the plan
  `extract-sandbox-interface-and-e2b.md` of that repository.
- **Recommendation:** C. It gives the shared interface that the user wants,
  and Boat does not wait for terminals and screens that ciproxy never uses.
- **Action:** ask the user, because the options trade the speed of the
  overflow work against a shared interface, and C changes the trait of
  Juno.

### 50. Adding a machine needs an unclear runner right

**Answer, 4 November 2026:** the Admin role alone, which reverses item 28.
See the answers above.

- **Severity:** High. **Effort:** Medium.
- **Source:** Implementation, from the docs work of item 48.
- **Category:** Security.
- **Location:** `docs/protocol/security.md`, "Roles", and
  `docs/protocol/workspaces.md`, "Add an organization".
- **Terms:**
  - **Runner right:** the GitHub right to manage the self-hosted runners of
    one organization. Every owner has it, and so does GitHub's CI/CD admin
    role.
- **Background:**
  - Item 28 decided that adding a machine needs the Admin role and the
    runner right, because a machine gets the secrets of the jobs that it
    runs. It rejected the Admin role alone.
  - A machine now runs the jobs of every organization of its workspace, and
    the Boat account of the workspace runs their overflow jobs.
  - Secrets reach a machine or Boat by five paths: adding a machine,
    connecting Boat, adding an organization, a re-join within 30 days of a
    removal, and an install token made before an organization joined.
  - When an owner approves an install that an Admin asked for, GitHub shows
    that owner the App, not the workspace.
- **Problem:** No rule says whose runner right each path needs, so the
  secrets of one organization can reach machines that nobody with its
  runner right accepted.
- **Impact if no action:** A person with the runner right of one
  organization can give a machine or a Boat account the job secrets of
  another organization.
- **Root question:** How do the job secrets of each organization reach only
  the machines and the Boat account that a person with its runner right
  accepted?
- **Options:**
  - **A.** Adding a machine needs the runner right of every organization of
    the workspace. Adding an organization needs the runner right of that
    organization. Effort: Medium.
  - **B.** Adding a machine needs the runner right of one organization, and
    the App install of each organization counts as its consent. Effort:
    Small.
  - **C.** A machine runs only the jobs of the organizations whose runner
    right the person who added it had. Effort: Large.
  - **D.** A person with the runner right of a new organization confirms its
    join, on a page that names the workspace, its machines, and its Boat
    account. Machines added later need the right of one organization only.
    Effort: Medium.
  - **E.** ciproxy records which organizations each machine and the Boat
    account may serve, and the scaler starts a runner only where a record
    exists. People with the runner right add the records. Effort: Large.
  - **F.** A, plus the paths that A leaves open. Effort: Medium.
    - Connecting or changing the Boat key needs the runner right of every
      organization.
    - When the person who started a join lacks the runner right of the new
      organization, the join waits until a person with that right confirms
      it, on a page that names the workspace, its machines, and its Boat
      account.
    - A re-join within 30 days follows the rule of an added organization.
    - When an organization joins, ciproxy revokes the open install tokens.
- **Left out:**
  - The Admin role alone, because item 28 rejected it.
  - A GitHub App for each workspace, so that GitHub names the workspace,
    because GitHub allows at most 100 Apps for each account.
  - VMs that hide a job from the host of its machine, because most customer
    hardware lacks the processor support.
  - A consent that ends when its person loses the right, because a machine
    works until a person removes it.
- **Wider fix:** E. It checks the one place where secrets go, the start of a
  runner, so a new path needs no check of its own.
- **Previous examples:** Item 28 of the foundation plan.
- **Related:** Item 51: its options use the same runner right.
- **Recommendation:** F, changed from A by the options check. It keeps the
  reason of item 28 for every organization and closes the four paths that
  A leaves open. E protects the codebase best, at a Large cost.
- **Action:** ask the user, because item 28 covers one organization only,
  and each option adds a rule.

### 51. An organization with the App cannot join

**Answer, 4 November 2026:** D.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, from the docs work of item 48.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/workspaces.md`, "Take an organization out"
  and "Add an organization".
- **Terms:**
  - **Take-out:** an Admin takes an organization out of its workspace.
- **Background:**
  - "Add an organization" works through a new App install, and GitHub
    installs an App only once on each organization.
  - Today a take-out leaves the App installed, so the organization cannot
    join a workspace again until an owner removes the App.
  - The App can remove its own install through the GitHub API.
  - The answer to item 50 makes the Admin role the trust to control the
    runners of every organization of the workspace, so an Admin takes an
    organization out.
- **Problem:** No rule says how an organization leaves so that it can join
  again, or what happens to the last organization.
- **Impact if no action:** A taken-out organization cannot come back until
  an owner removes the App, and a workspace can pay with no organization.
- **Root question:** How does an organization leave a workspace so that it
  can join one again later, and what happens to the last organization?
- **Options:**
  - **A.** No join while the App stays: an owner removes the App, and an
    Admin adds the organization again. The last organization cannot be
    taken out. Effort: Small.
  - **B.** "Add an organization" also lists the organizations that have the
    App and no workspace, to an Admin who owns them. Effort: Medium.
  - **C.** B, and an owner can also move an organization out of another
    workspace. Effort: Large.
  - **D.** A take-out removes the App. ciproxy deletes the scale sets of the
    organization, lets its running jobs finish, and removes its own
    install. The organization can join any workspace again as a new
    install. The last organization cannot be taken out. Effort: Small.
  - **E.** A take-out only pauses the jobs of the organization, as a
    suspension does, and an Admin can start them again. Effort: Small.
  - **F.** An Admin can take out the last organization, which closes the
    workspace as a removal of the last organization does. Effort: Small.
- **Left out:** A join through the GitHub settings page of an install,
  because B is better on every point.
- **Wider fix:** D. Its rule is that ciproxy keeps no install that no
  workspace serves.
- **Previous examples:** None found.
- **Related:** Item 50: an Admin acts for every organization of the
  workspace.
- **Recommendation:** D, changed from B by the options check. It removes the
  cause, an install that no workspace serves, and it needs no new join
  path.
- **Action:** ask the user, because no requirement covers the take-out, and
  each option adds a rule.
