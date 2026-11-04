# ciproxy MVP

Status: Active. On branch `calummoore/ci-proxy-load-balancer`, no pull
request yet. The pull request that completes the work deletes this plan, as
the `plans` skill says.

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
- **Spikes:** the scale set, Firecracker, cache, and job hooks spikes
  confirm their mechanisms. The Boat spike got HTTP 402 `billing_required`,
  and it runs again when Firna has a Boat plan. The Boat work waits for it.
- **Actions:** `Tags:` milestones go to the `ui implementor`, Milestones 2
  and 24 to the `docs implementor`, and the rest to the `non-ui implementor`.
- **Open parts:** `ciproxy-protocol`, `ciproxy-agent`, `ciproxy-guest`,
  `ciproxy-cache-sidecar`, the runner patch, and `image/` are Apache-2.0, as
  item 20 decided. They depend only on each other, so that their public copy
  builds alone. Other crates are closed.
- **One host:** the control plane serves `app.ciproxy.dev/api/`, `/agent/`,
  and `/hooks/`. Reason: three hosts in all, and no cross-origin calls.
- **Job facts:** the repository of a job comes from the scale set messages,
  never from the VM. Item 32 decides the source of its branch. Reason: job
  code runs in the VM.
- **Docs of the code:** each milestone updates the READMEs and the
  `AGENTS.md` commands that it changes.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

Milestone 1 of the foundation plan applies the answers and items 8 to 11.
This milestone adds the states of the new items, and the legal pages.

- [ ] Add a removed App, an ended sign-in, and a personal account.
- [ ] Add the install command, a refused label name, too little disk, and
      mixed machines.
- [ ] Add a payment that Stripe still confirms, and two banners at once.
- [ ] Update the cache fields of the job page, as item 19 says.
- [ ] Add the terms and privacy pages. Run the mockup check, and smoke-test
      the changed pages.
- [ ] Update this plan for each problem that the mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

Each answer goes into the protocol doc that it changes, and its `Open:`
line goes.

- [ ] Write each answer that changes a doc. Waits for items 32 and 37.

## Milestone 3: Protocol crate

Depends on: none

`ciproxy-protocol` holds the types of the agent connection. My call: one
WebSocket over HTTPS with JSON. Reason: machines open no inbound port.

- [ ] Add the rules of "Slots" in `job-routing.md`, tested on its examples.
- [ ] Add the registration, the hello, the report every 10 seconds, and the
      commands for runners, job facts, drain, remove, cache, and images.
- [ ] Add the vsock messages of `ciproxy-guest`. Use tagged enums, hide
      secrets from `Debug`, and snapshot each message with `insta`.

## Milestone 4: Store

Depends on: none

Postgres through Diesel, as the `databases` skill says: `ciproxy-schema`,
`ciproxy-store-interface`, and `ciproxy-store-pg`. My call: one schema file
for each area. Reason: parallel branches then do not conflict.

- [ ] Add the core tables: organizations, labels, scale sets, machines,
      tokens, runners, jobs, settings, and Boat accounts.
- [ ] Keep tokens as hashes and Boat keys as ciphertext. Add an advisory
      lock for each scale set session.
- [ ] Test the queries on the Postgres of `DATABASE_URL`, which CI starts.

## Milestone 5: Deployment: hosting and DNS

Depends on: none

Production runs in the GCP project `ciproxy`, and staging and tests in
`internal-498318`. My calls: `europe-west1`, Terraform under `deploy/`, and
Workload Identity Federation. Reason: EU data, reviews, and no stored keys.

- [ ] Add Cloud SQL, Secret Manager, and Cloud KMS for the Boat keys.
- [ ] Run the control plane and the `futex-ai/cli-release` server on GKE
      Autopilot with Helm, as item 31 says.
- [ ] Set the release server to `get.ciproxy.dev`, `ciproxy-agent`, the tag
      prefix `ciproxy-agent-v`, and a token for this repository only.
- [ ] Serve the site, the dashboard, and signed image URLs from a Cloud
      Storage bucket, through Cloud CDN and one HTTPS load balancer. My call:
      they are static files, so they need no runtime.
- [ ] Create the DNS zone, which the user delegates.
- [ ] Deploy staging on each push, and production on each `deploy-` tag. Add
      uptime alerts.

## Milestone 6: GitHub App and scale sets

Depends on: 4

`ciproxy-github` acts as the App, with one scale set and one listener for
each label. The scale set spike confirms it.

- [ ] Add the App tokens, the repositories, the default branch, and the
      installation events. Create the default labels of `job-routing.md`.
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
- [ ] Forward the facts of each job to its agent. Waits for item 32.
- [ ] Create and remove the just-in-time registrations.

## Milestone 7: Scaler

Depends on: 3

`ciproxy-scaler` turns a state snapshot into actions: start a runner on a
machine or on Boat, stop a runner, or delete a sandbox.

- [ ] Apply `job-routing.md` from "Where a job runs" to "Failures", and the
      size changes of "Custom labels". Give each start a typed reason.
- [ ] Unit-test each rule with a hand-worked state.

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
The Firecracker spike confirms it for VMs. Boat waits for its spike.

- [ ] Measure the full image first. If it fits no size, propose a smaller
      image as a decision.
- [ ] Define it under `image/` from a pinned `actions/runner-images`, with
      Docker, the runner, and a kernel.
- [ ] Add `ciproxy-guest`: it takes the registration over vsock, starts the
      runner, and powers off the VM after the runner exits.
- [ ] Publish it with its SHA-256, and as a shared Boat snapshot. Rebuild
      before GitHub stops accepting the runner.

## Milestone 11: Cache sidecar and runner patch

Depends on: 3

A patched runner sends the cache calls to a sidecar in the VM, which reads
and writes through to GitHub. The machine cache keeps entries after GitHub
evicts them. Boat runs the stock runner. The cache spike confirms it.

- [ ] Rename `ACTIONS_CACHE_URL`, `ACTIONS_CACHE_SERVICE_V2`, and
      `ACTIONS_RESULTS_URL` in `Runner.Worker.dll` at equal length.
- [ ] Add `ciproxy-cache-sidecar`. Pass other results calls through, and
      fall back to GitHub on any error.
- [ ] Scope each entry from the job facts, as "Cache scopes" says. Waits for
      item 32.
- [ ] Add "Clear cache" for one machine, eviction, and the reports.
- [ ] Test the sidecar against a fake GitHub cache.

## Milestone 12: Boat backend and the size list

Depends on: 4

`ciproxy-backend-interface` holds the contract of `overflow.md`, and
`ciproxy-backend-boat` implements it. **Unconfirmed:** the Boat spike.
**Fallback:** AWS EC2 behind the same trait, which needs a user decision.

- [ ] Check keys. Read `machineTypes` of `GET /limits` every 5 minutes. My
      call: a Boat plan can change at any time.
- [ ] Pick the cheapest size that fits, or the largest.
- [ ] Start from the shared snapshot with marks, a private registration
      file, a `ttlSeconds` backstop, and an `Idempotency-Key`.
- [ ] Delete idle marked sandboxes, and map each refusal. Count the spend as
      run time and start charges. My call: Boat bills these two parts.
- [ ] Encrypt the keys with Cloud KMS. Test against a fake Boat server.

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
- [ ] Sign in with the GitHub App. Apply "Roles" in
      `security.md`, with the access requests.
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

- [ ] Count the starts of each period, the machines, the soft limits, and
      the offer.
- [ ] Compute the savings report, and keep the daily totals. Waits for item
      37.
- [ ] Unit-test each rule with hand-worked numbers.

## Milestone 17: Billing with Stripe

Depends on: 4

`ciproxy-billing` keeps the tier and the add-ons in step with Stripe, as
`billing.md` says, after `futex-ai/accounting`. My call: the return page and
the event both read the session, so the tier changes once.

- [ ] Create the prices, and open Checkout, with no tax, as item 39 says.
- [ ] Take each signed event once. Change tiers and add-ons with proration,
      and open the portal.
- [ ] End the subscription of a removed App at the end of its period, and
      take a leaving organization out of its Business pool.
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

- [ ] Build the landing page, the pricing page, the footer, and the legal
      page frames.
- [ ] Theme the docs. Restyle the Starlight search to its mockup as far as
      CSS allows, and record each gap as a follow-up item.
- [ ] Smoke-test each page at 1440, 720, and 390 pixels.

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

This milestone builds the features that ship at launch, as item 8 says:
Docker disks on Team and Business, and several organizations on Business.
The job hooks spike confirms the Docker disks.

- [ ] Attach the Docker disk in the job-started hook, as "Docker disks" in
      `cache.md` says.
- [ ] Give the disk to the first default-branch job, and a reflink copy to
      each other job. Waits for item 32.
- [ ] Use drive IDs of letters, digits, and underscores. Keep each Docker
      image store on the disk.
- [ ] Discard a disk that fails to mount. My call: a rebuild loses only
      time.
- [ ] Join organizations in one Business pool.

## Milestone 26: Dashboard UI: tier features

Tags: ui

Depends on: 20, 21, 25

Each screen follows its mockup, as Milestone 19 says.

- [ ] Build the Docker disks of the machine page and the joining of
      organizations.

## Milestone 27: Launch

Depends on: every milestone from 1 to 26

The public copy goes live before the site, because the site links to it.

- [ ] Run the CI of ciproxy on ciproxy, and fix or record each problem.
      Close the follow-up item `expo-56-npm-advisories.md`.
- [ ] Draft the terms and the privacy policy, which the user writes or
      approves. Waits for item 37.
- [ ] Register the production App and the Stripe live mode, which need the
      accounts of the user.
- [ ] Deploy production. Smoke-test a first job, an overflow, and a live
      checkout that the user approves.
- [ ] Publish the open parts in a public repository that the user creates,
      and check that it builds alone. Check the licence of the installer
      of `futex-ai/cli-release`.
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
- Item 35: A sign-in lasts 30 days, with a membership check each hour. Reason:
  access ends soon after a person leaves the organization.
- Item 36: On a personal account, ciproxy shows a page that says it serves
  organizations only, with the link to remove the App. Reason: a GitHub
  organization costs nothing.
- Item 38: Checkout takes cards only, and its return page says when Stripe still
  confirms. Reason: the page never asks for a second payment.
- Item 40: The label `ciproxy` gets 50 GB of disk, if the Boat spike shows that
  space free on `default`. Reason: it then fits the cheaper Boat size.
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

Design calls for the parts that answer B to item 29 leaves open, my calls,
4 November 2026. The options check named these parts:

- The revoke covers every credential: the machine keys and the Boat key go
  at once. Reason: no credential of an organization that left stays usable.
- In a Business pool, a removal takes only that organization out of the
  pool. The subscription ends when its last organization leaves. Reason:
  the other organizations still use it.
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

Items 32 and 37 wait for the user. The options check of the `plans` skill
ran over items 29 to 45 on 4 November 2026 and changed the recommendations
of items 29, 32, 37, and 39. Its report is in
`.context/ciproxy-mvp-foundation/options-check-3.md`.

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
  - Trusted write-back needs the ref, the event, and the head repository of
    each job. A branch name is not enough, because a fork can use the
    name of the default branch.
  - These facts must come from a source that job code and its author
    cannot change.
  - The scale set messages carry `jobWorkflowRef` and `eventName`. That ref
    is the ref of the workflow file, and a push to any branch can call a
    reusable workflow at the default branch.
  - The App asks for "Self-hosted runners" and "Metadata" only. The site
    promises "Runner access only".
- **Problem:** With its two permissions, ciproxy has no trusted source for
  the ref and the event of a job.
- **Impact if no action:** A pull request can poison the cache and the
  Docker disk on a machine.
- **Root question:** How does ciproxy learn the ref and the event of each
  job from a source that job code and its author cannot change?
- **Options:**
  - **A.** Add the "Actions" read permission, and read each run from the
    GitHub API. It also reads the logs of every run. The docs and three
    mockups drop "Runner access only". Effort: Medium.
  - **B.** Take the facts from the first report of the job-started hook,
    which runs before any job code. A spike must prove the order of the
    runner's setup steps. Effort: Medium.
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
- **Wider fix:** A. It holds whatever the runner does in the VM, and B
  depends on the order of the runner's setup steps.
- **Previous examples:** The job hooks spike, with the "Actions" permission
  of the spike App.
- **Related:** Item 8: the Docker disk of its option E needs a trusted ref.
  Item 18: option C undoes its answer.
- **Recommendation:** A, changed from B by the options check. It needs no
  proof, and a later move to B drops the permission with no owner
  approval, while a move from B to A needs each owner to approve.
- **Action:** ask the user, because A adds a permission that `security.md`
  and the site rule out.

### 37. Data retention has no rule

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
