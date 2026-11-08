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
- [x] Add the rename of a workspace, the banner of a Free workspace with
      more than one organization, the job page with the mixed cache source
      and with no entries restored, a `labels` key in `related-docs.ts`,
      the warning when an Admin gives the Admin role, no state without the
      runner right after item 50, and the take-out of item 51: the dialog
      says that the App is removed, and the last organization has no
      take-out.
      Branch: `conductor/ciproxy-mvp-milestone-1-last-mockups`.

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
`internal-498318`. My calls: Terraform under `deploy/`, and Workload Identity
Federation. Reason: reviews, and no stored keys. Item 81 = C moves every
regional resource to `us-central1`. The Terraform state buckets of the seed
stay in `europe-west1`.

- [ ] Add Cloud SQL, Secret Manager, and Cloud KMS for the Boat keys.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define both projects, private SQL, secret containers, KMS, and IAM.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Orchestrator: run `deploy/scripts/bootstrap.sh` for the main
        preview, set the GitHub variables, and start the deploy workflow, as
        item 56 says. Done 6 November 2026: run 42145526077 passed every job.
        Branch: `calummoore/ci-proxy-load-balancer`.
  - [ ] Orchestrator: the same for production, at launch, as
        [the launch guide](../deploy/launch.md) says.
- [ ] Run the control plane and the `futex-ai/cli-release` server on GKE
      Autopilot with Helm, as item 31 says.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define Autopilot, Helm charts, SQL proxy, and runtime identities.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: deploy Helm. Enable the control plane after its runtime exists.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [ ] Set the release server to `get.ciproxy.dev`, `ciproxy-agent`, the tag
      prefix `ciproxy-agent-v`, and a token with read access to public
      repositories only, as item 70 says.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Configure release hosts, binary, prefix, and an existing secret.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] User: prepare the pinned image and store the public-read-only token
        for the main preview. Done 6 November 2026.
        Branch: `calummoore/ci-proxy-load-balancer`.
- [ ] Serve the site, the dashboard, and signed image URLs from a Cloud
      Storage bucket, through Cloud CDN and one HTTPS load balancer. My call:
      they are static files, so they need no runtime.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define public buckets, private images, CDN, routes, and publishing.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] User: deploy the edge and register a CDN signing key outside Terraform,
        for the main preview. Done 6 November 2026: key
        `ciproxy-main-preview-images-v1`, HTTPS checked, uptime checks on.
        Branch: `calummoore/ci-proxy-load-balancer`.
- [ ] Create the DNS zone, which the user delegates.
      Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [x] Define production and staging DNS zones and HTTPS records.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
  - [ ] User: apply the zones and delegate the domain and staging subzone.
        Branch: `conductor/ciproxy-mvp-milestone-5-deployment`.
- [x] Deploy the main preview on each push, and production on each merge
      into `main` after the main preview run of the same commit passes, as
      item 58 says. Add uptime alerts.
      Branch: `codex/m5-terraform-promotion`.
- [x] Plan, check, and promote every root from CI, as item 56 says: a
      read-only plan identity, a policy that needs an approval file for each
      destructive change, an apply identity for one reusable workflow, and
      deletion protection for the database and the secrets.
      Foundation creation now runs in CI after one-time seed and bootstrap.
      Design calls, 5 November 2026: operator bootstrap initializes every
      remote backend through shared root, directory, bucket, and prefix
      metadata. Reason: first initialization writes state, which the plan
      identity cannot do. Seed `main.tf` owns trust API enablement. Reason:
      both operator and CI finish seed before any bootstrap resource starts.
      No state move is needed because nothing has been applied in GCP.
      Branch: `codex/m5-terraform-promotion`.
- [x] Move both environments into the GCP project `ciproxy`, as item 68
      says: one shared network, cluster, and image registry, with a
      namespace for each environment, and separate databases, KMS keys,
      secrets, buckets, DNS zones, and edges. Scope the CI identities of
      each environment to its own resources. Apply the shared parts from
      the main preview until launch, and only from `main` after it.
      Branch: `conductor/ciproxy-mvp-milestone-5-shared-project`.
- [x] User: move `Firna-AI` to GitHub Team, as item 57 says. Orchestrator:
      add the ruleset on `main` that needs a pull request and the gate.
      Done on 5 November 2026: ruleset `main-pull-request-and-gate`, ID
      `26601138`, active, with deletion, non-fast-forward, pull request, and
      required status check rules.
      Branch: `codex/m5-terraform-promotion`.
  - [x] Prepare and test the idempotent `deploy/scripts/github-ruleset.sh`.
        Branch: `codex/m5-terraform-promotion`.
  - [x] User: upgrade the organization. Orchestrator: apply the ruleset.
        Branch: `codex/m5-terraform-promotion`.
- [x] Make this repository public, as item 70 says. The orchestrator scans
      the history, the run logs, and the pull request texts for secrets, and
      checks the workflows and the runner groups. The user changes the
      visibility. Then turn on secret scanning with push protection, and
      require approval of workflow runs from all outside contributors.
      Done on 5 November 2026: the scans found no secret, the user made the
      repository public, and an admin read confirmed secret scanning, push
      protection, and approval for all outside contributors.
- [x] Rename the staging environment to the main preview of item 53, with
      its hosts under `br-main.preview`, before the first apply.
      Branch: `conductor/ciproxy-mvp-milestone-5-main-preview`.
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
- [x] Build and publish the control-plane image in the deploy workflow, as
      item 90 = G says. A build job builds it on the runner with no cloud
      credentials. A publish job pushes it by digest with workload
      identity. The preview deploys that digest, and production takes over
      the same digest from the preview's saved inputs. A push that does not
      change the control-plane sources reuses the image of their hash.
      Branch: `conductor/item-90-g-control-plane-image-in-the-deploy`.

## Milestone 6: GitHub App and scale sets

Depends on: 4

`ciproxy-github` acts as the App, with one scale set and one listener for
each label. The scale set spike confirms it.

- [x] Add the App tokens, the repositories, the default branch, and the
      installation events. Create the default labels of `labels.md`.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Handle a removed App, a suspended App, and a personal account, as
      item 29 says.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Use the crate `scaleset`, with one session for each label under the
      lock of Milestone 4.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Limit the repositories with one runner group, because GitHub assigns
      jobs with no acquire step. Check that a Free organization can do it,
      or propose the requirement change.
      The selected private group is implemented and tested over HTTP.
      GitHub Free enforces the repository limit. The requirement does not
      change. On 5 November 2026, a live probe ran in the Free organization
      `Firna-AI`. The job of the allowed repository ran on the runner of the
      probe. The job of the refused repository stayed queued with no runner
      for 101.8 seconds after the allowed job succeeded. The scale set got
      messages only for the allowed run: one `JobAssigned`, one
      `JobStarted`, and one `JobCompleted`. Every cleanup check of the probe
      passed. The default branches of both repositories stayed the same.
      The probe now waits until GitHub reports the allowed job as completed,
      with a conclusion. GitHub can report a successful run before its job
      has a conclusion. The passing live run used a local copy with this
      wait. An offline regression covers the race.
      Evidence:
      `.context/ciproxy-mvp/milestone-6/runner-group-proof/live-evidence.md`.
      Branch: `conductor/ciproxy-mvp-milestone-6-runner-group-proof`.
- [x] Store each message before you delete it, by job ID. Count the wait
      from `scaleSetAssignTime`. My call: `queueTime` is empty, and the
      spike found this time within about a second of the queue time.
      My call, 4 November 2026: keep UUID demand separate from numeric
      Actions job history. Reason: GitHub sends UUID job IDs and zero
      request IDs. A first completion-observation time bounds retention
      when GitHub omits the finish time. Replay keeps that observation.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Read the ref, the event, and the head repository of each run with
      the "Actions" read permission, and forward them to the agent, as item
      32 says.
      Closed 7 November 2026: the user answered items 54 = E and 55 = A on
      5 November 2026. The full ref now comes from the checked runtime token
      in the machine cache, which Milestone 11 built, and an unknown ref
      fails closed. The agent gets the event and the head repository in
      `JobFacts` from `crates/ciproxy-github/src/assignments.rs`. The text
      below is the history of the API research.
      Raw API metadata is implemented. Full-ref forwarding stays blocked.
      Orchestrator: run `ref_metadata_probe.py` and `push_ref_probe.py`
      from `crates/ciproxy-github/scripts/`. Inspect supplied PR and
      reusable run IDs with `run_job_ref_probe.py` from that directory.
      Run `workflow_ref_probe.py` there to inspect scale-message workflow
      refs for all five cases with the approved existing fixtures.
      The orchestrator verified all five cases with cleanup. A top-level
      workflow ref distinguishes branch and tag pushes. A reusable
      workflow ref names the called file. The crate README gives the
      environment and arguments. No ref is guessed on this branch.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Link each installation to the workspace of the Admin who started it,
      by the `requester` of the `installation` event, because GitHub sends
      an approving owner to the setup URL without the state of ciproxy.
      Keep one scale set for each organization and label.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Create and remove the just-in-time registrations.
      Evidence: `.context/ciproxy-mvp/milestone-6/verification.md`.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Serialize organization policy operations across control-plane
      processes, with state and repository reads inside the lease.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Preserve current installation identity and App state when delayed
      lifecycle deliveries arrive. Keep the authenticated requester.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Reuse durable API facts for known runs when repository access ends.
      First-read unknown handling still waits for owner item 55.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Save current remote scale-set mappings without replacing internal
      identities. Restart stale listeners and keep owned groups by ID.
      Use a stable organization key across workspace registrations.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Persist the verified organization login and App state together.
      Restart a listener whose saved organization login changed.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.
- [x] Save lifecycle notifications atomically with their state changes.
      Replay them in order and confirm each only after consumer acceptance.
      Cover failed replacement cleanup, refresh, and suspension delivery.
      User-approved review fix: option B, 4 November 2026.
      Branch:
      `conductor/ciproxy-mvp-milestone-6-github-app-and-scale-sets`.

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

- [x] Issue tokens for one hour and any number of machines, which an Admin
      can revoke. Register machines, and accept the agent WebSockets.
      My call, 4 November 2026: hash independent 256-bit secrets with SHA-256.
      Reason: the store keeps only hashes, and each credential stays private.
      Branch: `conductor/ciproxy-mvp-milestone-8-machine-gateway`.
- [x] Store the reports, mark machines offline after 60 seconds, send the
      commands, and revoke a removed machine at once.
      My call, 4 November 2026: keep full typed reports in JSONB, apart from
      desired settings, and fence them by connection ID. Reason: an old
      report cannot undo an Admin action or replace a new connection.
      Branch: `conductor/ciproxy-mvp-milestone-8-machine-gateway`.
- [x] Accept this protocol version and the one before. My call: agents
      update themselves.
      Branch: `conductor/ciproxy-mvp-milestone-8-machine-gateway`.
- [x] Test the gateway with a fake agent over a real WebSocket.
      Branch: `conductor/ciproxy-mvp-milestone-8-machine-gateway`.

## Milestone 9: Agent

Depends on: 3

`ciproxy-agent` runs as root under systemd. It starts each runner in a fresh
Firecracker VM, as `agent.md` says, which the Firecracker spike confirms. It
ships through `futex-ai/cli-release`.

Orchestrator call, 5 November 2026: Milestone 13 owns the cache handoff and
the Docker disk attachment of each job. Reason: that work crosses the control
plane, the agent, the guest, the runner patch, and the sidecar.

Implementation calls, 5 November 2026: a provisional image budget of
4 SI GB, persisted on first install, leaves system space on minimum hosts.
Measured XFS costs plus conservative metadata headroom stay in the host
reserve. The image measurement sets the real budget later.

- [x] Add `install --token` and `uninstall`. My call: "Remove a machine" needs
      `uninstall`.
      Registration now saves the workspace returned by the gateway.
      Host preparation follows the approved XFS-file approach below.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Report the hardware and the slots every 10 seconds. Reconnect after a
      growing wait, and apply "Drain and remove".
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Run each VM under the jailer in its own systemd unit, on a reflink
      copy of the image, with the internet and the machine cache only. My
      call: an agent restart then keeps the VMs.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Pass registrations over vsock only. Give each VM an empty drive for a
      Docker disk.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Fetch and check the newest image. Update the agent with `cli-updater`
      every hour.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Release with release-plz and `scripts/cli-release.sh package`, as
      `x86_64-unknown-linux-gnu` for old glibc versions.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Keep install tokens and machine credentials out of WebSocket logs at
      every verbosity. Test real transport traffic with TRACE enabled.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Test with fakes.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Prepare the host in `install`, as item 62 says: Firecracker, the
      jailer, the host tools, and a reflink disk area in one XFS file on any
      file system.
      Orchestrator call, 5 November 2026: always use the XFS file, also on
      a file system with reflinks. Reason: every machine uses one path.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Size the area as the job disk, an image budget, and the XFS overhead
      measured after formatting. Take the budget and the overhead from the
      host reserve quarter, so that the slots, the job disk, and the
      hardware minimum stay the same. Enforce the budget on each image
      fetch and on the kept bases. Drop unused older bases before the
      images exceed the budget. Never take space from the job disk. Make
      the budget a setting with a safe provisional default. The image
      measurement of Milestone 10 sets its real value later. Add a
      regression test on a disposable file system.
      Orchestrator call, 5 November 2026: the finding that the area leaves
      no room for images is fix now, apply A. It is not a user decision,
      because the three-quarter area was an orchestrator call in the brief
      of item 62, not the user's answer. A keeps every requirement: "Disk"
      in `job-routing.md` already keeps the images in the host reserve, and
      reflinks need the bases in the file system of the VM disks.
      Left out: C, images in the job disk, weakens the job-disk promise. B,
      a smaller job disk, changes the job capacity and the minimum machine.
      E, read-only bases with a snapshot in each guest, replaces the
      approved reflink roots and needs guest, image, and KVM work.
      F, an XFS project quota, adds a host requirement that only the user
      can approve. It waits in the
      [quota item](follow-ups/agent-xfs-image-project-quota.md), and A does
      not wait for it.
      Branch: `conductor/ciproxy-mvp-milestone-9-agent`.
- [x] Apply item 73 = A to the label disks, image budget, machine minimum,
      storage split, protocol docs, READMEs, and site calculations. Keep the
      image budget apart from the job disk, and test the 250 GB minimum.
      Apply item 75 = A to the AX42 calculation and setup docs. Keep the
      site and mockup wording and six-job count aligned.
      Orchestrator call, 5 November 2026: use one resource calculation with
      an explicit image budget. Pass the recorded budget to agent
      admission and cache sizing. Keep raw hardware capacity and the wire
      schema. The scaler still receives its job-disk budget from its
      caller. Reason: the override then uses the same disk split without
      adding a report field or new scheduler integration.
      Mockup calls, 5 November 2026: give the mini PC 512 GB so that it
      still shows the disk limit. The mixed-machine dialog edits an
      unsaved 100 GB draft of `ciproxy-4vcpu` to retain its job-count range.
      The saved default remains 80 GB.
      Branch: `conductor/ciproxy-mvp-73a-disk-sizes`.
- [x] Test the five existing root-preparation failures and failed-start
      cleanup for PR #25 review finding 1, option A. Check returned errors,
      cleanup requests, and released disk and slots at both boundaries.
      Branch: `conductor/ciproxy-mvp-73a-disk-sizes`.
- [x] Align the released image bundle, vsock port, and guest state,
      reconnect, and shutdown behavior. Use the selected image's exact
      root size and the original free-disk request under item 77 = D.
      Reflink the image and grow each fresh ext4 root before boot.
      Keep sticky Docker disks under the checked-scope cache integration.
      Apply the approved removal rule of item 79 = A and the Docker space
      rule of item 80 = D.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
- [ ] Smoke-test a Docker job, a warm runner, and a restart during a job.
      Orchestrator: run `crates/ciproxy-agent/scripts/smoke-docker.sh`,
      `crates/ciproxy-agent/scripts/smoke-warm.sh`, and
      `crates/ciproxy-agent/scripts/smoke-restart.sh` on the KVM test host
      with the staging gateway, image, guest, and GitHub App.
      Orchestrator: run `crates/ciproxy-agent/scripts/smoke-host.sh` on a
      fresh disposable KVM host with `/var/lib` on ext4. It checks the first
      install, a repair, two reflink jobs, and uninstall.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.

## Milestone 10: Job VM image

Depends on: 3

One image serves the job VMs and Boat, as "The image" in `agent.md` says.
The Firecracker spike confirms it for VMs. The Boat spike ran the runner
from a minimal snapshot, not from the full image.

Orchestrator design call, 5 November 2026: use the build machine's CPU in
the QEMU build VM. Reason: the build machine is ours, and no requirement
fixes that VM's CPU model. The software set needs features such as SSSE3
and AVX at run time. This matches the host CPU exposure of Firecracker job
VMs. A fixed model would need more trial builds for no current requirement.

Orchestrator design call, 5 November 2026: use Virtio-SCSI for the QEMU
build root. Reason: the unchanged upstream read-ahead check needs a real
SCSI or NVMe disk. This keeps virtio transport and changes only the build
VM hardware. The exported Firecracker disk stays ext4 on `/dev/vda`.

Item 77 sizing call, 5 November 2026: add 16 MiB to the label's free bytes,
divide by 0.97, and round up. Add the image's logical root bytes and round
up to a 4096-byte block. This covers the ext4 kernel reserve pool, the 1%
root reserve, and new metadata. The existing image already includes its
filesystem costs. This keeps a default job within the 82.5 GB job budget
of a 250 GB machine. The real growth test found that the kernel pool also
reduces the free bytes that `df` shows to the runner.

Item 77 image-size call, 5 November 2026: the user supplies
61,000,000,000 bytes until Milestone 10 reports `root_disk_bytes`.
One agent constant holds that default. The control-plane configuration
can override it. The release manifest will replace that configuration
source. The configured logical length must match the selected release.

Measured image call, 6 November 2026: the user supplies the final logical
root size, 60,263,759,872 bytes, from PR 28 at `b0c2ba7`. This replaces
the provisional value in the source default. The root rule stays the same.
A 20 GB default request reserves 80,899,612,672 bytes and leaves
1,600,387,328 bytes on the 82.5 GB minimum-machine job budget.
PR 28 merges before PR 27. Historical image-size decisions stay unchanged.

PR 27 review call, 5 November 2026: the user applies finding 1 option A.
Each release keeps its logical root size with its image identity. The
agent retains those facts through the image hold, sizes the root from
that held image, and keeps the original free-disk request for retries.
The scheduler uses each machine's selected image facts. The configured
size is only its fallback before a machine reports its image.

The user applies finding 2 option B: the three open image and storage
follow-ups keep the free-disk meaning and link to the canonical protocol.
The image README stays with PR 28. The revised measured image size stays
with that workspace until its export and full-image checks pass.

PR 27 format call, 5 November 2026: the user selects the new report, index,
and VM record formats without a compatibility layer. No production state
exists. Keep the existing unreadable-state handling. Reset old smoke-test
state on the KVM host. This supersedes the earlier compatibility request.

Runner storage call, 5 November 2026: Milestone 13 owns the exact-byte
actual runner allocation and its forward migration. Label and free-disk
request store types stay in GB. PR 27 changes no runner-store schema and
adds no migration. Its protocol report keeps original requested resources
separate from actual resources. PR 27 merges first, then Milestone 13
merges the primary branch and uses those types.

Item 77 mockup calls, 5 November 2026: separate label free disk from Boat
total disk in the fixture types. Count whole VM bytes before displaying GB.
Display whole GB to match the other disk values. Give the Boat-only sample
label 120 GB of free disk, so its VM still fits Boat. Give the mixed-machine
draft 40 GB of free disk, so its two-to-three-job range stays. Keep one
mirror of the agent rule in the mockups. The site keeps one computed
default VM disk value from that rule.

Replaced ways for item 77:

- A label disk means free job bytes instead of total VM bytes. Move the
  defaults, label text, tables, and dialogs to that meaning.
- The scheduler counts the full image and growth allocation instead of
  raw label bytes. Move own placement, held jobs, warm runners, Boat fit,
  and dispatch checks to the one sizing rule.
- Remove the resolved smaller-label follow-up after its work passes.

- [x] Normalize the generic Ubuntu APT sources during QEMU bootstrap for
      the existing upstream mirror-list check. Keep the pinned base, full
      software, tests, and software report. Check the generated bootstrap
      offline and verify the upstream APT checks on KVM.
      Branch: `conductor/ciproxy-mvp-milestone-10-apt-sources`.
- [x] Expose the build host CPU to QEMU and repeat the full image build
      without changing the pinned base, software, or upstream checks.
      Branch: `conductor/ciproxy-mvp-milestone-10-apt-sources`.
- [x] Check upstream hardware assumptions and use Virtio-SCSI for the build
      root. Verify boot before repeating the full KVM build.
      Branch: `conductor/ciproxy-mvp-milestone-10-apt-sources`.
- [x] Measure the full image first. If it fits no size, propose a smaller
      image as a decision.
      Run `bash image/build.sh .context/ciproxy-image-build` on the KVM
      host. Read `artifacts/measurement.json` in that build directory.
      The full software uses 55.22 GB apparent and 58.29 GB allocated.
      Item 73 = A keeps that software, selects an 80 GB label disk with
      1% root reserve, and gives the agent a 140 GB image budget.
      The approved 80 GB disk booted under Firecracker and reported
      19,494,903,808 free bytes to runner UID 1001 with the release kernel.
      Evidence: `.context/ciproxy-mvp/kvm/apt-sources-status.json`.
      Branch: `conductor/ciproxy-mvp-milestone-10-apt-sources`.
- [x] Finish the 80 GB root from the retained full userspace with 1% root
      reserve. Boot it, record job free bytes, package both release
      artifacts, and verify their checksums.
      Branch: `conductor/ciproxy-mvp-milestone-10-apt-sources`.
- [x] Define it under `image/` from a pinned `actions/runner-images`, with
      Docker, the runner, and a kernel.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.
- [x] Add `ciproxy-guest`: it takes the registration over vsock, starts the
      runner, and powers off the VM after the runner exits. On Boat, it
      tells the control plane when the runner exits, as item 47 says.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.
- [x] Export the image disk at its smallest size for item 77 = D. Measure
      `root_disk_bytes` for the image release manifest.
      Branch: `conductor/ciproxy-mvp-milestone-10-root-size`.
  - [x] Image: compact ext4, retain its 1% root reserve and a small free
        margin, and record `root_disk_bytes` in measurement and manifest.
        Export the retained userspace and prove at least 20 GB free after
        growth by 20 GB in a booted runner-user probe.
        Keep the release free of writable mounts after creation. Check
        plain `resize2fs` growth with e2fsprogs 1.46.5 without a prior repair.
        Implementor call, 5 November 2026: keep 2 GiB free after compaction.
        It covers the reserve and metadata growth of the default 20 GB
        and 60 GB job allowances. Round growth to complete ext4 groups.
        Evidence: `.context/ciproxy-mvp/kvm/item-77/status.json`.
        Branch: `conductor/ciproxy-mvp-milestone-10-root-size`.
  - [x] Image fix round 1: keep the base root small enough for a default
        job on the 250 GB minimum with at least 1 GB of job-disk space left.
        Use the real `RootSizing` helper of PR #27 for 1, 20, 60, and 500 GB
        allowances. Boot separate grown copies and verify each free value.
        Implementor call, 5 November 2026: use a 64 MiB base margin with
        measured empty ext4 metadata. It leaves 48 MiB beyond the kernel
        reserve pool. The agent helper supplies all job-growth headroom.
        Evidence: `.context/ciproxy-mvp/kvm/item-77/fix-round-1/status.json`.
        Branch: `conductor/ciproxy-mvp-milestone-10-root-size`.
- [x] Apply item 77 = D: size each job VM disk as the image plus the free
      disk of its label, with filesystem allowance,
      on machines and on Boat. Update the label table, the protocol docs,
      the site, and the mockups.
      Branch: `conductor/ciproxy-mvp-77d-free-disk`.
      Evidence: `.context/ciproxy-mvp-77d-free-disk/reports/`.
- [x] Fix PR 27 review findings 1 and 2: keep per-release root facts through
      image holds, size and reserve each actual root in the agent, keep
      original requests for retries, and schedule from each machine's
      selected image. Correct the three open image and storage follow-ups.
      Test mixed releases, publication between report and start, and retry
      after publication and restart. Run new async tests ten times.
      Branch: `conductor/ciproxy-mvp-77d-free-disk`.
      Evidence: `.context/ciproxy-mvp-77d-free-disk/fix-round-1/`.
- [x] Fix PR 27 re-review finding 3: check retained Boat runners against
      their saved required root allocation. Keep the current image floor
      for new starts. Preserve the current Wait and largest-size policies
      without applying a new image floor to an existing older VM.
      Branch: `conductor/ciproxy-mvp-77d-free-disk`.
      Evidence: `.context/ciproxy-mvp-77d-free-disk/fix-round-2/`.
- [x] Apply the final measured image root of 60,263,759,872 bytes to the
      default, current fixtures, tests, examples, site, and mockups.
      Fix owner review finding 4 with B: reconcile current warm targets on
      connected machines before replenishment checks new-image capacity.
      Test count zero and changed organization allocation after image
      growth. Run every new or changed scheduler test ten times.
      Branch: `conductor/ciproxy-mvp-77d-free-disk`.
      Evidence: `.context/ciproxy-mvp-77d-free-disk/fix-round-3/`.
- [x] Publish it with its SHA-256, and as a shared Boat snapshot. Rebuild
      before GitHub stops accepting the runner.
      Done 7 November 2026: release `2026-11-06-7484d1b` is on the CDN with
      its checksums, and the shared Boat snapshot has the same name. Its
      pinned runner 2.338.2 was the latest release that day. The weekly
      check of `image/check-runner.sh` stays an operations task.
      Orchestrator: run `image/publish.sh` and the create, prepare, and
      snapshot commands of `image/boat.py` from the
      [image guide](../image/README.md#boat-shared-snapshot).
      Use `image/check-runner.sh` and `image/github.py runner-support`
      to plan the next rebuild before runner support ends.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.
- [ ] Run a job from the shared snapshot on Boat `large`, and measure the
      free disk. Item 77 = D selects 20 GB of free disk for the default
      label. Boat `large` provides 111 GB and fits the image plus that
      free disk, with filesystem allowance. The full
      image does not fit the 51 GB of `small` or `default`.
      Orchestrator: run the registration and result commands of
      `image/github.py`, and the create, run, status, and delete commands
      of `image/boat.py` on `large`. The
      [smoke guide](../image/README.md#boat-smoke) gives
      the arguments. Supply the overflow callback and its scoped token.
      Compare `image/boat.py metadata` before and after snapshot restore.
      Record the measured free bytes against the approved label disk in
      the release evidence.
      Branch: `conductor/ciproxy-mvp-milestone-10-job-vm-image`.

## Milestone 11: Cache sidecar and runner patch

Depends on: 3

A patched runner sends the cache calls to a sidecar in the VM, which reads
and writes through to GitHub. The machine cache keeps entries after GitHub
evicts them. Boat runs the stock runner. The cache spike confirms it.

- [x] Rename `ACTIONS_CACHE_URL`, `ACTIONS_CACHE_SERVICE_V2`, and
      `ACTIONS_RESULTS_URL` in `Runner.Worker.dll` at equal length.
      Branch: `conductor/ciproxy-mvp-milestone-11-cache-sidecar`.
- [x] Add `ciproxy-cache-sidecar`. Pass other results calls through, and
      fall back to GitHub on any error.
      Branch: `conductor/ciproxy-mvp-milestone-11-cache-sidecar`.
- [x] Scope each entry from the job facts, as "Cache scopes" says.
      Branch: `conductor/ciproxy-mvp-milestone-11-cache-sidecar`.
- [x] Add "Clear cache" for one machine, eviction, and the reports.
      Branch: `conductor/ciproxy-mvp-milestone-11-cache-sidecar`.
- [x] Test the sidecar against a fake GitHub cache.
      Branch: `conductor/ciproxy-mvp-milestone-11-cache-sidecar`.
      Orchestrator: run `image/runner-patch/smoke/live-check.py` with
      trusted registration and runtime credentials on a GitHub runner.
- [x] Let only the 7 trusted events of GitHub write shared entries, and
      write item 59 into "Trusted write-back" of `cache.md`.
      Evidence: `.context/ciproxy-mvp/trusted-cache-events/acceptance.md`.
      Branch: `conductor/ciproxy-mvp-milestone-11-trusted-cache-events`.
- [x] Orchestrator: run the three live checks of item 54 = E: a push to the
      default branch, `pull_request_target`, and a job on a ciproxy runner.
      All passed on 5 November 2026. A push and a dispatch on `main` got
      `refs/heads/main` with write access, `pull_request_target` got
      `refs/heads/main` read only, and on a ciproxy runner the `run_id`,
      `job_id`, and `runner_id` of the token matched the scale set message
      and the registration. Evidence: `.context/job-ref-research/report.md`.
- [x] Check the runtime token of each job in the machine cache, as item 54
      says, and take the full ref from its write scope. Write the rule into
      `cache.md` and `security.md`. A job whose token fails the check fails
      closed, as item 55 = A says. It waits for the live checks.
      Branch: `conductor/ciproxy-mvp-milestone-11-signed-job-ref`.
      Evidence: `.context/ciproxy-mvp/signed-job-ref/implementor-report.md`.

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

- [x] In `futex-ai/sandbox`, write the plan of the change, as its
      `AGENTS.md` says.
- [x] In `futex-ai/sandbox`, split `SandboxBackend` into capability traits:
      the lifecycle, snapshots, image preparation, processes, files,
      terminals, ports, and screens. Move `sandbox-e2b`, the mocks, and the
      conformance harness to them. Record the adoption steps for Juno.
- [x] In `futex-ai/sandbox`, add a VM size request, the sizes that an
      account can start with their prices, a start refusal with a retry
      time, and one-shot lifetimes up to the limit of each provider.
- [x] In `futex-ai/sandbox`, add `sandbox-boat` with the lifecycle,
      snapshots, processes, and files. Use marks, `noEnv`, `ttlSeconds`,
      and an `Idempotency-Key`. Join `machineTypes` of `GET /limits` with
      the published sizes and prices, because the API gives no dimensions.
      Map HTTP 429 `rate_limited` to a refusal with its `Retry-After`, 403
      `machine_class_plan_required` to a size that the plan lacks, and 402
      `billing_required` to a billing refusal. Test it against a fake Boat
      server and the conformance harness.
      Merged in `futex-ai/sandbox` #11 as `9658361`, for the four TODOs
      above.
- [x] Add `ciproxy-overflow` on `sandbox-interface`, with the Boat adapter
      of the reviewed revision. Read the sizes of each account every 5
      minutes. My call: a Boat plan can change at any time.
      My call, 5 November 2026: serialize each workspace with a store
      session. Reason: deliveries and callbacks need one durable order.
      Branch: `conductor/ciproxy-mvp-milestone-12-overflow-to-boat`.
- [x] Pick the cheapest size that fits, or the largest.
      Branch: `conductor/ciproxy-mvp-milestone-12-overflow-to-boat`.
- [x] Start from the shared snapshot with marks, a private registration
      file, a token for the end report, a lifetime of 6 hours, or 2 on a
      free trial, and a delivery key for each start.
      My call, 5 November 2026: use the adapter's trusted controller for
      the fixed image entrypoint. Reason: the image needs root for Docker
      and its private namespaces. Persist file delivery before writing.
      Branch: `conductor/ciproxy-mvp-milestone-12-overflow-to-boat`.
- [x] Delete a sandbox at once when its guest reports the end of its
      runner. Accept one report for each token.
      Branch: `conductor/ciproxy-mvp-milestone-12-overflow-to-boat`.
- [x] Delete idle marked sandboxes. Wait out a refusal with a retry time,
      drop a size that the plan lacks, and stop Boat starts at a billing
      refusal. Treat `canStart` as a hint, as the spike saw a refusal after
      `canStart: true`. Count the spend as run time and start charges. My
      call: Boat bills these two parts.
      Item 67 = B, approved 5 November 2026: count confirmed Boat costs and
      the pending full-lifetime bound of each unconfirmed start. Use the
      requested size's published hourly price for 6 hours, or 2 on a trial,
      plus the saved start-fee quote. Permit starts while the limit covers
      existing costs, bounds, and the prospective start. Replace a bound
      with real costs or settle it after its lifetime without proof of
      longer running. Show confirmed spend and pending amounts separately.
      A cost without a safe bound blocks new starts until it resolves.
      Evidence:
      `.context/ciproxy-mvp/milestone-12/item-67/implementor-report.md`.
      Branch: `conductor/ciproxy-mvp-milestone-12-overflow-to-boat`.
- [x] Encrypt the keys with Cloud KMS. Test with the mock of
      `sandbox-interface`.
      My call, 5 November 2026: bind ciphertext to the workspace, record,
      and secret kind. Reason: a secret must open only in its own context.
      Evidence: `.context/ciproxy-mvp/milestone-12/implementor-report.md`.
      Branch: `conductor/ciproxy-mvp-milestone-12-overflow-to-boat`.

## Milestone 13: Own machines end to end

Depends on: 6, 7, 8, 9, 10, 11

This milestone builds "A job on an own machine" of `architecture.md`, on a
test host in `internal-498318`.

Code and local verification evidence: `.context/ciproxy-mvp/`.
Live KVM and authenticated job checks remain with the orchestrator.

- [x] Wire the control plane and the agent. Add the patch and the sidecar to
      the image.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
- [x] Register each job with the machine cache. Add the missing guest
      handoff: the original runtime token, the cache URLs, the effective
      cache permissions, and a readiness pause. The
      [integration record](follow-ups/agent-cache-area-integration.md)
      keeps the remaining live verification.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
- [x] Attach each Docker disk by its checked scope, as items 54, 55, and 66
      say. Reserve its data plus label free disk under item 80 = D.
      Use only spare disk that no set of job roots can use. If no
      reservation fits, keep the job on its root without a Docker disk.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
- [x] Stop a warm runner only after GitHub removed it. My call: a runner
      that just took a job keeps it. Item 79 = A keeps this rule for
      ordinary stops and Drain. Remove, uninstall, and every App removal
      stop warm runners at once. A job assigned before its guest handover
      can fail. Jobs that the guest received still finish.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
- [x] Count each trusted taken assignment through metering, without a
      numeric job-ID guess. Preserve prior starts across new assignments
      of the same UUID. Keep a terminal job without a trusted finish out
      of minutes, and retry the GitHub job lookup with backoff. Preserve
      exact actual allocations and trusted timestamp precision.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
- [ ] Smoke-test warm starts, slots, drain, remove, offline, a cache hit, a
      restart, and a disallowed repository.
      Orchestrator: use the credential-file scripts on the KVM host with
      the main preview control plane. Keep live evidence separate from
      the code checks.
      Include an agent restart before the guest receives its registration.
      The current restart script covers a running job. Add the interrupted
      startup case before claiming its live proof.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.

Owner review of pull request #31, round 1, 6 November 2026: the user chose
fix now for findings 1 to 8 and fix after the merge for finding 9.

- [x] Repair findings 1 to 8 with the user's options. 1A saves the original
      cache rights and URL origins of each job. 2A loads the listener rules
      before each cache-service start. 3B releases capacity only on a full
      report received after the first confirmation. 4B resends saved stops.
      5B makes the lease the only Docker record. 6B keeps the App removal
      cause. 7B shares one reflink copy. 8A gives accounting its own worker.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
- [x] Build item 85 = F with item 86 = B. The runner hook waits for the host
      decision, with no fallback and no time limit. A reconnect before the
      first handoff means only GitHub's cache for the whole job.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`. Code: `cf622fc`,
      merged in `fc5b46c`. Its live KVM proof stays open in the
      [integration record](follow-ups/agent-cache-area-integration.md).

Finding 9 = A waits until after the merge, because production uses the new
source hook, and cleanup reaches unchanged operator scripts. The
[runner patch removal](follow-ups/runner-string-patch-removal.md) holds that
work. The smoke TODO above and the
[integration record](follow-ups/agent-cache-area-integration.md) keep the
live KVM proof of these fixes.

Owner review of pull request #31, round 2, 6 November 2026:

- [x] Repair finding 12 with the approved B and the scoped acceptance order
      of D. Each ordinary stop needs positive GitHub retirement proof. A
      take-out accepts only that proof, busy answers, or saved stops.
      Code: `82aba82`, merged in `62de88e`.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.

Owner review of pull request #31, round 3, 6 November 2026: the user
authorized a third and final fix round as item 87 = A, and approved finding
13 = B after its fresh check. Later findings become follow-up items, with
no fourth fix round.

- [x] Repair finding 13 with B. Each accepted assignment gets an explicit
      hosted preparation result: trusted facts, or GitHub-only when the
      loaded App snapshot lacks the repository. Snapshot and transport
      errors retry. The agent saves a GitHub-only decision before it
      releases a waiting Worker, and that decision is final.
      Code: `118ddd9`.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.

## Milestone 29: Mockups: organization linking

Tags: mockup

Depends on: none

Items 82 and 83 change how an organization joins a workspace after "Add an
organization". These screens come before the requirements and the backend
work of Milestone 14. The merged mockup catalogue and its screens are the
base of this milestone.

- [x] Show the return from GitHub: the link at once for one Admin
      workspace, the picker for several, a failed install check, and the
      page for no Admin workspace.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Show a request that waits for an owner, and the choice of the person
      who asked at the next sign-in after a late approval.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Show an inactive organization with the plan reason, with both
      reasons, and with the Admin reason only. Show the Turn on dialog, the
      organizations after Team, and the Plans page of that December.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Show "Add an organization" on Free, and keep the soft-limit banner of
      a Free workspace that kept its organizations.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Connect the screens through the updated add flow and four new flows:
      a workspace choice, a late approval, an add on Free, and a Turn on.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Run `npm --prefix ts run mockups:check`. Smoke-test the changed pages
      at 1440, 720, and 390 px, and the changed flows.
      Branch: `conductor/ciproxy-add-organization`.
      Evidence: `.context/ciproxy-add-organization/mockups/`.
- [x] Update the design and the TODOs of this plan for each problem that
      the mockups show.
      Branch: `conductor/ciproxy-add-organization`.

## Milestone 30: Documentation: organization linking

Depends on: 29

This milestone writes the answers to items 82 and 83 into the protocol
docs, with their decisions and rejected options.

Orchestrator calls, 6 November 2026: a request return chooses no
workspace, and the person chooses only after the check. A lost Admin role
makes only a chosen workspace inactive. Every linked organization counts
against the tier, except a removed one, and an active organization stays
active. A link gives no person a role. Reason: these calls apply items 82
and 83 with no new rule.

- [x] Write "Choose the workspace", "A request that an owner approves
      later", and "Inactive organizations" into
      `docs/protocol/workspaces.md`. Keep the organization count of the
      tiers.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Align the 30-day reinstall in `app-removal.md`, with the kept hold,
      and the soft limits in `metering.md` with these rules.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Run `cargo xtask check --include markdown`.
      Branch: `conductor/ciproxy-add-organization`.

## Milestone 14: Dashboard API: sign-in, setup, machines, and settings

Depends on: 4, 6, 8, 30

`ciproxy-api` serves the dashboard with axum, and `utoipa` with
`openapi-typescript` generates its types. My call: money in cents, times in
UTC RFC 3339, and IDs as strings. Reason: JavaScript loses 64-bit numbers.

- [x] Name the generator and the wire forms in `AGENTS.md`. Add a gate phase
      that regenerates the types and fails on a diff, as item 45 says.
      Branch: `conductor/ciproxy-mvp-milestone-14-api`.
      Implementation call, 5 November 2026: encode integer cents as decimal
      strings, so JavaScript keeps their precision.
- [x] Sign in with the GitHub App. Apply "Roles" in `security.md` for each
      workspace, with the access requests.
      Branch: `conductor/ciproxy-mvp-milestone-14-api`.
      Implementation call, 5 November 2026: a sealed browser cookie keeps
      sign-in metadata without a GitHub grant. The store keeps a person and
      grant only while a role or access request exists.
      App registration uses `/api/auth/callback` for sign-in and `/` for
      setup. Keep user authorization separate from installation for PKCE.
- [x] Apply the owner's App registration override of 6 November 2026:
      `/api/auth/github/callback` for sign-in and `/api/github/setup` for
      the trusted browser return. Verify installation access on the server,
      then redirect to the dashboard. Setup never chooses the workspace.
      Keep no old aliases or dashboard-only forwarding flow. OAuth during
      installation and redirect on update stay off. Expiring user tokens
      stay on. Use automatic installation events only. Milestone 13 owns
      `/hooks/github` and periodic repository refresh. Branch:
      `conductor/ciproxy-mvp-milestone-14-api`.
- [x] Add an organization to a workspace, take one out, and keep the
      repository access of each one. A take-out deletes the scale sets and
      removes the install through the GitHub API, as item 51 says. ciproxy
      tells its own removal apart from a removal by an owner.
      Branch: `conductor/ciproxy-add-organization`.
      Items 82 and 83 chose the add rules. Milestones 29 and 30 come first.
  - [x] Serve per-organization repository access through the GitHub App.
        Branch: `conductor/ciproxy-mvp-milestone-14-api`.
  - [x] Take out an organization with durable, installation-specific
        cleanup. Keep its history and distinguish owner removal.
        Branch: `conductor/ciproxy-mvp-milestone-14-api`.
  - [x] Link an organization after "Add an organization", as
        `workspaces.md` says. The install link carries a `state` for the
        add. The return `/api/github/setup` checks the installation with
        `GET /user/installations` and the person's own token, at that
        moment. Then one current Admin workspace links at once, several
        open the picker, and none keeps `waiting_for_admin`. This replaces
        "Setup never chooses the workspace" for an add. A first install and
        a reinstall within 30 days keep their behaviour.
        Branch: `conductor/ciproxy-add-organization`. Waits for Milestone 30.
  - [x] Keep a request return as `waiting_approval`, with only its exact
        request from `setup_action=request` and no workspace. After the
        approval, link only after the check and the choice of the person who
        asked. Never infer a workspace from the start page, the newest add,
        the person, or the time of an event.
        Branch: `conductor/ciproxy-add-organization`. Waits for Milestone 30.
  - [x] Keep an inactive organization out of every runner path. Keep the
        plan reason and the Admin reason apart from a GitHub suspension or
        removal, and let both hold. Count every linked organization except
        removed ones. End the plan reason when the organizations fit the
        tier, and the Admin reason when a current Admin turns it on. Apply
        the Admin reason only when the person lost the role before the link
        completed. After Milestone 13 merges into the primary branch, handle
        its inactive lifecycle and pending installation outcomes. Exclude an
        organization from runtime snapshots unless `organization_runnable`
        allows it. The Add branch owns these control-plane changes.
        Implementation call, 6 November 2026: a removal and a new install
        within 30 days keep a hold and its reasons, because neither is an
        approval or a change of the plan. Keep the held registration, its
        history, and its intent until the normal deletion of retained
        records. A new install never resets a hold.
        Branch: `conductor/ciproxy-add-organization`. Waits for Milestone 30.
  - [x] Serve the picker, the waiting states, the reasons, and "Turn on",
        with the actions that each role may take. Regenerate the API types.
        Branch: `conductor/ciproxy-add-organization`. Waits for Milestone 30.
- [x] Give the build of the API with each response, for the reload of an
      old dashboard tab in Milestone 19.
      Branch: `conductor/ciproxy-mvp-milestone-14-api`.
- [x] Give the install command to an Admin.
      Branch: `conductor/ciproxy-mvp-milestone-14-api`.
- [x] Serve Setup, Machines, and Settings, with "Clear cache". Milestone 15
      adds the Boat parts.
      Branch: `conductor/ciproxy-mvp-milestone-14-api`.
      Implementation call, 5 November 2026: Setup reads durable UUID GitHub
      demand. Inject machine capacity and shared root sizing from the
      runtime, so the API needs no new agent report fields.
- [x] Test each route at the HTTP level with a real store.
      Branch: `conductor/ciproxy-add-organization`.
      Items 82 and 83 chose the add rules.
  - [x] Test the implemented routes with PostgreSQL, the gateway, and the
        GitHub App service. Verify upstream adapters with a local HTTP peer.
        Branch: `conductor/ciproxy-mvp-milestone-14-api`.
  - [x] Test the add routes with a real store. Cover one, several, and no
        Admin workspace, an event before the return, two adds at once, and
        a failed installation check. Cover a waiting request, a late
        approval, a lost return, a lost Admin role, a full Free workspace,
        several inactive organizations on Free after a take-out, room after
        a take-out, an upgrade, "Turn on", a removal and a new install that
        keep a hold, an active retained reinstall, and the organizations
        that keep running.
        Branch: `conductor/ciproxy-add-organization`. Waits for Milestone 30.
- [x] Run Redocly in the `api-types` gate phase, with its minimal rule set
      and its path-parameter check, on the generated API contract, as item
      84 says. Pin the Redocly version. Update `ts/api/README.md` and
      `xtask/README.md`.
      Branch: `conductor/ciproxy-add-organization`.
- [x] Orchestrator: mount the router at `/api` through Milestone 13. Supply
      the shared gateway, role and install adapters, selected-image sizing,
      and current release metadata. Run API maintenance on startup and at
      least hourly. Discover listeners with `Store::listener_scale_sets`
      to keep retired bindings through completion and the final ACK.
      Branch: `conductor/ciproxy-mvp-milestone-13-e2e`.
      Code status, 6 November 2026: the control plane mounts the router and
      runs API maintenance at startup and every minute. The authenticated
      live probes of the next TODO have not run.
- [x] Apply item 98 A to the security protocol and GitHub crate guide.
      State the four App permissions and the Members read requirement
      for the owner and member checks through the person's sign-in.
      Branch: `conductor/items-94-b-and-98-a-boat-wallets-and-members-permission`.
- [ ] Orchestrator: run the live API, repeated-authorization, and GitHub
      install-binding probes in `crates/ciproxy-api/scripts/README.md`.
      Read credentials through `CIPROXY_API_CREDENTIALS_FILE`. Record no
      secret in evidence. Branch: `conductor/ciproxy-mvp-milestone-14-api`.
      Verification evidence: `.context/ciproxy-mvp-m14/`.

## Milestone 15: Overflow to Boat end to end

Depends on: 12, 13, 14

This milestone builds "A job on Boat" of `architecture.md`.

Implementation calls, 6 November 2026:

- Keep `BOAT_SHARED_SNAPSHOT` optional until the shared image exists.
  Without it, own-machine service starts and Boat dispatch stays off.
- Match the approved mockup money format for derived Boat amounts: round
  each aggregate once to the nearest cent, with half-cents away from zero.
  Keep spend admission unchanged. Show unconfirmed costs separately from
  confirmed spend, including bounds that settled after their lifetime.

- [x] Wire Boat into the scaler loop with every rule of `overflow.md`.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Serve the Boat step of Setup and Settings under Overflow, with the key
      check and the spend limit at each connection.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Supply scripts for the overflow, idle-delete, and spend-limit smokes.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Reject startup authority for a retired label before dispatch.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Keep fractional queue times when applying the overflow wait.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Apply item 89 A after an owner removes the App. Keep launched Boat
      sandboxes until runner exit or their saved lifetime. Keep running
      jobs and the existing exit callback. State the exception and its
      reason in both protocol docs. The user chose A on 7 November 2026:
      App removal is rare, and lifetime plus the spend limit bound cost.
      My calls, 7 November 2026: use the saved owner-removal timestamp and
      the existing runner hold. Read removal again after successful GitHub
      retirement. Keep registration-only recovery and exit cleanup
      separate, because ended deliveries already permit safe cleanup.
      A new install in the same workspace restores ordinary retirement,
      because it ends the saved removal state. Keep guest retirement B in
      `plans/follow-ups/boat-guest-retirement.md` for later review.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Record the broader immutable scheduler-binding fix in
      `plans/follow-ups/immutable-scheduler-bindings.md`.
      Include the existing own-machine name-matching problem.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Apply owner review finding 1 C. Give both runner destinations one
      retirement result. Keep take-out pending until every saved own-machine
      and Boat reservation has retirement or running-job proof.
      My calls, 7 November 2026: reuse the lifecycle acceptance barrier,
      saved provider deletion, Busy writes, and registration-intent removal.
      Count the union of Boat intents, deliveries, and stored runners.
      Require a current busy answer beside the saved result, because an
      old Busy row cannot prove that a failed GitHub call is safe.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Apply owner review finding 2 A. Test reinstall and Boat reconnect
      with the real store. Prove early deletion of old idle supply while
      running work stays. Prove failure with a permanent removal hold.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Apply re-review finding 6 B. Defer inaccessible registration recovery
      without GitHub calls after owner removal. Keep take-out unanswered
      and resume ordinary recovery after reinstall.
      My call, 7 November 2026: pass loaded organization facts and use an
      explicit deferred result. Keep broader registration retention in
      `plans/follow-ups/boat-registration-retention.md` for after merge.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Apply owner review round 2 finding 1 A. Test successful recovery of
      a saved post-create Boat delivery through a recreated runtime tick.
      Keep identities, one create request, and one counted cost bound.
      Prove failure when runtime recovery admits only Unsent deliveries.
      Branch: `conductor/mvp-m15-overflow`.
- [x] Orchestrator: Configure the optional Boat deployment secret as
      `deploy/runtime.md#boat-shared-snapshot` describes.
      Set it to the shared Milestone 10 snapshot.
      Keep the selected image manifest at the same release.
      My call, 7 November 2026: deliver the share code through Secret
      Manager and a Kubernetes Secret, like the release token, and not
      through a GitHub variable, because the deploy logs are public.
      Pull request #37 built it. The main preview holds version 1, the
      share code of snapshot `2026-11-06-7484d1b`, which is also the release
      of its `JOB_IMAGE_MANIFEST_PATH`. Production gets its version at launch.
- [x] Apply item 94 B to the Boat key check and connection API.
      Return named personal and organization wallets and the likely
      selection. Prove both key kinds through HTTP with a real PgStore.
      Update the overflow protocol and regenerate the API types.
      My call, 8 November 2026: preselect Boat's active wallet, then
      its first offered wallet. Reason: Boat supplies the likely wallet.
      Read the wallets without choosing one, so organization keys work.
      The key check saves no key.
      Branch: `conductor/items-94-b-and-98-a-boat-wallets-and-members-permission`.
- [x] Apply review finding 1 E for the wallet key check.
      Prove active-wallet selection and its first-wallet fallback at
      the Boat boundary. Keep the existing personal and organization
      HTTP scenarios.
      Branch: `conductor/items-94-b-and-98-a-boat-wallets-and-members-permission`.
- [ ] Orchestrator: Run the three live scenarios with the private credential
      file, as `crates/ciproxy-control-plane/overflow.md` describes.

Verification evidence: `.context/ciproxy-mvp-m15/`.

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
- [x] Write items 63 and 64 into `metering.md`: the last price counts for
      a day with a price change, and a month has the days of its calendar
      month. Build both rules, and fix the 30-day view of the Savings
      mockup. Then run `cargo test --package ciproxy-metering` and
      `cargo xtask check` with `DATABASE_URL` set.
      The report, service, and test callers no longer choose a monthly
      divisor. Each date uses its calendar month.
      Branch: `conductor/ciproxy-mvp-milestone-16-cost-rules`.

## Milestone 17: Billing with Stripe

Depends on: 4

`ciproxy-billing` keeps the tier and the add-ons in step with Stripe, as
`billing.md` says, after `futex-ai/accounting`. My call: the return page and
the event both read the session, so the tier changes once.

- [x] Create the prices, and open Checkout, with no tax, as item 39 says.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing`.
- [x] Take each signed event once. Change tiers and add-ons with proration,
      and open the portal.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing`.
- [x] End the subscription of a workspace at the end of its period when
      its last organization leaves.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing`.
- [x] Apply the verified billing review fixes and rerun the complete gate.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing`.
- [x] Preserve pending upgrade expiry across billing reconciliation and
      recover abandoned changes after the Stripe pending window.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing`.
- [x] Tie failure dates to typed invoice evidence and test invalid
      signatures through the public billing service.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing`.
- [x] Require expected requests for each scripted Stripe subscription
      outcome and prove that wrong prices fail the integration tests.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing`.
- [x] Keep durable renewal outcomes after recovery. Replay the live paid
      renewal with a void upgrade and later events without a new failure.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing-live-fixes`.
- [x] Persist verified automatic collection stops for owned unpaid
      renewals. Ask Stripe to cancel and record Free after confirmation.
      Replay the final retry and exclude manual stops and paid invoices.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing-live-fixes`.
      Evidence: `.context/ciproxy-mvp/billing-live-fixes/verification.md`.
- [x] Apply PR #23 review finding 1, option B. Keep eligible, superseded,
      and completed cancellation decisions for each signed stop event.
      Test invalidation before manual events, restart, and a new stop.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing-live-fixes`.
      Evidence: `.context/ciproxy-mvp/billing-live-fixes/pr23-fix-round-1/`.
- [x] Complete the PR #23 cancellation read path. Lock known subscription
      events before Stripe reads and merge every owned observation before
      later reads. Preserve supersession from expanded renewal invoices.
      Branch: `conductor/ciproxy-mvp-milestone-17-billing-live-fixes`.
      Evidence: `.context/ciproxy-mvp/billing-live-fixes/pr23-fix-round-1/`.
- [x] Test each state with recorded events and the Stripe test mode.
      The orchestrator ran the
      [Stripe test mode checklist](../crates/ciproxy-billing/README.md#stripe-test-mode)
      in the account of item 61 with real signed events, test clocks, and
      the bank confirmation. The rerun on 5 November 2026 at `287f1bc`,
      after the merge of #23, gave 37 passed, 0 failed, and 1 skipped.
      Evidence: `.context/ciproxy-mvp/billing-live/report.md`.
- [ ] User: turn on receipt emails for successful payments in the sandbox
      now and in the live Stripe account before launch. Open
      <https://dashboard.stripe.com/settings/emails>, then turn on
      **Successful payments** under **Payments**. The live checklist skipped
      this row, because only the Dashboard shows it. Test payments never
      send these emails.

## Milestone 18: Dashboard API: jobs, usage, savings, and billing

Depends on: 14, 16, 17

These routes serve the other pages of the dashboard.

- [ ] Orchestrator: finish serving Jobs, Savings, Plan and usage, Plans,
      Checkout, and one banner for each page. Complete the composition and
      paid-review steps below.
      Branch: `calummoore/ci-proxy-load-balancer`.
  - [x] Add the API routes, Rust wire forms, and store projection. Preserve
        opaque workspace access, tier windows, trusted job facts, and the
        single-banner priority. Generate the TypeScript contract.
        Branch: `conductor/mvp-m18-api`.
        Implementation call, 6 November 2026: round only displayed savings
        amounts to integer cents, with half cents away from zero. Reason:
        this matches the existing mockup formatter and keeps metering exact.
  - [x] Add label-specific fit data for the Jobs waiting notice. Reuse the
        existing machine fit, Boat size selection, and image root sizing.
        Keep unknown fit distinct from no fit. Test role-aware actions over
        HTTP.
        Branch: `conductor/mvp-m18-api`.
        Evidence: `.context/ciproxy-mvp/m18/nothing-fits-handoff.md`.
- [x] Finish HTTP verification after the ignored-event parser repair.
      Keep its HTTP regression in the suite.
      Branch: `conductor/mvp-m18-api`.
  - [x] Add real-PgStore HTTP tests for every route and a supported signed
        Checkout smoke through a bound dashboard server. Verify the twelve
        independent tests in ten exact-name runs each.
        Branch: `conductor/mvp-m18-api`.
        Evidence: `.context/ciproxy-mvp/m18/requirements.md` and
        `.context/ciproxy-mvp/m18/test-cost.md`.
  - [x] Apply review findings 3, 4, 5, and 6 within the owned API and store
        files. Preserve aliased lifecycle facts, known starts without a
        timestamp, current queue membership, and fit during accounting gaps.
        Add HTTP regressions and repeat the new or changed tests ten times.
        Branch: `conductor/mvp-m18-api`.
        Evidence: `.context/ciproxy-mvp/m18/review-fix-round1/`.
- [ ] Orchestrator: compose `PageServices` in the control plane. Mount
      `router_with_pages` under `/api` and `stripe_router` at the dashboard
      host root for `/hooks/stripe`. Supply the existing billing, metering,
      and overflow services and dated USD comparison price lists. Supply
      the exact Boat `RootSizing` already used by the overflow runtime.
      Poll `billing_cancellations` and call `reconcile_removal` on blocking
      workers so durable last-organization cancellations complete.
      Branch: `calummoore/ci-proxy-load-balancer`.
- [ ] Orchestrator: implement and inject `PaidReview` with an authoritative
      Stripe invoice preview for a move between paid plans. The existing
      billing review gives the full monthly price as today's charge. The
      API keeps paid review unavailable until the preview exists.
      Branch: `calummoore/ci-proxy-load-balancer`.
- [x] Fix the billing parser for signed Stripe events that it ignores.
      Use a typed parser and add the billing-level unit regression.
      Keep the HTTP regression for real `data.object` payloads.
      Branch: `conductor/mvp-m18-api`.
      The user extended the billing scope on 7 November 2026 for item 1-E.
      Evidence: `.context/ciproxy-mvp/m18/billing-fix-round2/report.md`.
- [x] Add the quantity-only billing operation of review finding 2.
      Resolve the confirmed tier after Stripe reconciliation under the
      existing billing lock. Switch the add-on route and quantity-only
      callers. Preserve the paid tier in the stale-store HTTP and
      provider-form regressions.
      Branch: `conductor/mvp-m18-api`.
      The user extended the billing scope on 7 November 2026 for item 2-B.
      Evidence: `.context/ciproxy-mvp/m18/billing-fix-round2/report.md`.
- [x] Apply owner review round 1, finding 1, option A. Preserve original
      job requests from retained own-machine and Boat execution facts.
      Use current labels only while queued. Report missing executed
      requests as unknown. Keep the public request field names and make
      their values nullable. Update their generated types and descriptions.
      Add real-PgStore history and detail regressions after a label edit.
      Branch: `conductor/mvp-m18-api`.
      Implementation call, 7 November 2026: keep one optional request in
      the store projection and nullable values at the existing wire paths.
      Reason: preserve one complete request and the current field names.
      Evidence: `.context/ciproxy-mvp/m18/owner-fix-round1/report.md`.
- [ ] Orchestrator: expose the overflow owner's resource-only catalog
      accessor of review finding 6. Replace the private fit-conversion
      adapter after this workspace merges. Keep accounting and availability
      outside physical fit, and preserve the accounting-gap regression.
      Branch: `calummoore/ci-proxy-load-balancer`.
- [ ] Orchestrator: run the
      [live page checks](../crates/ciproxy-api/scripts/README.md#jobs-savings-and-billing)
      after the runtime and Stripe test workspace are ready. Set
      `CIPROXY_API_CREDENTIALS_FILE` to a private file outside the repository.
      Run `live_pages.py` for reads, review, Checkout and return, paid
      changes, add-ons, the portal, Move to Free, and signed webhook replay.
      Branch: `calummoore/ci-proxy-load-balancer`.

## Milestone 19: Dashboard UI: shell, sign-in, and setup

Tags: ui

Depends on: 1, 14

The Expo app under `ts/app` builds each screen and state of its mockup, with
`@firna/ui`, `@ciproxy/tokens`, and the generated types. Each UI milestone
smoke-tests its pages at 1440, 720, and 390 pixels.

- [x] Build the shell, sign-in, the App install, Setup without its Boat
      step, the page for a member with no role, and the page of a removed
      App.
      Branch: `conductor/mvp-m19-ui`.
      Implementation call, 6 November 2026: keep static page routes and
      carry opaque workspace IDs in query parameters, so the existing
      static hosting serves a reload of each page.
      Orchestrator call, 7 November 2026: include the exact API continuation
      field, its HTTP test on a real store, and regenerated types.
      Implementation call, 7 November 2026: join the claim IDs from the
      latest GitHub authorization to their link results. Handle each ended
      continuation once. Show every result when several claims end,
      because claim order and the remembered workspace choose nothing.
      Orchestrator call, 7 November 2026: fix PR #36 finding 8 with B.
      Keep continuation data in the store, so claim counts cannot enlarge
      the sign-in cookie. Include its schema, migration, and store support.
      Implementation call, 7 November 2026: key a companion record by the
      session ID already in the cookie. The private session row depends on
      a retained person and does not exist for browser-only sign-ins.
      The companion keeps those results without a new cookie reference.
      Owner review round 1, 7 November 2026: use one proof-result collector
      for setup and sign-in. Record canonical IDs at proof or completion,
      and keep previously proved completed claims out. Give the app action
      wrappers content-based sizing by default, so full names stay visible.
      Keep Firna unchanged and preserve its minimum size for short labels.
      Owner review round 2, 7 November 2026: fix its finding 1 with B, so a
      failed sign-in cannot consume an undelivered install result. Store
      pending results apart from proof status. Save each result in the
      transaction of its proof. Consume only the exact read results in the
      transaction that saves the session and continuation. A failed cookie
      preparation or continuation save keeps them for the next successful
      authorization, and keeps the earlier sign-in and business records.
      Refresh and workspace choice add none. Keep the wire field, bounded
      cookies, fixed continuation expiry, and the exclusion of delivered
      history. Its finding 2 stays open as item 91, with no option applied.
      Implementation call, 7 November 2026: add the pending table to the
      unmerged migration `202611071200_dashboard_continuations`, not to a new
      migration. Reason: it ran only in disposable test databases and is not
      on the target branch, so the `databases` skill does not freeze it. No
      merged migration changes.
- [x] Reload a dashboard tab that is older than the API. My call: the API
      gives its build with each response, and the dashboard reloads at the
      next page change after the build changes. Reason: an old tab must not
      call a newer API with old types, and a page change loses no input.
      Branch: `conductor/mvp-m19-ui`.
      Verification evidence: `.context/ciproxy-mvp-m19/`.

Owner review round 2 answers, 8 November 2026: the user chose A for item
91 and A for item 92. Both answers keep the product behaviour. The app, the
API, and the generated types stay unchanged. Only the two approved mockups
change to match the app.

- Item 91, the workspace picker of "Add an organization": each row shows
  the workspace name and, when needed, the upgrade warning. The picker
  mockup drops the tier and the organization count.
- Item 92, the workspace switch: each row keeps the role, or "No role yet".
  The switch mockup shows the role in place of the tier and the count.
- Neither list needs the optional shared plan summary now. At the user's
  explicit request, this branch deletes the three settled follow-up items
  `workspace-picker-plan-facts.md`, `workspace-switch-plan-facts.md`, and
  `workspace-list-plan-facts.md` under `plans/follow-ups/`.

Item 98 answer, 8 November 2026: the user chose A. The GitHub App gets the
organization permission "Members: read". The App then has four permissions.
A sign-in reads the organization memberships of the person with it.

- The install mockup and the install page of the app list the four
  permissions. Both explain "Members: read" as "see who belongs to your
  organization, so that ciproxy can give roles".
- For item 98, this branch changes only that list and this record. The
  backend change and the protocol docs for the membership read are outside
  this branch.

Installation follow-up outcomes, 8 November 2026:

- `session-installation-attempt-growth.md`, B: session reads return a stable
  navigation link and create no attempt. An Origin- and CSRF-protected POST
  saves the attempt before returning its GitHub URL. Keep durable Add
  evidence, continuation purposes, late approvals, and no age limit.
  Rejected: reusing read-created attempts, signed Direct state alone,
  sharing one open attempt, and adding an expiry.
- `retained-add-reinstall-admission.md`, B: use one effective admission
  policy for installation, synchronization, and registration. Retained
  admission overrides a new Add's initial holds. Prepare current bindings
  before publishing Active. Retained inactive organizations keep their
  holds. Rejected: a local preparation patch, separate readiness storage,
  and preparation for every held installation.
- `unused-retired-label-query.md`, A: delete the unused trait method,
  PostgreSQL method, helper, and imports. Keep retired data and the
  workspace discovery query that maintenance uses.
- Admin recovery flag: return false for an existing Admin. Remove the
  dashboard's extra role check, because the API owns this capability.
- Python compatibility: defer annotation evaluation in `live_smoke.py`.
  Check the other crate and deployment scripts for evaluated union types.
  Keep the documented Python 3.9 minimum without another runtime dependency.
- My call, 8 November 2026: use the existing `/install` page as the stable
  navigation link. Use POST `/installation-links/start` for the explicit
  start, so the existing browser guard protects it.
- My call, 8 November 2026: put the pure activation predicate on
  `OrganizationActivation`. The GitHub service and registration query then
  consume the same retained-admission rule.
- Post-push review, 8 November 2026: apply B for the install link's native
  navigation. Give `ActionLink` an opt-in plain-activation callback. Only
  that callback marks the link as an action for the build-change handler.
  Keep other links and their raw `onClick` handling unchanged. A modified
  click opens the stable install page and creates no attempt. A plain
  activation saves its attempt, also after observing a new API build.

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
- [ ] Show an unknown request on Jobs. The API returns null vCPUs, memory,
      and free disk for a running or finished job whose request was not
      saved. Add that state to the Jobs mockups first.
      Source: pull request #34, fix round 1, 7 November 2026.

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

- [x] Start here: Overview, Quickstart, and How ciproxy works.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Guides: Add a machine, Connect Boat, Move your workflows, Set a spend
      limit, and Remove a machine.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Reference: Labels and VM sizes, Agent install options, GitHub App
      permissions, and Plans and starts.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Say on the docs page of the job image that a job can run its steps
      in any Docker image with the `container:` key, as item 78 = C says.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Security: Job isolation, and The agent connection.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Show the mockup's steps, cards, and default-label badge. My call,
      7 November 2026: use Astro's MDX component map and expressions with
      the Starlight components, because the existing Markdown check accepts
      them. The gate configuration stays unchanged.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Fix the copied workflow examples in Quickstart and Move your
      workflows with `CodeBlock` and `afterChange`. Parse both copied jobs
      as YAML in the site regression tests. Owner review round 1, finding 1.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Apply item 93 = C to the agent host requirement, the approved
      install mockup, and the docs. State the supported Linux hosts with
      systemd and cgroup v2. The Milestone 20 workspace owns the dashboard
      note. Wider host support stays in its follow-up item.
      My call, 8 November 2026: Add a machine owns the distribution list,
      and the other docs pages link to it, so the list has one home.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Apply item 96 = A in the shared docs grid styles. Show two columns
      at 720 and 1440 pixels and one at 390 pixels. Put the Overview Next
      link in the right half on the two-column layout.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Apply item 98 = A to GitHub App permissions and the marketing
      permission texts. List Members read as the fourth permission, for
      organization memberships and roles at sign-in.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Apply item 94 = B to Connect Boat. Explain the personal or
      organization wallet choice, with the likely wallet preselected.
      Branch: `conductor/m24-launch-docs-pages`.
- [x] Smoke-test all 14 pages at 1440, 720, and 390 pixels. Check mobile
      navigation, search, page links, anchors, light mode, and aside borders.
      Branch: `conductor/m24-launch-docs-pages`.
- [ ] Orchestrator: check each page against the running product at
      `https://br-main.preview.app.ciproxy.dev`, after the dashboard
      workspaces deliver their screens. Use a disposable private repository,
      a KVM test machine, and a Boat test account.
  - [ ] Overview: confirm the Free plan, an own-machine start, Boat after
        the default wait, and a machine-cache restore.
  - [ ] Quickstart: follow sign-in, App install, machine registration, and
        the first job with the documented command and workflow change.
  - [ ] How ciproxy works: check warm runners, drain, cache clearing, Boat
        routing, and steps in a job container.
  - [ ] Add a machine: check Admin access, token expiry and revoke,
        registration, hardware, slots, no KVM, disk fit, and repeat install.
  - [ ] Connect Boat: check the key, wallet choices and preselection,
        account name, spend limit, wait, rejected key, key change, and
        disconnect.
  - [ ] Move your workflows: check repository access, the `runs-on` change,
        cache actions, and the Docker image example.
  - [ ] Set a spend limit: check confirmed and pending spend, blocked new
        Boat starts, running jobs, and a sufficient increase of the limit.
  - [ ] Remove a machine: check drain, resume, removal while offline,
        credential revocation, the machine count, and local uninstall.
  - [ ] Labels and VM sizes: check default and custom labels, free disk,
        slots, fit, names, deletion, and both "When nothing fits" choices.
  - [ ] Agent install options: check command help, host preparation, the
        systemd service, repeat install, outbound access, and updates.
  - [ ] GitHub App permissions: check the four permissions, sign-in
        memberships and roles, private repository access, personal accounts,
        suspension, and removal.
  - [ ] Plans and starts: check prices, starts, machine counts, history,
        add-ons, period dates, soft limits, and the payment controls.
  - [ ] Job isolation: check fresh VMs, cache scopes, Docker disks, public
        internet access, and blocked private, host, and metadata access.
  - [ ] The agent connection: check WSS, reports, offline detection,
        reconnect, continuing jobs, credential protection, and agent version.

The source map, browser evidence, and detailed live checklist are under
`.context/ciproxy-mvp/` in the delegated workspace.

## Milestone 25: Tier features: backend

Depends on: 13, 18

Milestone 13 owns the Docker attachment, trusted writer, disposable copies,
valid drive IDs, and the item 80 = D reservation. This milestone keeps
the remaining Docker disk recovery work for Team and Business at launch.
Several organizations come with the workspace of item 48 in Milestones 6,
14, and 20.

- [x] Discard a disk that fails to mount. My call: a rebuild loses only
      time.
      Branch: `conductor/m25-docker-disk-recovery`.

## Milestone 26: Dashboard UI: tier features

Tags: ui

Depends on: 20, 21, 25

Each screen follows its mockup, as Milestone 19 says.

- [ ] Build the Docker disks of the machine page.

## Milestone 27: Launch

Depends on: every milestone from 1 to 26

The repository is public from item 70 = E, and the site links to it.

- [ ] Run the CI of a private test repository on ciproxy, and fix or record
      each problem. ciproxy runs private repositories only, and item 70 = E
      made this repository public.
      Close the follow-up item `expo-56-npm-advisories.md`.
- [ ] Draft the terms and the privacy policy, which the user writes or
      approves. They state the limits of "Data retention" in `security.md`.
- [ ] Register the production App and the Stripe live mode, which need the
      accounts of the user.
- [ ] Deploy production. Smoke-test a first job, an overflow, and a live
      checkout that the user approves.
- [ ] Check that the open parts build alone, and check the licence of the
      installer of `futex-ai/cli-release`.
- [ ] Ask Boat to confirm in writing that a customer may run ciproxy
      runners on its own Boat account. The user sends the request. My call:
      the terms allow this use by inference only.
- [ ] Check the three prices of the cost claim and the GitHub charge for
      self-hosted runners again. Update the price list of the site and its
      date. Reason: GitHub only postponed a charge of $0.002 a minute, which
      would lower the claim to about 54%.
- [ ] Publish the site. Then make the GitHub App public.

- [ ] Move the image-publishing credentials of item 100 = B from the
      reusable image workflow on the primary branch to the one on protected
      `main`, after the launch merge puts the repository on `main`.

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

Answers of 5 November 2026:

- Item 56, D: CI plans, checks, and promotes every root after the one-time seed
  and first bootstrap. A read-only identity saves a plan of each root. A policy
  refuses each destructive change that has no matching approval file in the
  merge. A separate apply identity, which only one reusable workflow can use,
  applies that saved plan. Helm, the NEG wait, and the edge follow. Production
  runs the same jobs after the main preview. The database and the secrets get
  deletion protection. Rejected: options A, B, C, and E. The deny policy of B
  waits in the follow-up item `production-deny-policy.md`.
- Item 57, B: `Firna-AI` moves to GitHub Team, and a ruleset on `main` needs a
  pull request and the gate, and blocks force pushes and deletion. The user
  upgrades the plan. Rejected: GitHub Free with the rule "Agent merges: off"
  only.
- Item 58, B: production deploys on each merge into `main`, after the main
  preview run of the same commit passes. A GitHub variable keeps production off
  until launch. Rejected: the `deploy-*` tag, and a daily promotion.
- Item 59, C: only the 7 trusted events of GitHub write shared entries in the
  machine cache: `push`, `workflow_dispatch`, `repository_dispatch`, `delete`,
  `registry_package`, `page_build`, and `schedule`. A direct write to the GitHub
  cache that a workflow opted into with `cache-mode: write` follows the rule of
  GitHub, as on GitHub-hosted runners. Rejected: required read-only rights, and
  a runtime token kept outside the job VM.
- Item 60, A: the orchestrator creates the private test repository
  `Firna-AI/ciproxy-spike-refused`, runs the runner group probe, and deletes the
  repository. Rejected: keeping the repository, and no check.
- Item 62, C: `install` sets up Firecracker, the jailer, and the host tools, and
  creates a reflink disk area on any file system, in one file formatted as XFS.
  Rejected: preparation by the customer, full copies where reflinks are missing,
  and C after launch.
- Item 63, B: on a day when a monthly price changes, the last price counts for
  the whole day. Rejected: the first price, a split by the hours of each price,
  which was the recommendation, and the time used.
- Item 64, B: a monthly price becomes a daily cost by the days of its calendar
  month. Rejected: 30 days, 365 divided by 12 days, a mix of bases, and billing
  periods.
- Item 65, B: the orchestrator merges `futex-ai/sandbox` #11 this one time. It
  merged as `9658361` on 5 November 2026.
- Item 54, E: the machine cache checks the runtime token of each job outside the
  job VM. It checks the signature against the public keys of GitHub, the issuer
  and the expiry, and that the run and the runner of the token match the job
  that ciproxy started. The scope of the token with write access gives the full
  ref. A shared write needs that scope to be the default branch and the event to
  be trusted, as item 59 says. Every other job saves only in its own scope. The
  App keeps its three permissions. This reverses the line of item 32 that left
  out the token scopes, because a signature check now verifies the token.
  Rejected: the API only, `jobWorkflowRef` for direct jobs, the hook report, a
  test cache entry, the push webhook with the "Contents" permission, and the
  OIDC token. Before the build, three live checks run: a push to the default
  branch, `pull_request_target`, and a job on a ciproxy runner. If a check
  fails, the fallback B goes back to the user.
- Item 55, A: a job whose runtime token gives no checked ref fails closed in the
  machine cache. It restores default-branch entries from the machine cache, and
  its other entries through the GitHub cache. It saves in a private scope that
  no other job restores, and through to the GitHub cache. It gets its own copy
  of the Docker disk, which ciproxy discards. The job facts get one "unknown"
  state with this one rule. Rejected: the GitHub cache only for such a job,
  because one change of the token format would take the machine cache from every
  job, and a later promotion through the cache list, because it is much code for
  a rare case.
- Item 67, B: the spend limit counts the confirmed Boat spend plus the cost
  bound of each start whose cost Boat has not confirmed. The bound is the
  published hourly price of the size for the whole sandbox lifetime, 6 hours or
  2 on a free trial, plus the saved start-fee quote. New starts continue while
  the limit covers both. When Boat reports the real cost, or the lifetime has
  passed, the real cost or the bound replaces the estimate. The pages show the
  confirmed spend and the amount still pending. `overflow.md` states the rule.
  Rejected: holding all starts until Boat confirms, Admin reconciliation by
  hand, one combined estimate figure, and recovery through a replacement key.
  Item 69 replaces the saved quote with the highest published start fee.
- Item 68, A, with one shared cluster: the user wrote "ciproxy holds prod and
  test infra, we can share. the same k8 cluster". Both the main preview and
  production run in the GCP project `ciproxy`, and they share one GKE Autopilot
  cluster. This replaces the project rule of 4 November 2026 for the main
  preview, and the project of item 53. The spike and test machines stay in
  `internal-498318`. Rejected: a new project for the main preview, and the main
  preview in `internal-498318`.
- Item 69, B: each start counts the highest published Boat start fee, $0.07
  today, from `overagePrices.startBrackets` of `GET /limits`, and not the
  saved quote. Boat reports no charged start fee, so this fee stays counted.
  The user wrote "let's not over engineer this feature, it's a nice to have, if
  we block a bit early 0.07 early that's really not a problem". So the limit
  can stop overflow early, by $0.07 for each start that Boat does not charge.
  `overflow.md` states the rule. Rejected: the saved quote, a second quote
  after the start, and a charged fee from Boat.
- Item 73, A: the image keeps the full software of `ubuntu-latest`, 58.3 GB.
  The label `ciproxy` gets 80 GB of disk, and job disks get a root reserve of
  1%, so a job keeps about 19 GB free. Blacksmith gives the same runner size
  80 GB. The image budget holds two images and one download during an
  update, about 135 GB, apart from the job disk. My call: the host reserve
  stays a quarter of the disk that the image budget leaves. An own machine
  needs 250 GB of disk. Boat stays on `large`. Rejected: a slim image, and a
  shared read-only image with a writable layer for each job.
  User calls in the item 73 implementation brief, 5 November 2026: set the
  default image budget to 140 GB for two 58.3 GB images and one 18.3 GB
  download, with about 2 GB for image growth. Subtract that budget first,
  then reserve a quarter of the remaining disk for the host. The other
  three quarters hold jobs. A 250 GB machine leaves 27.5 GB for the host
  and 82.5 GB for jobs. The existing budget override stays available.
  My call, 5 November 2026: each job VM gets one root disk of its label
  size, so Docker data shares the free space as on GitHub, and the cached
  Docker images of the image stay visible. The agent grows the reflink copy
  of the 80 GB image disk to a bigger label. The empty Docker disk, which
  the guest never mounts, goes. Milestone 13 attaches a sticky Docker disk
  where `cache.md` gives one. Reason: the split of the agent left the job
  two smaller disks and hid the cached images.
- Item 72, B: `ciproxy.dev` stays on Cloudflare. Production creates Cloud DNS
  zones for `app.ciproxy.dev` and `get.ciproxy.dev`, and NS records in
  Cloudflare delegate them. The user adds the site record of `ciproxy.dev`
  in Cloudflare by hand, once. Rejected: a move to Cloud DNS, and the
  Cloudflare provider in Terraform.
- Item 74, A: both namespaces of the shared cluster enforce the Baseline
  PodSecurity level, with labels that only the operator can change, so no
  pod mounts `hostPath`. Rejected: a custom admission policy, and a trusted
  deployment controller.
- Item 75, A: the cost claim keeps the AX42 with both disks combined into
  one filesystem of 1,024 GB, for six default jobs. The site and the mockups
  state this basis, for example RAID0 or one LVM volume. The machine setup
  docs say that a machine used only for CI can combine its disks, because
  its jobs, machine cache, and Docker disks are disposable. Rejected: the
  default mirror of 512 GB, and another machine.
- Item 77, D: the disk of a label is the free disk that each of its jobs
  gets, as GitHub states its runners. The VM disk is the image plus that
  free disk. The image disk is built at its smallest size, and the agent
  grows each job's copy by the free disk of its label. The scheduler counts
  the image plus the free disk on machines and for the Boat size, from the
  root disk size of each image release. Rejected: the image size as a floor
  with a warning, a refusal on save, and a floor for placement only.
  My calls, 5 November 2026:
  - The default labels get 20, 20, and 60 GB of free disk, so their VM
    disks stay close to today's 80, 80, and 120 GB. The image budget of
    140 GB and the machine minimum of 250 GB stay.
  - "Run on the largest Boat size" can give a job less free disk than its
    label, as it gives less vCPU and memory, but never a size that cannot
    hold the image.
  - A label keeps a free disk of at least 1 GB.
- Item 78, C: until launch, every label starts from the full image, and a
  job picks other software with a job container, the `container:` key of
  GitHub Actions. The docs say so. The user wrote "we will do A/B after
  launch": a catalog of ciproxy images that each label names, and custom VM
  images for Business. The follow-up item `job-image-catalog-after-launch.md`
  holds that work. Rejected for now: toolset layers.
- Item 81, C: the control plane and the customer data run in `us-central1`.
  The user wrote "lets go with US then it has better capacity", after
  Google had no capacity for the new cluster in `europe-west1`. It costs
  about 10% less for compute, and the data then lives in the US. The
  Terraform state buckets of the seed stay in `europe-west1`, because a move
  recreates them. Rejected: a retry of `europe-west1`, and `europe-west4`.
- Item 79, A, in the user's words: "lets not over engineer this, if they
  remove the runner, then something should probably fail, like we can fail
  the action in that case". So Remove, `ciproxy-agent uninstall`, and the
  last App removal stop the warm runners at once. A job that a warm runner
  took before its handover to the guest then fails. Jobs that the guest
  received still finish. Ordinary stops and Drain still ask GitHub first.
  Rejected: B, C, D, and E. My call, 5 November 2026: the same rule covers
  an organization that removes the App while other organizations of its
  workspace stay. Reason: the root question of item 79 includes a leaving
  organization, and the answer accepts a failed job on any removal.
- My call, 6 November 2026, from Milestone 13: when GitHub reports a job as
  completed with no finish time, metering counts its start at once and
  reads the finish from the GitHub job with the "Actions" read permission.
  Until a finish arrives, the job is finished with an unknown time and adds
  no minutes. If no finish ever arrives, the job stays out of the minutes.
  Reason: GitHub's own finish meets the minutes rule of `metering.md`, and
  the rare fallback can understate the savings but never overstate them.
- Item 76, E, 6 November 2026: the organization rule refused the grant of
  D to the load balancer agent, with "One or more users named in the policy
  do not belong to a permitted customer". The project `ciproxy` now enforces
  the managed rule `iam.managed.allowedPolicyMembers`, with the futex.ai
  organization and the load balancer and CDN fill agents by name, and the
  legacy rule is off for `ciproxy` only. The user holds Organization Policy
  Administrator for this. A probe grant to an outside Gmail account was
  still refused. Rejected: A, B, C, and F.
- Item 82, B, in the user's words: "right we can just show a list of the
  users workspaces on return and then the user can pick one if there are
  multiple". So the person still chooses the organization on GitHub. When
  the person returns to ciproxy, ciproxy lists the workspaces where the
  person is an Admin. With one, it links at once. With several, the person
  picks one. A late approval with no browser session waits until the person
  who asked signs in again and picks the workspace. Rejected: A and C.
- Item 83, D, in the user's words: "we could link the workspace, but then
  mark it as inactive until they have the correct billing". So the
  organization joins the workspace that was meant, but stays inactive until
  the plan of the workspace allows another organization. An inactive
  organization gets no runners, and the dashboard says why, with the
  upgrade offer. It turns active by itself after the upgrade. Rejected: A,
  B, and C. My call, 6 November 2026: when the person who asked is no
  longer an Admin, the organization also joins inactive, and an Admin of the
  workspace can turn it on. Reason: the same rule, link and wait until the
  blocker clears.
- Item 85, F: the agent accepts a first handoff only over the host-owned
  connection that configured the runner before any job code ran. After a
  reconnect before the first handoff, that job uses GitHub's own cache for
  its whole run. The user asked for E as a follow-up item:
  `follow-ups/cache-service-egress-firewall.md`. Rejected: A, B, C, and D.
- Item 84, B: the `api-types` gate phase also runs Redocly, with its
  minimal rule set and its path-parameter check, on the generated API
  contract, and fails on an invalid structure. Rejected: A and C.
- Item 86, B: to build item 85 = F, the runner hook waits for the host
  decision before the steps of a job start. The decision is either "the
  machine cache is ready", with the sealed original handoff, or "use only
  GitHub's cache". The 30-second fallback and the early fallback on a
  socket error go. A reconnect before the first handoff still means only
  GitHub's cache. There is no guessed time limit; GitHub's job time limit is
  the backstop. Rejected: A and C.
- Item 100, B, 8 November 2026: the image-publishing credentials bind to
  the reusable image workflow on the primary branch until launch, and move
  to protected `main` at launch. Rejected: A and C.
- Item 101, B, 8 November 2026: the Machines and Settings page responses
  gain the facts of the approved mockups, with regenerated types. Rejected:
  A, C, D, and E.
- Item 102, A, 8 November 2026: the warm runner control shows the saved
  number in every state, and the mockup changes to it. Rejected: B and C.
- Item 113, A, 8 November 2026: the label dialog keeps the fit counts and
  the placement, and the mockup drops the largest-job figures. Rejected: B.
- Item 108, C with D, 8 November 2026: the paid change sends the same
  pending credit that the preview counted, Stripe computes the amount, and
  an alert compares each invoice with its quote. A third fix round of pull
  request #39 builds it. Rejected: A, B, E, and F.
- Item 109, C, 8 November 2026: the spend limit notice names the budget
  month and the day Boat resumes, as Jobs does. Rejected: A and B.
- Item 110, B, 8 November 2026: History loads pages without a total, and
  the database applies the filters, the order, and a limit. Each read stays
  bounded, also the live queue. Rejected: A, C, D, and E.
- Item 111, A, 8 November 2026: the banners keep today's wording, and the
  mockups change to it. Rejected: B and C.
- Item 112, D, 8 November 2026: ciproxy saves the actual size choice when a
  job starts, and Jobs explains what ran from it. Rejected: the other
  options.
- Item 103, A now and B in parallel, 8 November 2026: the agent release is
  a full release, so installs and updates take it. A pre-release channel
  for the preview release server follows, so that a test release reaches no
  customer. Rejected: no channel at launch.
- Item 107, A, 8 November 2026: the cache firewall proof keeps the cloud
  provider's metadata route and DNS, and checks the block against the real
  metadata server. Rejected: B.
- Items 104, 105, and 106, A, 8 November 2026: grow the test host disk to
  500 GiB, sign the sandbox in to GitHub with the `workflow` scope, and
  upgrade the test workspace to Team with a Stripe test-mode checkout after
  pull request #39 merges.
- Item 99, A, 8 November 2026: the live machine check publishes a
  pre-release of the agent on `Firna-AI/ciproxy`, creates two private test
  repositories in Firna-AI, and limits the workspace to the test
  repositories. Rejected: B and C.
- Item 94, B, 8 November 2026: the Boat form lets the customer choose the
  personal wallet or an organization wallet of the key, with the likely one
  preselected. Rejected: A, because organization accounts must work, and C.
- Item 95, B, 8 November 2026: each paid review stores its quote on the
  server. The payment names its quote, and retries keep its proration time.
  An unknown or outdated quote is refused, and the page asks for a new
  review. Rejected: A and C.
- Item 98, A, 8 November 2026: the App gets "Members: read", so that a
  sign-in can read the person's organization memberships. The App now has
  four permissions. Rejected: B, C, and D.
- Items 91 and 92, A, 8 November 2026: the workspace picker shows the
  name and the upgrade warning, and the workspace switch shows the role.
  The mockups change to match. Rejected: B to E of item 91, and B to D of
  item 92.
- Item 93, C, 8 November 2026: the protocol, the install mockup, the
  dashboard, and the docs state the supported hosts: x64 Linux with
  systemd and cgroup v2, on Debian, Ubuntu, Fedora, RHEL, Rocky Linux, or
  AlmaLinux. Wider support is the follow-up item
  `follow-ups/installer-more-linux-hosts.md`. Rejected: A and B.
- Item 96, A, 8 November 2026: the shared docs grid styles match the
  mockups. Rejected: B and C.
- Item 97, G, 8 November 2026: render with comrak, as a follow-up item that
  blocks nothing. Until then, fix a link test that breaks by hand.
- Item 90, G, 7 November 2026: the deploy workflow builds the
  control-plane image as `mokly-ai/mokly-cloud` does. A build job with no
  cloud credentials builds it, a publish job pushes it by digest, and the
  preview and then production deploy that digest. A push that keeps the
  control-plane sources reuses their image. Rejected: A and B, because the
  user wants no manual step, and C, D, E, and F.
- Item 89, A, 7 November 2026: after an App removal, idle Boat sandboxes
  are not retired early. They end at their lifetime limit, six hours or two
  on a trial, and running jobs finish. The user asked to review B later:
  `follow-ups/boat-guest-retirement.md`. Rejected for now: B, C, and D.
- Item 80, D: an attached Docker disk reserves its data plus the free disk
  of its label. The reservation comes only from spare disk that no set of
  job VMs can use, so a Docker disk stops no start, except for a short time
  after a label or image change. When no reservation fits, the job runs
  without its Docker disk. Machines with little disk get few or no sticky
  disks. Rejected: A, B, C, E, and F. The add-ons G and H were not chosen.
- Item 70, E: this whole repository becomes public now, in place of a public
  copy of the open parts at launch. The agent builds and releases in public.
  This replaces the code location rule of the foundation plan. The closed
  parts keep no open licence, and the README says which directories are
  Apache-2.0. ciproxy runs private repositories only, so Milestone 27 tests
  the CI of a private test repository on ciproxy. `futex-ai/cli-release`
  needs a GitHub token, so the release server keeps one, but it reads public
  repositories only. Rejected: options A, B, C, and D.
- Item 66, A: only a job of a trusted event on the default branch keeps
  its changes to a Docker disk, as item 59 says for the machine cache.
  `cache.md` states it. Rejected: any event on the default branch, and
  `push` jobs only.
- Item 61, A: the user gave the test-mode keys of a Stripe account for
  ciproxy only, "CI Proxy sandbox" (`acct_1UYEdnAGa5h8DTS3`). The secret key
  lives in the secrets folder of the orchestrator and never in the
  repository. Rejected: the account of `futex-ai/accounting`, and no live
  test.
- Item 60, result: on GitHub Free, a runner group limited to one
  repository ran the job of that repository, and the job of the refused
  repository stayed queued with no runner for 101.8 seconds. The probe
  needed a fix of a race in its job check, which Milestone 6 adds.

Design calls for the answers E to item 54 and A to item 55, my calls and those
of the research, 5 November 2026:

- The token must name the repository, the run, and the runner of the assigned
  job. Reason: the token of another job then fails the check.
- Two or more `ac` entries with the write bit make the job unknown. Reason: each
  probe job had at most one.
- The event and the head repository still come from the GitHub API, so the App
  keeps "Actions" read. Reason: the token has no event field.
- The machine cache keeps the scope strings of GitHub unchanged, such as
  `refs/heads/refs/tags/<tag>` for a tag, and compares them exactly with
  `refs/heads/<default branch>`. Reason: a tag can then never match the default
  branch.
- The machine refreshes the public keys of GitHub every 5 minutes, and once more
  for an unknown `kid`, with a limit on refetches. Reason: a false `kid` cannot
  then flood GitHub.
- The machine report counts the unknown jobs by reason, and the token probe
  becomes an operator script that runs before each image release. Reason: a
  change of the token by GitHub then shows at once.

Design calls for the answer A to item 68, my calls, 5 November 2026:

- Shared by both environments: the network, the NAT, the GKE Autopilot cluster,
  and the image registry. Each environment gets its own namespace in the
  cluster. Reason: the user asked for one cluster.
- Separate for each environment: the Cloud SQL instance, the KMS key of the Boat
  keys, the secrets, the buckets, the DNS zones, and the edge with its load
  balancer and certificate. Reason: customer data and production routing never
  share a resource with code that deploys before review, and a small preview
  database costs little.
- The CI identities of each environment get rights only on the resources and the
  namespace of that environment, not owner of the project. Reason: item 58 lets
  only a merge into `main` change production, and an owner of the shared project
  could change production from any push.
- The shared parts apply from the main preview pipeline until the launch switch
  turns on, while production does not exist. After launch, only the production
  pipeline from `main` applies them, and the main preview only reads them.
  Reason: the main preview needs the cluster before launch, and production needs
  the gate of item 58 after it.

Design calls for the answer D to item 56, my calls, 5 November 2026:

- The plan, the check, and the apply run in order in one deploy workflow for
  each environment. Reason: each environment has its own project.
- The apply job is a reusable workflow, and the federation condition checks its
  `job_workflow_ref` claim. Reason: only that file can then use the
  owner-equivalent identity.
- The plan identity gets read roles, and object write only for the lock files.
  Reason: a lock in Cloud Storage needs an object write.
- An approval file names the exact actions of one destructive change, a reason,
  and a digest of the Terraform files, as in `mokly-cloud`. Reason: a stale or
  edited approval then fails.
- The alert address moves to a GitHub variable. Reason: CI applies the
  foundation, and an address is no secret.
- No scheduled drift run. Reason: each merge plans every root, and a new failing
  check needs the user's consent.
- A failed run starts again from a new plan. The operator keeps the recovery
  from a lock-out and the release of a stale lock.

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

No item waits for the user. The items under "Open items of 4 November 2026" at
the end of this plan hold their answers of 5 November 2026. The options check of
items 50 and 51 ran, and its report is in
`.context/ciproxy-mvp/options-check-50-51.md`.

The options check of the `plans` skill ran over these items on
4 November 2026. Its reports are under
`.context/ciproxy-mvp-foundation/`:

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

## Open items of 4 November 2026

The user answered items 54 to 81 on 5 November 2026, and item 76 became a
design call. Item 76 came back on 6 November 2026, and the user chose E.
The user answered items 82 to 86 on 6 November 2026, and item 89 on
7 November 2026.
The conversation gave each item in full in the layout of the
`decisions-and-findings` skill. Each holds one root question,
and its options keep the letters that the user saw.

### 54. The GitHub API cannot name the full ref of a job

**Answer, 5 November 2026:** E.

- **Severity:** High. **Effort:** Medium.
- **Source:** Implementation, Milestone 6, pull request #16.
- **Background:** The run API gives only `head_branch`, a short name. A live
  probe on 4 November 2026 pushed a branch and a tag with the same name, and the
  API data of the two runs was the same. `jobWorkflowRef` of the scale set
  message ended in `@refs/heads/<name>`, `@refs/tags/<name>`, `@refs/tags/<tag>`
  for a dispatch on a tag, `@refs/pull/2/merge`, and `@refs/heads/main` for
  `pull_request_target`. A job in a reusable workflow called at `main` got
  `@refs/heads/main`, also from a tag or a pull request.
- **Options:**
  - **A.** The API only. No push gets a shared write. Effort: Small.
  - **B.** A, and `jobWorkflowRef` for each job that the workflow file of the
    run defines directly, when it agrees with the event and `head_branch`. A job
    in a reusable workflow gets an unknown ref. It reverses the "left out" line
    of item 32. Effort: Medium.
  - **C.** A, and the first hook report for the full ref, which must agree with
    the API. Each runner version needs a repeat test. Effort: Medium.
  - **D.** A, and a test cache entry with the job's token, whose ref GitHub
    reports. Effort: Large.
  - **E.** A, and a check of the runtime token of the job outside the VM: its
    signature, its issuer and expiry, and its run and runner. The scope with
    write access gives the full ref, also in reusable workflows. Effort: Medium.
- **Recommendation:** E, after three live checks. It covers every job with a
  fact that GitHub signs, and it needs no new permission. B is the fallback.
- **Action:** answered E.

### 55. No rule for a job whose ref is unknown

**Answer, 5 November 2026:** A.

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation, Milestone 6.
- **Background:** Each option of item 54 leaves some jobs without a full ref,
  such as a job in a reusable workflow and a pull request from a fork.
  `JobFacts` cannot say "unknown".
- **Options:**
  - **A.** Fail closed: restore default-branch entries from the machine cache
    and other entries through the GitHub cache, save where no other job
    restores, and give the job its own Docker disk copy. Effort: Small.
  - **B.** The machine cache does not handle the job, which uses the GitHub
    cache only and gets its own Docker disk copy. Effort: Small.
  - **C.** Keep the saves private, and promote them when GitHub shows a
    default-branch ref. It needs D of item 54. Effort: Large.
- **Recommendation:** A. It never widens what a job may write, and the job keeps
  its hits on default-branch entries.
- **Action:** answered A.

### 56. Merged Terraform changes still need a person to apply them

**Answer, 5 November 2026:** D.

- **Severity:** High. **Effort:** Large.
- **Source:** Design, the brainstorm of the user's request of 4 November 2026:
  "we should set it up, so that main can deploy terraform changes - I don't want
  to have to do anything manually when in prod".
- **Background:** A person applies the seed, bootstrap, and foundation roots
  from a laptop. The deploy workflow applies only the edge. The seed and the
  first bootstrap need a person once for each environment. The report is
  `.context/terraform-ci/brainstorm-report.md`.
- **Options:**
  - **A.** Widen the deploy run: the deploy identity gets the admin roles and
    applies the foundation. Effort: Medium.
  - **B.** A, and a GCP deny policy on production that refuses the irreversible
    deletes. It needs a GCP organization. Effort: Medium.
  - **C.** Plan on each pull request, and apply on merge. Effort: Large.
  - **D.** Plan, check, and promote: a read-only plan, a policy that needs an
    approval file for each destructive change, an apply of that saved plan by an
    identity that one workflow file can use, then production after the main
    preview. Effort: Large.
  - **E.** Apply outside GitHub Actions, with HCP Terraform or Cloud Build.
    Effort: Large.
- **Recommendation:** D, as in `mokly-cloud`. Add the deny policy of B before
  launch when the projects sit in a GCP organization.
- **Action:** ask the user.

### 57. `main` has no branch protection

**Answer, 5 November 2026:** B.

- **Severity:** High. **Effort:** Small.
- **Source:** Design, the same brainstorm.
- **Background:** `Firna-AI` is on GitHub Free, and the repository is private,
  so GitHub offers no ruleset. Any session with push access can push straight to
  `main`.
- **Options:**
  - **A.** Stay on GitHub Free, and keep "Agent merges: off". Effort: Small.
  - **B.** Move `Firna-AI` to GitHub Team, and add a ruleset on `main` that
    needs a pull request and the gate. Effort: Small.
- **Recommendation:** B. It is the only option where GitHub itself stops an
  unchecked push.
- **Action:** ask the user.

### 58. Production waits for a manual tag

**Answer, 5 November 2026:** B.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design, the same brainstorm.
- **Background:** Milestone 5 deploys production on each `deploy-*` tag, and a
  tag can point at any commit.
- **Options:**
  - **A.** Keep the `deploy-*` tag. Effort: Small.
  - **B.** Deploy on each merge into `main`, after the main preview run of the
    same commit passes, with a variable that keeps production off until launch.
    Effort: Small.
  - **C.** Promote once a day. Effort: Medium.
- **Recommendation:** B. Only merged code runs with production rights, and no
  person acts.
- **Action:** ask the user.

### 59. A workflow can let pull request code write the shared cache

**Answer, 5 November 2026:** C.

- **Severity:** High. **Effort:** Small.
- **Source:** Review of pull request #10, finding 1. Follow-up item
  `cache-runtime-token-write-bypass.md`.
- **Background:** GitHub gives jobs of untrusted events read-only shared caches,
  unless the workflow sets `cache-mode: write`. The job VM holds the runtime
  token. Only 7 events write default-branch entries on GitHub, and `cache.md`
  names only `pull_request_target` as untrusted.
- **Options:**
  - **A.** Require read-only cache rights for these jobs, which needs workflow
    edits. Effort: Small.
  - **B.** Keep the runtime token outside the job VM, and forward its calls.
    Effort: Large.
  - **C.** Match GitHub's rule: only the 7 trusted events write shared entries
    in the machine cache, and a direct write that a workflow opted into follows
    GitHub's rule. Effort: Small.
- **Recommendation:** C. ciproxy cannot be safer than the GitHub cache that it
  writes through, and the risk needs the customer's choice.
- **Action:** ask the user.

### 60. Create a second test repository

**Answer, 5 November 2026:** A.

- **Severity:** Low. **Effort:** Small.
- **Source:** Implementation, the live probe of Milestone 6.
- **Background:** A Free organization created a runner group limited to one
  repository. The check that GitHub keeps a refused repository away needs a
  second private repository.
- **Options:**
  - **A.** Create `Firna-AI/ciproxy-spike-refused`, run `runner_group_probe.py`,
    and delete it. Effort: Small.
  - **B.** As A, and keep the repository. Effort: Small.
  - **C.** Skip the check. Effort: Small.
- **Recommendation:** A. It proves the limit and leaves nothing behind.
- **Action:** ask the user.

### 61. No Stripe test-mode key for the live billing test

**Answer, 5 November 2026:** A, the account "CI Proxy sandbox".

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, Milestone 17.
- **Background:** `crates/ciproxy-billing/scripts/stripe-test-mode.sh` needs a
  test secret key and a webhook signing secret.
- **Options:**
  - **A.** A Stripe account or sandbox for ciproxy. Effort: Small.
  - **B.** A test key of the Stripe account of `futex-ai/accounting`. Effort:
    Small.
  - **C.** Skip the live test until the main preview runs with Stripe. Effort:
    Small.
- **Recommendation:** A. It keeps ciproxy in its own Stripe account from the
  start.
- **Action:** ask the user.

### 62. The install command does not prepare the machine

**Answer, 5 November 2026:** C.

- **Severity:** High. **Effort:** Large.
- **Source:** Implementation, Milestone 9, draft pull request #12.
- **Background:** The install sets up no Firecracker, no jailer, no host tools,
  and no disk area with reflinks. ext4, the default of Ubuntu and Debian, has no
  reflinks.
- **Options:**
  - **A.** The customer prepares them first, which breaks the one-command
    promise. Effort: Small.
  - **B.** The install sets up the tools, and uses full copies where the file
    system has no reflinks. Effort: Medium.
  - **C.** The install sets up the tools and a reflink disk area on any file
    system: one file formatted as XFS. Effort: Large.
  - **D.** B now, and C as a follow-up item after launch. Effort: Medium.
- **Recommendation:** C. B would make the slow path the common case on Ubuntu
  and Debian.
- **Action:** ask the user.

### 63. No rule for a day on which a price changes

**Answer, 5 November 2026:** B.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, Milestone 16.
- **Background:** The savings report counts monthly costs "for the days of the
  range". The merged code marks such a day as unknown.
- **Options:**
  - **A.** The first price counts for the whole day. Effort: Small.
  - **B.** The last price counts for the whole day. Effort: Small.
  - **C.** Split the cost of the day by the hours of each price. Effort: Small.
  - **D.** Count only the time used, for every monthly cost. Effort: Medium.
- **Recommendation:** C. It counts each price for the time that it applied.
- **Action:** ask the user.

### 64. No rule for how a monthly price becomes a cost per day

**Answer, 5 November 2026:** B.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, Milestone 16.
- **Background:** The report covers the last 30 days, this calendar month, or
  the last calendar month.
- **Options:**
  - **A.** 30 days. Effort: Small.
  - **B.** The days of the calendar month. Effort: Small.
  - **C.** 365 divided by 12 days. Effort: Small.
  - **D.** 30 days in the 30-day view, and calendar days in the calendar views.
    Effort: Small.
  - **E.** The length of each billing period. Effort: Medium.
  - **F.** Calendar days for machines, and billing period days for the tier.
    Effort: Medium.
- **Recommendation:** B. Each day has the same cost in every view. The Savings
  mockup then needs a small fix.
- **Action:** ask the user.

### 65. Merge `futex-ai/sandbox` #11

**Answer, 5 November 2026:** B. Merged as `9658361`.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, Milestone 12, part 1.
- **Background:** #11 adds the capability split and the Boat backend of item 49.
  Its review findings are fixed. That repository says "Agent merges: off".
- **Options:**
  - **A.** The user merges #11. Effort: Small.
  - **B.** The user allows the orchestrator to merge it once. Effort: Small.
- **Recommendation:** A. It keeps the merge rule of that repository.
- **Action:** ask the user.

### 66. Untrusted events can keep Docker disk changes

**Answer, 5 November 2026:** A.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Review of pull request #17, outside the scope.
- **Background:** "Docker disks" in `cache.md` lets the first job on the
  default branch keep its changes to the Docker disk of a repository,
  whatever its event. Item 59 = C limits shared machine-cache entries to the
  7 trusted events.
- **Options:**
  - **A.** Only a job of a trusted event on the default branch keeps its
    changes. Effort: Small.
  - **B.** Keep the current rule. Effort: Small.
  - **C.** Only a `push` job on the default branch keeps its changes.
    Effort: Small.
- **Recommendation:** A. One rule then covers the machine cache and the
  Docker disks.
- **Action:** ask the user.

### 67. The spend limit has no rule for unconfirmed costs

**Answer, 5 November 2026:** B.

- **Severity:** High. **Effort:** Medium.
- **Source:** Review of draft pull request #22, Milestone 12.
- **Background:** The Boat API gives the dollars of each sandbox through
  `GET /sandboxes/{id}/usage`, and the next start fee through `GET /limits`.
  The hourly prices of the sizes come from the published price list. A cost
  stays unknown after a start with no clear answer, a failed usage read, or a
  start fee that differs from its quote.
- **Options:**
  - **A.** Hold all new starts until Boat confirms. Effort: Small.
  - **B.** Count the cost bound of each unconfirmed start until Boat
    confirms. Effort: Medium.
  - **C.** An Admin reconciles each unclear cost by hand. Effort: Large.
  - **D.** As B, with one figure that includes the bound. Effort: Medium.
  - **E.** An add-on: recovery of old costs through a replacement key.
    Effort: Medium.
- **Recommendation:** B. Uncounted costs never pass the limit, and overflow
  keeps running while the limit has room.
- **Action:** answered B.

### 68. Which GCP project holds the main preview

**Answer, 5 November 2026:** A, with one shared GKE cluster.

- **Severity:** High. **Effort:** Large, for the answer.
- **Source:** The user's message "we should be using ciproxy project for the
  deployment".
- **Background:** The plan placed the main preview in `internal-498318` and
  production in `ciproxy`, each with its own foundation and apply identity.
- **Options:**
  - **A.** Both environments in `ciproxy`. Effort: Medium.
  - **B.** A new project for the main preview, such as `ciproxy-preview`.
    Effort: Small.
  - **C.** Keep the main preview in `internal-498318`. Effort: none.
- **Recommendation:** B. It keeps the main preview away from production.
- **Action:** answered A, with one shared cluster. The design calls keep
  production apart through scoped identities.

### 69. A start-fee quote does not bound the start fee

**Answer, 5 November 2026:** B.

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
- **Source:** Review of pull request #22, Milestone 12, finding 1. The
  review graded it High. It is Medium here, because one start can hide at
  most $0.07.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-overflow/src/service/start.rs` and
  `crates/ciproxy-overflow/src/spend.rs` on the branch of #22, and the spend
  rule in `docs/protocol/overflow.md`.
- **Terms:**
  - **Start fee:** the price that Boat charges for one sandbox start. It is
    $0 inside the free starts of the plan, then $0.03, $0.05, or $0.07.
  - **Quote:** the field `nextStartDollars` of `GET /limits`. It gives the
    price of the next start at the moment of the read.
- **Background:**
  - Item 67 = B counts a bound for each start whose cost Boat has not
    confirmed. The bound is the hourly price for the whole lifetime, plus
    the quote that ciproxy saves before the start.
  - Boat counts free starts in rolling windows of one minute, one hour, and
    one day. A start pays the highest bracket of the three windows.
  - Boat reports no charged start fee. The create response, the usage, and
    the sandbox object have no such field. So the saved quote stays the
    counted fee for good.
  - ciproxy starts one sandbox at a time for each workspace. Other users of
    the same Boat account can start sandboxes at any time.
  - A 10-minute job on the default size costs about $0.006 of runtime, at
    the list price of $0.036 for each hour.
- **Problem:** The quote is not a reservation. When another Boat user starts
  a sandbox between the quote and the start, the start can cost one bracket
  more than its quote. ciproxy then counts the lower quote.
- **Impact if no action:** The spend can pass the limit by up to $0.07 for
  each such start. This needs other use of the Boat account near the edge
  of a bracket.
- **Root question:** What start fee does ciproxy count, when Boat never
  reports the fee that it charged?
- **Options:**
  - **A.** Keep the saved quote. State in `overflow.md` that the quote
    stands for the fee, and why. A start that another Boat user pushes into
    a higher bracket counts too little. Effort: Small.
  - **B.** Count the highest published bracket for each start. Read it from
    `overagePrices.startBrackets` of `GET /limits`. It is $0.07 today. This
    is a true bound. Each free start then also counts $0.07 for good, about
    12 times the runtime of a 10-minute job. The limit then stops overflow
    much earlier than the real Boat bill does. Effort: Small.
  - **C.** Read the quote again directly after Boat accepts the start.
    Count the higher of the two quotes. When another Boat user starts a
    sandbox in the gap, the second quote is also higher. So the count
    catches nearly every such start. It is not a strict bound, because old
    starts can leave a rolling window between the two reads. It adds one
    read for each start. Effort: Small.
  - **D.** Ask Boat to report the charged start fee of each sandbox. Count
    that fee as confirmed when Boat gives it, and use C until then. It
    depends on work and a timeline at Boat. Effort: Small in ciproxy.
- **Left out:**
  - Separate quoted, bounded, and confirmed states, from the review: Boat
    never confirms a start fee, so each start stays bounded for good. That
    is B.
  - A bound of the highest bracket while the start runs, which later
    settles to the quote: the month total is then the same as in C.
  - One lock for all users of the Boat account: ciproxy cannot control
    other Boat clients.
- **Wider fix:** D. A confirmed fee from Boat removes the estimate.
- **Previous examples:** None found. Commit `75bd853` records the quote in
  the answer to item 67.
- **Related:** item 67. Its recorded answer names the saved quote.
- **Recommendation:** C. It catches nearly every such start for one extra
  read, and it does not count $0.07 for each free start as B does. D best
  protects the codebase, but it needs Boat.
- **Action:** answered B.

### 70. The agent binaries stay private after launch

**Answer, 5 November 2026:** E.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design. The user wrote "probably the cli needs to be open
  source, if people are going to install it on their machine", and then "i
  think just make the repo public?". Options D and E come from that message.
- **Category:** Docs or spec.
- **Location:** Milestone 27 of this plan, the release workflow of
  `ciproxy-agent`, and step 4 of `deploy/setup.md`.
- **Terms:**
  - **Public copy:** the public repository that gets the open parts at
    launch. The user creates it in Milestone 27.
  - **Release server:** the `futex-ai/cli-release` server at
    `get.ciproxy.dev`. It serves the install script and the agent binaries.
  - **Build attestation:** a signed GitHub record that names the workflow
    and the commit that built a file.
- **Background:**
  - Item 20 = F made every part on the customer's side Apache-2.0: the
    agent, its protocol, the image definition, the install script, the
    cache sidecar, and the runner patch.
  - This repository stays private. At launch, the open parts go public from
    a public copy. The foundation plan rejected a public copy from the first
    push, because early code would go public before launch.
  - The release workflow builds the agent in this private repository and
    uploads the binaries to its releases. The release server reads them
    with the release token.
  - The plan does not say where the binaries come from after launch.
- **Problem:** After launch, customers can read the source in the public
  copy. But the binary that they install still comes from a private build,
  and they cannot check it against the public source.
- **Impact if no action:** The claim "Read and audit the code that runs on
  your machines" holds for the source, but not for the binary. The release
  server keeps a token for the private repository.
- **Root question:** After launch, how can a customer trust that the agent
  binary comes from the public source?
- **Options:**
  - **A.** Keep the plan. The binaries stay in the private releases, and
    the release server keeps the token. A customer can build the agent from
    the public source. Effort: none.
  - **B.** At launch, the private release workflow also publishes each
    release in the public copy, with a build attestation. The release
    server reads the public releases, so it needs no access to private
    code. The attestation names a private commit. So it proves where the
    binary was built, but not that it matches the public source. Effort:
    Small.
  - **C.** At launch, the release workflow of the agent moves to the public
    copy. The public copy builds and publishes each release from its public
    source. The attestation then names a public commit. This repository
    only pushes the source of the open parts to the public copy. The
    release server needs no access to private code. Effort: Medium.
  - **D.** At launch, make this whole repository public, in place of the
    public copy. The agent builds and releases in public, so no token and no
    copy step stay. Everyone can then read the paid control plane, the
    dashboard, the plans, and the full history. The closed parts keep no
    open licence. ciproxy runs private repositories only, so its own CI moves
    to GitHub-hosted runners after launch. This reverses the code location
    rule of the foundation plan. It cannot be undone. Effort: Small.
  - **E.** As D, but now. The token and the copy step go away now. Early code
    goes public before launch, and ciproxy can no longer test its own CI on
    ciproxy runners before launch. Effort: Small.
- **Left out:** Reproducible builds, so that anyone can rebuild the same
  bytes: C and D give the same proof with much less work.
- **Wider fix:** D.
- **Previous examples:** None found.
- **Related:** The main preview needs the release token now under every
  option, because the public copy exists only from launch.
- **Recommendation:** D. It gives a public build with no token and no copy
  step, and the private repository still serves the CI test before launch.
  C best keeps the paid code private.
- **Before D or E:** scan the whole history for secrets, check each workflow
  for pull requests from forks, and remove what must stay private.
- **Action:** answered E.

### 71. Leftover Boat test snapshots cost money

**Answer, 5 November 2026:** A.

- **Severity:** Low. **Effort:** Small.
- **Source:** Implementation, during the live Boat check of #22.
- **Background:** The Boat wallet "Firna" kept 4 snapshots of one stopped
  sandbox, `bx_h6amcqtu`, from 00:58 to 01:02 UTC on 5 November 2026.
- **Options:**
  - **A.** Delete the stopped sandbox and its snapshots. Effort: Small.
  - **B.** Keep them. Effort: none.
- **Recommendation:** A.
- **Action:** answered A. Boat refused the deletion with
  `api_key_action_forbidden`, because the test key did not create that
  sandbox. The user deletes it in the Boat dashboard, or asks the owner of
  the other key.

### 72. Production DNS expects Cloud DNS, but the domain uses Cloudflare

**Answer, 5 November 2026:** B.

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation. A public DNS lookup on 5 November 2026.
- **Category:** Docs or spec.
- **Location:** `deploy/terraform/modules/foundation/variables.tf`, the
  `dns_domains` of production, and step 5 of `deploy/setup.md`.
- **Background:**
  - `ciproxy.dev` uses the Cloudflare name servers. Cloudflare holds the
    Google Workspace mail records of the domain: MX, SPF, and a site
    verification. The alert address `devops-alerts@ciproxy.dev` needs them.
  - The main preview creates three Cloud DNS zones under `preview`. Three NS
    records in the parent zone delegate them, and Cloudflare can hold these
    records too.
  - Production creates one Cloud DNS zone for the whole domain, and
    Terraform adds the site, app, and release records there. That zone
    answers only when the domain uses the Google name servers.
- **Problem:** The production zone never answers, because the registrar
  points `ciproxy.dev` at Cloudflare.
- **Impact if no action:** The production hosts do not resolve after the
  production deploy. A careless move of the domain breaks its mail.
- **Root question:** Which DNS service serves `ciproxy.dev` in production?
- **Options:**
  - **A.** Move `ciproxy.dev` to Cloud DNS. Terraform copies the mail
    records into the production zone first. Then the user changes the name
    servers at the registrar from Cloudflare to Google. Terraform then owns
    every record. A missed record can stop mail for a while. Effort: Small.
  - **B.** Keep Cloudflare. Production creates Cloud DNS zones for
    `app.ciproxy.dev` and `get.ciproxy.dev`, and NS records in Cloudflare
    delegate them. The site record of `ciproxy.dev` stays a record that the
    user adds in Cloudflare by hand. Effort: Medium.
  - **C.** Keep Cloudflare, and Terraform manages the records there with the
    Cloudflare provider and an API token in CI. It adds a provider and a
    secret. Effort: Medium.
- **Left out:** A second domain for production: the site, the docs, and the
  App already use `ciproxy.dev`.
- **Wider fix:** A. One DNS service under Terraform.
- **Previous examples:** None found.
- **Related:** Under every option, the main preview works now: the user adds
  three NS records in Cloudflare after the foundation applies.
- **Recommendation:** A. Terraform then owns every production record, as the
  user asked for no manual production step, with no new provider or secret.
- **Action:** answered B.

### 73. The full image leaves a job no free disk

**Answer, 5 November 2026:** A.

- **Severity:** High. **Effort:** Medium.
- **Source:** Implementation, Milestone 10, the full KVM image build of
  5 November 2026.
- **Category:** Docs or spec.
- **Location:** The label table of `docs/protocol/labels.md`, the machine
  minimum of `docs/protocol/agent.md`, and
  `crates/ciproxy-agent/image-budget.md`.
- **Terms:**
  - **Image budget:** the disk of an own machine that holds the job images,
    apart from the job disk.
  - **Root reserve:** the part of an ext4 disk that only root can use. Its
    default is 5%.
- **Background:**
  - `agent.md` requires the software of the GitHub image that
    `ubuntu-latest` names. The build of that full set passed all 502
    upstream checks. It uses 58.3 GB, and its download is 18.3 GB.
  - The largest parts are the Android SDK with 11.8 GB, .NET with 6.2 GB,
    the tool cache with 5.6 GB, Haskell with 3.9 GB, Swift with 3.7 GB, and
    the cached Docker images with 2.7 GB.
  - With the default root reserve, a job disk of 60 GB leaves a job 0 GB,
    75 GB leaves 11.2 GB, and 80 GB leaves 15.9 GB. With a reserve of 1%,
    75 GB leaves 14.2 GB, and 80 GB leaves 19.1 GB.
  - A standard GitHub-hosted Linux runner promises 14 GB of free disk, on a
    disk of about 72 GB.
  - Blacksmith boots the same image as GitHub, in Firecracker microVMs. Its
    runner with 2 vCPU and 8 GB has 80 GB of disk, and its runner with
    8 vCPU has 160 GB. Source: the Blacksmith instance types page, read on
    5 November 2026.
  - The label `ciproxy` has 60 GB of disk. An own machine needs 80 GB of
    disk, and its image budget is 4 GB.
  - Boat `small` and `default` have 51 GB, so they cannot hold the full
    image. The label already needs Boat `large`, with 111 GB, because of
    its 60 GB.
- **Problem:** A job on the label `ciproxy` gets no free disk, and an own
  machine has no room for the image.
- **Impact if no action:** Every job on an own machine fails for lack of
  disk.
- **Root question:** How does a job get the software of `ubuntu-latest` and
  enough free disk on an own machine?
- **Options:**
  - **A.** Keep the full image. Raise the label disk to 80 GB with a root
    reserve of 1%, so a job keeps about 19 GB free. Size the image budget
    for two images and one download during an update, about 135 GB, apart
    from the job disk. The machine minimum rises from 80 GB to 250 GB of
    disk. Boat stays on `large`. Effort: Medium.
  - **B.** Slim the image. Leave out the parts that a setup action installs
    during the job: Android, Haskell, Swift, Julia, and the cached Docker
    images. This saves about 23 GB. The 60 GB label then keeps about 23 GB
    free, and the machine minimum is about 150 GB. A workflow that uses a
    removed tool without a setup step needs one, which breaks the software
    rule of `agent.md`. Effort: Medium.
  - **C.** Share one read-only copy of the full image across the job VMs,
    and give each job its own writable layer of 20 GB. The job disk then
    counts only the layer. The machine minimum is about 170 GB, and a
    machine fits more jobs. It changes the disk model of the merged agent.
    Effort: Large.
- **Left out:**
  - Keep 60 GB and the full image: a job gets no free disk.
  - A full and a slim image under two labels: each machine then stores and
    updates both.
- **Wider fix:** C. It counts the image once for all jobs.
- **Previous examples:** None found.
- **Related:** The final disk of Milestone 10 waits for this answer.
- **Recommendation:** A. It keeps the software rule and the agent design,
  it gives the same disk as Blacksmith for the same runner size, and 250 GB
  is small for a CI server. C saves the most disk, at a large cost.
- **Action:** answered A.

### 74. Preview pods can read production logs

**Answer, 5 November 2026:** A.

- **Severity:** High. **Effort:** Small.
- **Source:** Review of pull request #24, Milestone 5, finding 1.
- **Category:** Security.
- **Location:** `deploy/scripts/namespace_access.py`, the namespaces of the
  shared cluster.
- **Terms:**
  - **PodSecurity:** the Kubernetes check that refuses pods whose settings
    break a standard security level.
  - **hostPath:** a pod volume that reads files of the cluster machine.
- **Background:**
  - Item 68 put both environments in one GKE Autopilot cluster, each in its
    own namespace.
  - A push to the primary branch deploys the main preview, so the preview
    deploy identity can change the pods of the preview namespace.
  - Autopilot allows read-only `hostPath` volumes under `/var/log`.
- **Problem:** Neither namespace enforces a PodSecurity level. A preview pod
  on a machine that also runs production can mount `/var/log` and read the
  container logs of production.
- **Impact if no action:** Unreviewed preview code can read production logs.
  Network policy and the scoped identities do not stop a local file mount.
- **Root question:** How does the shared cluster keep production files away
  from preview pods?
- **Options:**
  - **A.** Enforce the Baseline PodSecurity level on both namespaces, with
    labels that only the operator can change. Baseline refuses `hostPath`
    volumes. Effort: Small.
  - **B.** Add a custom admission policy that refuses `hostPath` volumes. It
    keeps the other Autopilot defaults, but adds policy code to maintain.
    Effort: Medium.
  - **C.** Replace the free deployment writes with a trusted controller that
    accepts only image and configuration changes on fixed pod templates.
    Effort: Large.
- **Left out:** Separate clusters: item 68 chose one. Network rules: they
  cannot stop a local file mount. Rules in the committed Helm charts: a
  preview push can change the charts.
- **Wider fix:** A. One standard admission level for every workload.
- **Previous examples:** None found.
- **Recommendation:** A. It closes the hole with a maintained Kubernetes
  level and no new code.
- **Action:** answered A.

### 75. The AX42 cost claim needs a disk basis

**Answer, 5 November 2026:** A.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, the workspace of item 73.
- **Category:** Docs or spec.
- **Location:** the hardware fixtures of the site and the mockups, and the
  pricing calculation text of the site.
- **Terms:**
  - **RAID1:** two mirrored disks that give the space of one disk.
  - **RAID0 or one LVM volume:** two disks combined into the space of both,
    with no mirror.
- **Background:**
  - The cost claim of the site counts a Hetzner AX42, with 16 threads,
    64 GB, and two disks of 512 GB, that runs six default jobs at a time.
  - Item 73 = A gives each default job 80 GB and the machine an image
    budget of 140 GB. Six jobs then need one filesystem of 780 GB.
  - The Hetzner installer mirrors two disks by default, with RAID1. That
    gives about 512 GB, which runs three default jobs.
- **Problem:** The six-job claim now needs both disks in one filesystem,
  which is not the default setup, and no setup text says so.
- **Impact if no action:** A customer with the default mirror runs three
  jobs, and saves 65% in this calculation instead of the claimed 80%.
- **Root question:** Which disk setup of the AX42 do the claim and the
  fixtures count?
- **Options:**
  - **A.** Keep 1,024 GB and six jobs. The site and the mockups state that
    the example combines both disks, for example with RAID0 or one LVM
    volume. The machine setup docs say that a machine used only for CI can
    combine its disks, because jobs, the machine cache, and Docker disks
    are disposable. Effort: Small.
  - **B.** Count the default mirror of 512 GB: three jobs and a saving of
    65%. Update every AX42 scene to show the disk limit. Effort: Medium.
  - **C.** Count another machine with one disk of at least 1 TB. Check its
    price, and update the claim and the fixtures. Effort: Medium.
- **Left out:** Leave the setup unstated: it hides why a default install
  runs half the jobs. Combine the disks in the agent: it is outside item 73.
- **Wider fix:** One stated basis for the claim, used by the site and the
  mockups.
- **Previous examples:** With 60 GB jobs, six jobs fit in 512 GB, so the old
  claim did not depend on the setup.
- **Recommendation:** A. It keeps the machine and the story of the fixtures,
  and it tells the customer how to get the claimed result.
- **Action:** answered A.

### 76. The organization forbids the public site buckets

**Outcome, 5 November 2026:** D, a design call after the user asked why the
buckets must be public.

**Answer, 6 November 2026:** E. The first edge deploy showed that the rule
refuses D's grant, so the item came back with A, B, C, and the new E and F.

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation, the read-only checks before the GCP setup.
- **Category:** Process.
- **Location:** the organization policy `iam.allowedPolicyMemberDomains` of
  the futex.ai organization, and the public buckets of the foundation.
- **Terms:**
  - **Domain-restricted sharing:** the organization policy that allows IAM
    grants only to members of the futex.ai Google Workspace.
  - **`allUsers`:** the IAM member for everyone on the internet.
- **Background:**
  - The site and the dashboard are static files in buckets. The load
    balancer serves them through Cloud CDN, so these buckets grant read
    access to `allUsers`.
  - The futex.ai organization enforces domain-restricted sharing, set on
    21 April 2026. Every project inherits it, also `ciproxy`.
  - The user has the Organization Administrator role. That role can grant
    the Organization Policy Administrator role, which changes the policy.
- **Problem:** The first apply of the foundation fails when it grants read
  access to `allUsers`.
- **Impact if no action:** The site and the dashboard cannot go online.
- **Root question:** How can the public files be served while the
  organization keeps its sharing rule?
- **Options:**
  - **A.** Allow all members in the project `ciproxy` only, with a project
    override of the policy. Other projects keep the rule. Any resource in
    `ciproxy` can then get a public grant, and only Terraform writes grants
    there. Effort: Small.
  - **B.** Keep the rule, and allow all members only on resources with a
    tag, for example `public-access`. Tag only the site and dashboard
    buckets in Terraform. It needs the Tag Administrator role and a
    condition in the organization policy. Effort: Medium.
  - **C.** Serve the static files from a small web server in the cluster, so
    no bucket needs a public grant. It adds a workload and changes the
    deploy design. Effort: Medium.
  - **D.** Use the private bucket access of Cloud CDN. The buckets stay
    private, and only the load balancer service agent
    `service-PROJECT_NUMBER@https-lb.iam.gserviceaccount.com` reads them.
    Visitors still get the files through the load balancer and the CDN.
    The organization rule stays. Effort: Small.
  - **E.** In the project `ciproxy`, replace the rule with the managed rule
    `iam.managed.allowedPolicyMembers`. It allows the members of the
    futex.ai organization and, by name, the load balancer agent and the CDN
    fill agent of the project. Effort: Small.
  - **F.** Lift the rule in `ciproxy` for a few minutes, let the deploy make
    the grants, and restore the rule. Each new grant needs the same step.
    Effort: Small.
- **Left out:** Signed URLs for the whole site: browsers cannot fetch a
  public site with them.
- **Wider fix:** B. Only the resources with the tag can be public.
- **Previous examples:** None found. The projects `mokly-508518` and
  `internal-498318` keep the rule with no exception.
- **Recommendation:** D. It keeps the organization rule and the deploy
  design, and no one can read the buckets directly.
- **Action:** design call D. It changes no requirement and needs no change
  outside the project. Google creates the service agent after the first
  backend bucket, so the grant follows the backend buckets. If the
  organization rule refuses the grant, the item returns to the user with A,
  B, and C.

### 77. A label can be smaller than the job image

**Answer, 5 November 2026:** D.

- **Severity:** Medium. **Effort:** Large, for the answer.
- **Source:** Implementation, pull request #25.
- **Record:** the follow-up item
  [labels smaller than the job image](https://github.com/Firna-AI/ciproxy/blob/6b28eb6/plans/follow-ups/labels-smaller-than-job-image.md)
  holds the background, the options A to D, and the recommendation.
- **Recommendation:** A. A job of such a label runs on the image disk size,
  the scheduler reserves that size, and the label dialog warns.
- **Action:** answered D: the disk of a label is the free disk of its jobs.

### 78. How a workspace picks the software that its jobs start with

**Answer, 5 November 2026:** C, and A with B after launch.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design. The user asked "how would you suggest allowing the
  user to pick different base images?".
- **Options:**
  - **A.** A catalog of ciproxy images, `full` and `slim`, picked per label.
    Effort: Large.
  - **B.** A, plus custom VM images on Business. Effort: Large.
  - **C.** One image, and job containers with `container:` for other
    software. Effort: Small.
  - **D.** Toolsets on a common base. Effort: Large.
- **Recommendation:** A after launch, with C as docs now.
- **Action:** answered C. The follow-up item
  [job image catalog after launch](follow-ups/job-image-catalog-after-launch.md)
  holds A and B.

### 79. Removing a machine can cancel a job that a runner just took

**Answer, 5 November 2026:** A, in the user's words. See the answers above.

- **Severity:** High. **Effort:** Large.
- **Source:** Implementation, Milestone 13.
- **Category:** Docs or spec.
- **Location:** `crates/ciproxy-guest/src/process.rs`,
  `crates/ciproxy-guest/src/service.rs`, and `crates/ciproxy-agent/src/mode.rs`
  on the Milestone 13 branch.
- **Background:**
  - "Remove" revokes the machine credential at once. The agent then stops
    its warm runners and lets running jobs finish. `ciproxy-agent
uninstall` does the same without the control plane. After the last
    organization removes the App, each machine of the workspace acts as
    removed.
  - Ordinary stops and Drain ask GitHub to delete the runner first, and a
    busy answer keeps the runner and its job.
  - The guest learns of a job only when the patched runner worker hands it
    over, just before the first step. GitHub gives the job earlier.
- **Problem:** On Remove, uninstall, or the last App removal, the agent
  stops each runner that it still sees as warm. The runner then cancels a
  job that it took before the handover.
- **Impact if no action:** Removal can cancel a real job, which breaks the
  rule that a removed machine lets its running jobs finish.
- **Root question:** How does ciproxy stop the idle runners of a leaving
  machine or organization without stopping a job that GitHub already gave
  to one of them, also offline and with a revoked credential?
- **Options:**
  - **A.** Keep the immediate stop, and record that a job that the guest has
    not seen can be cancelled. It weakens a protocol rule. Effort: Small.
  - **B.** Revoke normal access at once, and stop each remaining VM only
    after GitHub's removal answer, with a narrow retirement path for an
    offline or revoked agent and for uninstall. It adds a trust boundary at
    the gateway, and it gets no answer after the App removal. Effort: Large.
  - **C.** Require a finished Drain before Remove or uninstall. An offline
    machine cannot finish Remove until it returns. Effort: Medium.
  - **D.** At Remove, the control plane deletes the GitHub registration of
    each runner, also while the machine is offline. A deleted idle runner
    exits by itself, and the agent waits for each VM to end. It does not
    cover the App removal. Effort: Large.
  - **E.** Extend the runner patch to the listener: on a stop request, it
    stops asking for jobs, runs a job that it already holds, and then
    exits. Every caller is then safe, also the App removal. It needs a live
    check of how GitHub treats a job at the stop. Effort: Large.
- **Left out:** a grace timeout or a process probe, because neither proves
  that GitHub gave no job; a delayed credential revocation; a GitHub
  removal token on the machine; cutting the network; and re-running a
  cancelled job.
- **Wider fix:** E.
- **Recommendation:** E. The stop itself can then never cancel a taken job.
- **Action:** ask the user, because E needs a live check at the GitHub
  boundary, and B adds a retirement boundary.

### 80. Sticky Docker disks have no space rule

**Answer, 5 November 2026:** D.

- **Severity:** Medium. **Effort:** Large.
- **Source:** Implementation, Milestone 13.
- **Category:** Docs or spec.
- **Location:** "Docker disks" in `docs/protocol/cache.md`, and "Disk" and
  "Fit" in `docs/protocol/job-routing.md`.
- **Background:**
  - A job VM starts only when the free job disk covers its whole root disk:
    the image, the free disk of its label, and the ext4 allowance.
  - Docker disks share the free job disk with the machine cache, and unused
    Docker disks go first when a VM needs space. The first trusted job on
    the default branch keeps its changes, and each other job works on a
    copy. A job runs without its Docker disk when the disk cannot attach.
- **Problem:** No rule says how much space an attached Docker disk holds,
  or whether that space can keep a job VM from starting. An attached disk
  cannot go during its job.
- **Impact if no action:** An unbounded disk can take the space of a VM
  root, so a job gets write errors.
- **Root question:** How much space does the sticky Docker data of a
  running job hold, and from which space, so that every job VM keeps its
  promised root?
- **Options:**
  - **A.** A lease of the disk's data plus the label's free disk, from the
    free job disk. Count leases in placement, and skip the disk when no
    lease fits. A Docker disk can then delay a job VM. Effort: Medium.
  - **B.** Attach the repository disk read-only, keep each job's Docker
    writes copy-on-write on its own root, and publish the trusted writer's
    changes after its job. Copies then hold no new space. Effort: Large.
  - **C.** A machine Docker budget setting and a size for each repository
    disk, taken out of the job disk. A budget above zero stops the 250 GB
    minimum from fitting one default job. Effort: Large.
  - **D.** A's lease, but only from spare disk that no set of job VMs can
    use. Leases then stop no start, except for a short window after a label
    or image change. Disk-limited machines get few or no sticky disks.
    Effort: Large.
  - **E.** Each VM of a workspace with sticky disks holds a fixed Docker
    allowance from its start. The 250 GB minimum then fits no default job.
    Effort: Medium.
  - **F.** B's read-only disk for copies, and a writable disk under D's
    lease for the trusted writer only. Effort: Large.
  - **G.** An add-on to A, D, E, or F: ship the image's cached Docker
    images as a seed disk with each release, about 6 GB of the 140 GB image
    budget. Effort: Medium.
  - **H.** An add-on to A, D, E, or F: one host ledger owns the whole free
    job disk, for VM disks, Docker disks, and the machine cache. Each start
    and each attach takes its space from that ledger. Effort: Large. The
    re-print of 5 November 2026 gave the wider fix this letter.
- **Left out:** unbounded disks, no sticky disks, space from the host
  reserve or the image budget, and a registry pull-through cache.
- **Wider fix:** one host ledger that owns the job disk for roots, Docker
  space, and the cache.
- **Recommendation:** D. It gives the same sticky disks as A on
  slot-limited machines such as the AX42, and its leases delay a start only
  shortly after a change. B removes the copy space best, at Large effort.
- **Action:** ask the user, because A and D change the start rule of
  `job-routing.md`, and no requirement gives the Docker space of a job.

### 81. Which region hosts ciproxy

**Answer, 5 November 2026:** C.

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Implementation, the failed cluster creation in `europe-west1`.
- **Options:**
  - **A.** Retry `europe-west1`. Effort: Small.
  - **B.** Switch to `europe-west4`, and keep the data in the EU. Effort:
    Medium.
  - **C.** Switch to `us-central1`, about 10% cheaper, with the data in the
    US. Effort: Medium.
- **Recommendation:** A, then B.
- **Action:** answered C.

### 82. Adding an organization can pick the wrong workspace

**Answer, 6 November 2026:** B, in the user's words. See the answers above.

- **Severity:** High. **Effort:** Medium.
- **Source:** Design, Milestone 14, pull request #30, its item 1.
- **Category:** Security.
- **Location:** "Add an organization" in `docs/protocol/workspaces.md`, and
  `crates/ciproxy-github/src/app/installation.rs:84`.
- **Background:**
  - An Admin starts an add attempt in one workspace. GitHub's install page
    then lets the person choose the organization.
  - GitHub can send the installation event before the browser returns.
  - The event names the person and the organization, not the add attempt.
- **Problem:** ciproxy cannot always tell which workspace the organization
  was meant for: after a late owner approval, or with two attempts at once.
- **Impact if no action:** An organization can join the wrong workspace.
  "Add an organization" stays off.
- **Root question:** How does ciproxy know the workspace of each install?
- **Options:**
  - **A.** The Admin picks the organization in ciproxy before GitHub opens.
    ciproxy saves the workspace and the organization, then opens the
    install page for that organization only. A new attempt for the same
    organization replaces the old one. The protocol and the mockup change.
    Effort: Medium.
  - **B.** Keep the choice on GitHub, match the browser's return to the
    waiting event, and add a recovery screen. Effort: Large.
  - **C.** Keep the choice on GitHub, save installs with no workspace, and
    add screens to assign them. Effort: Large.
- **Left out:** One waiting workspace per person; the latest attempt; no
  change.
- **Wider fix:** A.
- **Recommendation:** A. It gives the exact link with the least new code.
- **Action:** ask the user, because A changes where the protocol lets the
  person choose the organization.

### 83. A late approval has nowhere to go

**Answer, 6 November 2026:** D, which the user proposed. See the answers
above.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, Milestone 14, pull request #30, its item 2.
- **Category:** Docs or spec.
- **Location:** `crates/ciproxy-api/src/access/adapters.rs:49`.
- **Background:** An owner can approve a requested install days later.
  Meanwhile, the person who asked can lose the Admin role, or the workspace
  can move to Free, which allows fewer organizations.
- **Problem:** The install then cannot join its workspace, and no rule says
  what happens instead.
- **Impact if no action:** The App stays installed on an organization that
  no workspace serves. Its jobs stay queued.
- **Root question:** Where does an approved install go when its workspace
  cannot take it?
- **Options:**
  - **A.** Create a new workspace for the organization, with the person
    who asked as its first Admin. Effort: Small.
  - **B.** Remove the App, and show the reason at the next sign-in.
    Effort: Medium.
  - **C.** Keep the install waiting until an Admin accepts it. Effort: Large.
  - **D.** Link the organization to its workspace, but keep it inactive until
    the workspace can take it. The user proposed this option. Effort: Medium.
- **Left out:** Joining the old workspace anyway; repeated errors alone.
- **Wider fix:** None.
- **Recommendation:** A. It serves the organization without the machines or
  the keys of another workspace.
- **Action:** ask the user, because no requirement covers this outcome.

### 84. Generation does not check the API contract's structure

**Answer, 6 November 2026:** B.

- **Severity:** Low. **Effort:** Small.
- **Source:** Review, Milestone 14, pull request #30, its item 8.
- **Category:** Process.
- **Location:** `ts/api/generate.ts:28`.
- **Background:** The `api-types` gate phase compares the regenerated files
  with the saved files. Nothing checks the structure of the contract.
- **Problem:** The review found two invalid contract shapes that the drift
  check did not catch.
- **Impact if no action:** Another invalid shape can pass the gate.
- **Root question:** Should the gate also reject an invalid contract
  structure?
- **Options:**
  - **A.** Keep only the drift check and the code review. Effort: Small.
  - **B.** Add Redocly's minimal rules and path-parameter checks to
    generation, so that the gate rejects an invalid structure. Effort: Small.
  - **C.** Write down a manual structural check before each publication.
    Effort: Small.
- **Left out:** A custom schema checker; a separate CI phase.
- **Wider fix:** B.
- **Recommendation:** B. It catches defects that a byte comparison cannot.
- **Action:** ask the user, because B adds a new gate rejection.

### 85. A late first handoff can choose the cache destinations

**Answer, 6 November 2026:** F, and E as a follow-up item.

- **Severity:** High. **Effort:** Medium.
- **Source:** Review, Milestone 13, the check of #31 finding 1.
- **Category:** Security.
- **Location:** `crates/ciproxy-agent/src/assignment_state.rs:73` and
  `image/runner-patch/CiproxyAssignment.cs:45`, on the #31 branch.
- **Background:** The runner hook sends the handoff with the job's cache URLs
  and rights to the agent, and waits at most 30 seconds. Job code has root
  in its VM. The fix of finding 1 seals the first handoff against replays.
- **Problem:** If the agent is away for those 30 seconds, job code runs
  first and can send the first handoff itself, with no seal to check.
- **Impact if no action:** A job can make the host reach private addresses,
  such as the cloud metadata service, and claim cache rights it lacks.
- **Root question:** How does the agent trust a first handoff that can
  arrive after job code starts?
- **Options:**
  - **A.** Public destinations only, with DNS checks. Effort: Medium.
  - **B.** Cut the cache service off from private networks. Effort: Large.
  - **C.** Move GitHub cache traffic into the VM. Effort: Large.
  - **D.** Fixed GitHub host names only. Effort: Small.
  - **E.** A system user for the cache service with a kernel firewall output
    rule. Effort: Medium.
  - **F.** Accept a first handoff only over the connection that configured
    the runner before job code ran. After a reconnect before the first
    handoff, the job uses GitHub's own cache. Effort: Medium.
- **Wider fix:** E or A for any job-supplied URL. F closes this path.
- **Recommendation:** F.
- **Action:** ask the user, because each option adds an acceptance rule or a
  network rule.

### 86. The first handoff needs a host decision first

**Answer, 6 November 2026:** B.

- **Severity:** High. **Effort:** Medium.
- **Source:** Implementation, Milestone 13, for item 85 = F.
- **Category:** Security.
- **Location:** `image/runner-patch/CiproxyAssignment.cs:45` and
  `crates/ciproxy-agent/src/cache_authority.rs:44`, on the #31 branch.
- **Background:** The runner hook goes on without the machine cache after
  30 seconds, or at once after a socket error. Job code has root in its VM
  and can then send a first handoff on the same connection.
- **Problem:** A connection flag proves where a handoff came from, not that
  it came before job code, so F cannot tell a real late handoff from a fake.
- **Impact if no action:** A job could still choose the cache destinations
  and claim cache rights.
- **Root question:** Which cost is acceptable to prove that the first
  handoff came before job code?
- **Options:**
  - **A.** Only GitHub's cache and the VM's own Docker disk for new jobs.
    Effort: Small.
  - **B.** The hook waits for the host decision before the steps start, with
    no fallback and no guessed time limit. Effort: Medium.
  - **C.** Restrict root inside the VM, with a new image and a live proof.
    Effort: Large.
- **Wider fix:** B.
- **Recommendation:** B.
- **Action:** ask the user, because B removes an automatic fallback, A
  removes features, and C changes root inside the VM.

### 89. Idle Boat sandboxes keep running after an App removal

**Answer, 7 November 2026:** A, with B as a follow-up item to review later.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Review, Milestone 15, pull request #35, its finding 2.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-control-plane/src/boat_authority.rs:108`.
- **Background:** ciproxy finds idle Boat sandboxes through GitHub. An App
  removal ends that access, and the Boat guest reports only its runner exit.
- **Problem:** After an App removal, idle Boat sandboxes keep running.
- **Impact if no action:** Each idle sandbox bills until its lifetime ends:
  six hours, or two hours on a trial.
- **Root question:** How does ciproxy stop idle Boat cost after an App
  removal, without stopping running jobs?
- **Options:**
  - **A.** Accept it, and state the lifetime exception. Effort: Small.
  - **B.** Guest retirement through Boat's trusted controller. Effort: Large.
  - **C.** A confirmed handover receipt. Effort: Large.
  - **D.** A runner patch that stops taking jobs. Effort: Large.
- **Recommendation:** A for the MVP. Milestone 15 recommended B.
- **Action:** ask the user, because A changes a protocol promise.

### 90. The preview has no automatic control-plane build

**Answer, 7 November 2026:** G. The user ruled out A and B: "it defo
should not be manual".

- **Severity:** Low. **Effort:** Medium.
- **Source:** Implementation, the runtime of the main preview.
- **Category:** Process.
- **Location:** `.github/workflows/deploy-main-preview.yml`,
  `.github/workflows/deploy-production.yml`, `deploy/scripts/deploy.py`,
  and the `CONTROL_PLANE_IMAGE` row of `deploy/setup.md`.
- **Background:** The deploy workflow does not build the control-plane
  image. It deploys the image that the GitHub variable
  `CONTROL_PLANE_IMAGE` names, and the orchestrator builds and sets it by
  hand.
- **Problem:** No automatic step builds the image of a merged commit.
- **Impact if no action:** The preview runs older backend code than the
  primary branch until someone builds by hand.
- **Root question:** How does the preview get the control plane of each
  merged commit with no manual step?
- **Options:**
  - **A.** Keep the manual path through the user's Mac. Effort: Small.
  - **B.** Keep the manual path through the sandbox GitHub login.
    Effort: Small.
  - **C.** The deploy workflow sends a missing build to Cloud Build and
    deploys its digest. Effort: Medium.
  - **D.** A separate GitHub workflow builds and pushes on each push, and
    the deploy starts when it ends. Effort: Medium.
  - **E.** A Cloud Build trigger in GCP builds on each push, and the deploy
    waits for the image of its commit. Effort: Medium.
  - **F.** An image updater in the cluster deploys each new image.
    Effort: Large.
  - **G.** As Mokly does: a build job with no cloud credentials, a publish
    job that pushes by digest, and deploys of that digest, with a reuse of
    the image of unchanged sources. Effort: Medium.
- **Left out:** Google Cloud Deploy, because it replaces the GitHub
  promotion for the same result. A moving tag, because it loses the fixed
  digest.
- **Wider fix:** G.
- **Recommendation:** G. It is the pattern of Mokly, and its build step
  holds no cloud credentials.
- **Action:** ask the user, because G gives a CI job the right to push
  images.

### 91. The workspace picker lacks the plan and organization count

**Answer, 8 November 2026:** A. Keep the shorter picker, and update the
mockup.

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Review, Milestone 19, pull request #36, round 2, its finding 2.
- **Category:** Mockup.
- **Location:** `ts/app/src/view/add-organization.ts:56`, and
  `docs/mockups/src/screens/settings/choose-workspace.tsx:51`.
- **Background:** After an install, a person with several Admin workspaces
  picks one. The approved mockup shows the tier and the organization count
  of each workspace.
- **Problem:** The built picker shows only the name and an optional upgrade
  warning. The API summary `EligibleWorkspace` has neither fact.
- **Impact if no action:** The person cannot compare the plans and the
  organization counts before the choice.
- **Root question:** What does the picker show before the person confirms?
- **Options:**
  - **A.** Keep the reduced picker, and update the mockup. Effort: Small.
  - **B.** Add the tier and the count to `EligibleWorkspace`, regenerate the
    types, and show them as the mockup does. Effort: Medium.
  - **C.** Read the plan and the organization summaries in separate
    requests, with their own loading and error states. Effort: Medium.
  - **D.** Add both facts to the workspace summaries of the session.
    Effort: Medium.
  - **E.** Share one workspace summary and one formatter between the
    picker and the workspace switch of item 92. Effort: Medium.
- **Left out:** Deriving them from `requires_upgrade`. It gives neither
  fact. Stored counts, because they can become stale.
- **Wider fix:** E, if item 92 also shows the facts.
- **Related:** Item 92 has the same gap in the workspace switch. E serves
  both. B does not wait for item 92.
- **Recommendation:** B. One generated contract supplies the approved facts.
- **Action:** ask the user, because the user decides which side of a mockup
  and product mismatch is right.

### 92. The workspace switch shows roles, not plan facts

**Answer, 8 November 2026:** A. Keep the role, and update the mockup.

- **Severity:** Low. **Effort:** Medium.
- **Source:** Review, Milestone 19, pull request #36, the check of round 2,
  finding 2.
- **Category:** Mockup.
- **Location:** `ts/app/src/view/shell.ts:96`, and
  `ts/app/src/components/shell/menus.tsx:47`.
- **Background:** The workspace switch is the dashboard menu that opens
  another workspace. Its approved mockup shows the tier and the
  organization count of each workspace. The built switch shows the role of
  the person in each workspace.
- **Problem:** The session summary supplies neither the tier nor the count.
- **Impact if no action:** The switch differs from the approved mockup.
- **Root question:** What does each row of the switch show?
- **Options:**
  - **A.** Keep the role, and update the mockup. Effort: Small.
  - **B.** Show the tier and the count for each workspace where the person
    has a role. Keep "No role yet" for the others. Effort: Medium.
  - **C.** Show the tier and the count on every row, also without a role.
    Effort: Medium.
  - **D.** Show the role, the tier, and the count together. Effort: Medium.
- **Left out:** The limit flag of the summary. It gives neither fact.
- **Wider fix:** E of item 91.
- **Related:** Item 91, the same gap in the picker. This answer does not
  block B of item 91.
- **Recommendation:** B. It follows the mockup and keeps the current text
  for a person with no role.
- **Action:** ask the user, because the user decides which side of a mockup
  and product mismatch is right.

### 93. The installer supports fewer hosts than the docs say

**Answer, 8 November 2026:** C, and D as a follow-up item:
`follow-ups/installer-more-linux-hosts.md`.

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Implementation, Milestone 24, pull request #40, its item 1.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/agent.md:74`, and
  `crates/ciproxy-agent/src/host_packages.rs:39`.
- **Background:** The protocol and the install note of the dashboard give an
  x64 Linux hardware minimum. The installer also needs systemd and cgroup
  v2. It accepts only Debian, Ubuntu, Fedora, RHEL, Rocky Linux, and
  AlmaLinux.
- **Problem:** The protocol and the dashboard mockup do not state these
  limits.
- **Impact if no action:** A customer can prepare a machine that meets the
  stated list, and then the installer refuses it.
- **Root question:** Which hosts does ciproxy promise to support?
- **Options:**
  - **A.** Keep the hardware list, and rely on the repair messages of the
    installer. Effort: Small.
  - **B.** State the current support in the docs only. Effort: Small.
  - **C.** State these limits in the protocol, the mockup, the dashboard,
    and the docs. Effort: Medium.
  - **D.** Broaden the installer, then document the wider support.
    Effort: Large.
- **Left out:** None.
- **Wider fix:** C.
- **Recommendation:** C. Customers learn the real requirements before they
  prepare a machine.
- **Action:** ask the user, because C changes an approved requirement and
  the install note.

### 94. The Boat setup cannot connect an organization key

**Answer, 8 November 2026:** B. The user requires personal and organization
accounts. After the key check, the form lists the wallets that the key can
use, personal and each organization, and preselects the likely one. Each
Boat organization has its own wallet. A is dropped, because it supports
personal accounts only.

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
- **Source:** Implementation, Milestone 24, pull request #40, its item 2.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/wire/overflow.rs:16`, and
  `crates/ciproxy-overflow/src/boat/account.rs:46`.
- **Background:** A wallet is the personal Boat account or the Boat
  organization that pays for sandboxes. ciproxy sends it with each Boat
  request. The planned Boat form asks for a key and a spend limit. The API
  also accepts a wallet, and it uses the personal wallet when none comes.
- **Problem:** A key of a Boat organization then fails with HTTP 503 and
  "Try again". A retry does not help.
- **Impact if no action:** A customer whose Boat account is an organization
  cannot connect Boat from the dashboard.
- **Root question:** Which Boat accounts can the dashboard connect?
- **Options:**
  - **A.** Support personal accounts only. State the rule, and give an
    organization key a clear error. Effort: Small.
  - **B.** Add a choice of personal or organization to the form, the
    protocol, the mockup, and the docs. Effort: Medium.
  - **C.** Use the organization of the key when the form sends no wallet,
    and keep an explicit wallet of the API. A key that can reach several
    wallets still uses the personal one. Effort: Medium.
- **Left out:** Removing the wallet of the API. It loses a working
  capability and still needs the rule and the error of A.
- **Wider fix:** None.
- **Recommendation:** C. It connects both kinds of account with no new form
  field. Pull request #40 recommended A as the smallest change.
- **Action:** ask the user, because no requirement says which Boat accounts
  ciproxy supports.

### 95. A paid change can charge another amount than its review

**Answer, 8 November 2026:** B.

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
- **Source:** Review, Milestone 18, pull request #39, round 1, its finding 1.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-billing/src/stripe/http/preview.rs:64`, and
  `crates/ciproxy-billing/src/stripe/forms.rs:40`.
- **Background:** Proration is Stripe's charge for the rest of the current
  billing period after a plan change. A move between paid plans first
  shows a review with Stripe's preview of that charge, computed at a fixed
  proration time. The payment then builds a separate Stripe request.
- **Problem:** The payment request does not send the proration time of the
  review, so Stripe computes the charge at the time of payment. Stripe
  documents that the same time must be sent to match the preview.
- **Impact if no action:** The charge can differ from the reviewed amount.
  Across a renewal, the difference can be large.
- **Root question:** How does a paid change keep the charge that the
  customer reviewed?
- **Options:**
  - **A.** Return a signed quote with the review, and send it back with the
    payment. The server checks its signature and age, and gives its
    proration time to Stripe. Effort: Medium.
  - **B.** Store each quote on the server. The payment names its quote, and
    each retry keeps the same proration time. The server refuses an unknown
    or outdated quote. Effort: Medium.
  - **C.** Preview again at payment. When the amount changed, ask the
    customer to confirm again. Effort: Medium.
- **Left out:** Changing nothing, because it fails the review requirement.
  A proration time from the browser without a check, because a customer
  could choose a cheaper time.
- **Wider fix:** B.
- **Recommendation:** B. A stored quote best keeps the payment and its
  retries the same as the review.
- **Action:** ask the user, because each option adds a new refusal or a new
  confirmation for an outdated quote.

### 96. The docs navigation grids differ from the mockups

**Answer, 8 November 2026:** A.

- **Severity:** Low. **Effort:** Small.
- **Source:** Review, Milestone 24, pull request #40, round 1, its finding 2.
- **Category:** Mockup.
- **Location:** `ts/site/src/content/docs/docs/index.mdx:38`, and
  `ts/site/src/content/docs/docs/quickstart.mdx:93`.
- **Background:** The docs mockups show the navigation cards in two columns
  at 720 and 1440 pixels, and in one column at 390 pixels.
- **Problem:** At 720 pixels, Overview and Quickstart stack their cards in
  one column. Overview also stretches its Next link across the article,
  where the mockup puts it in the right half.
- **Impact if no action:** The pages stay usable, but they differ from the
  mockups, and the tablet pages get longer.
- **Root question:** How does the docs navigation use the available width?
- **Options:**
  - **A.** Set the shared grid styles to match the mockups. Effort: Small.
  - **B.** Keep the current layout, and update the mockups. Effort: Small.
  - **C.** Choose the columns from the article width, and revise the
    mockups around that rule. Effort: Medium.
- **Left out:** A change on each page, because it repeats the layout rules.
- **Wider fix:** A.
- **Recommendation:** A. Shared styles keep the approved mockups.
- **Action:** ask the user, because the user decides which side of a mockup
  and product mismatch is right.

### 97. The gate's Markdown link check differs from GitHub

**Answer, 7 November 2026:** I. The user wrote "we should not be blocking
for this - that is totally silly", and then chose I. A link to a missing
file still fails the gate. A fragment that matches no heading only warns.
My call, 7 November 2026: markdownlint's MD051 rule, which checks fragments
within one file, also only warns. Reason: answer I covers each anchor check
of the gate. This replaces the choice of 4 November 2026 to keep MD051
unchanged.

**Answer, 8 November 2026:** G, as a follow-up item that blocks nothing:
`follow-ups/markdown-heading-ids-comrak.md`. Until then, fix a link test
that breaks by hand. So I applies now, through pull request #49, and G
stays the follow-up item.

- **Severity:** Low. **Effort:** Small.
- **Source:** Implementation, pull request #45, the follow-up item
  `follow-ups/markdown-heading-ids-comrak.md`.
- **Category:** Repository rule.
- **Location:** `xtask/src/markdown/parser.rs`.
- **Background:** The Markdown phase of the gate finds broken links in the
  docs and plans. Its own parser computes the link target of each heading.
  The follow-up item chose comrak, a Rust parser of GitHub's Markdown. The
  orchestrator told the branch to add no new refusal without the user. All
  1,106 links of 113 documents got the same result from both parsers.
- **Problem:** An exact match with GitHub refuses some links that pass
  today. A direct switch to comrak also refuses valid `#softbreak` and
  `#hardbreak` links.
- **Impact if no action:** Valid links can fail the gate, and some broken
  links can pass it.
- **Root question:** How does the gate find the same link targets as
  GitHub?
- **Options:**
  - **A.** Repair the current parser by hand. Effort: Medium.
  - **B.** Switch directly to comrak, which fails valid-link tests.
    Effort: Medium.
  - **C.** Use a Node link checker, which also refuses missing images.
    Effort: Medium.
  - **D.** Fix comrak upstream first, then switch. Effort: Large.
  - **E.** Fix the known gaps of the current parser. Some gaps stay.
    Effort: Small.
  - **F.** Keep both parsers, and accept a link that either accepts.
    Effort: Medium.
  - **G.** Render each page with comrak, and take the heading targets from
    the HTML. It changes `Cargo.lock`, so it waits for pull request #39.
    Effort: Large.
  - **H.** Change nothing, and delete the follow-up item. Effort: Small.
  - **I.** Keep the check for missing files, but only warn when a fragment
    matches no heading. Effort: Small.
- **Left out:** A live GitHub call, because the gate then needs the
  network. A check of today's documents only, because known defects stay.
- **Wider fix:** G.
- **Recommendation:** G. One rendering model gives GitHub's targets for
  every kind of heading.
- **Action:** ask the user, because G refuses some links that pass today,
  and a new refusal in the gate needs the user.

### 98. Sign-in removes the Admin role of a new install

**Answer, 8 November 2026:** A. The App gets the organization permission
"Members: read", and the protocol and the docs list four permissions. This
replaces the three permissions of item 54 = E and the security rule that
the owner check uses no App permission. An added option D, roles without
GitHub membership, is rejected.

- **Severity:** High. **Likelihood:** Common. **Effort:** Small.
- **Source:** Implementation, the first live install on the main preview,
  8 November 2026.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/identity/http.rs:208`, and
  `crates/ciproxy-store-pg/src/dashboard/identity.rs:92`.
- **Background:** A direct install creates a workspace and makes its
  requester the first Admin. Each sign-in reads the person's organization
  memberships with `GET /user/memberships/orgs`. It deletes each role in a
  workspace whose organizations are not in that list. The App has three
  permissions: Actions read, Metadata read, and organization self-hosted
  runners write. GitHub lists the organization membership endpoints under
  the "Members" permission for GitHub App user tokens.
- **Problem:** On the preview, the install of Firna-AI created the workspace
  and the Admin role. Each later sign-in removed the role, so the user sees
  no workspace. The membership list returned no Firna-AI.
- **Impact if no action:** No customer can use the dashboard after an
  install.
- **Root question:** How does sign-in learn which organizations a person
  owns or belongs to?
- **Options:**
  - **A.** Add the organization permission "Members: read" to the App, and
    list four permissions in the protocol and the docs. Each install then
    asks for it. Effort: Small.
  - **B.** Keep three permissions. Base roles on the installations that
    the person's token can see, and drop the owner checks of items 82 and
    83. Effort: Large.
  - **C.** Keep three permissions. Read the memberships through a second,
    OAuth sign-in with the `read:org` scope. Effort: Medium.
- **Left out:** Keeping a role whose organization the membership list
  omits, because a person who left the organization would keep access.
- **Wider fix:** A.
- **Recommendation:** A. It is a common, read-only permission, and the role
  model stays as the user approved it.
- **Action:** ask the user, because A changes the App's three permissions,
  which item 54 = E keeps.

### 99. The live machine check needs a public agent release

**Answer, 8 November 2026:** A.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, the Milestone 13 live check, 8 November 2026.
- **Category:** Process.
- **Location:** `deploy/helm/release-values.yaml:4`, and
  `crates/ciproxy-agent/smoke-checks.md`.
- **Background:** The live check installs the agent on the test host the
  way a customer does, from the preview's release server. That server
  serves releases of the public repository `Firna-AI/ciproxy`, which has no
  release yet. The check also needs a second allowed private repository and
  a private repository that the runner group excludes.
- **Problem:** The check cannot run without an agent release and the test
  repositories.
- **Impact if no action:** The own-machine path stays unproven on real
  hardware before launch.
- **Root question:** Where does the live check get its agent release and its
  test repositories?
- **Options:**
  - **A.** Publish a pre-release of the agent on `Firna-AI/ciproxy`, create
    two private test repositories in Firna-AI, and limit the workspace to
    the test repositories. Effort: Small.
  - **B.** Point the preview's release server at a private repository with
    the release, and create the same test repositories. Effort: Medium.
  - **C.** Copy the agent to the test host by hand, and skip the install
    path of the check. Effort: Small.
- **Left out:** Using the real private repositories of Firna-AI, because a
  test must not run jobs in them.
- **Wider fix:** None.
- **Recommendation:** A. The agent is open source, so a public pre-release
  is safe, and the check then proves the customer's install path.
- **Action:** ask the user, because A publishes a release and creates
  repositories in the user's organization.

### 100. The protected image workflow needs a home before launch

**Answer, 8 November 2026:** B. Move the binding to `main` at launch.

- **Severity:** High. **Effort:** Small.
- **Source:** Review, pull request #47, round 1, finding 1, and its fix.
- **Category:** Security.
- **Location:** `deploy/image-workflow-bootstrap.md` and
  `deploy/terraform/bootstrap/main.tf` on the branch of pull request #47.
- **Background:** The fix of finding 1 lets only one reusable workflow get
  the image-publishing credentials. The npm and Cargo scripts of other
  deploy jobs cannot. The fix pins that workflow to the protected `main`
  branch, which today holds only `README.md`.
- **Problem:** Until `main` holds the whole repository, every preview
  deploy fails to start.
- **Impact if no action:** #47 cannot merge, and the preview keeps its
  manual image step.
- **Root question:** Where does the protected image workflow live until
  launch?
- **Options:**
  - **A.** Land the whole repository on `main` now, through a reviewed pull
    request that passes the gate, and pin the workflow there.
    Effort: Medium.
  - **B.** Pin the workflow to the primary branch until launch. Other jobs
    still cannot get the credentials. Only a change of the workflow file on
    the primary branch can. Move the pin to `main` at launch.
    Effort: Small.
  - **C.** Protect the primary branch with a ruleset that requires pull
    requests and the gate, and pin the workflow to it. Each merge then
    goes through a pull request. Effort: Medium.
- **Left out:** No pin, because the npm scripts of a deploy job could then
  publish images.
- **Wider fix:** A.
- **Recommendation:** B. Previews stay automatic, the npm path stays
  closed, and the code reaches `main` with the launch anyway.
- **Action:** ask the user, because A changes `main`, and B and C change
  the trust of the deploy.

### 101. The Machines and Settings mockups need facts the API lacks

**Answer, 8 November 2026:** B.

- **Severity:** Medium. **Effort:** Large.
- **Source:** Implementation, Milestone 20, pull request #50, its item 1.
- **Category:** Mockup.
- **Location:** `ts/api/generated/api.d.ts`, and the Milestone 20 mockups.
- **Background:** The approved Machines and Settings mockups show agent
  update status, Docker disk counts, disk limits, job links, label
  placement, and organization and member details.
- **Problem:** The page responses of the API do not give these facts.
- **Impact if no action:** The screens cannot show every approved state.
- **Root question:** How do the screens get these facts?
- **Options:**
  - **A.** Reduce the screens, and update the mockups and the affected
    requirements. Effort: Medium.
  - **B.** Extend the existing page responses, and regenerate the types.
    Effort: Large.
  - **C.** Add separate metadata endpoints, with their own requests and
    failure states. Effort: Large.
  - **D.** Use existing routes where possible, and add the rest. This adds
    job polling and GitHub reads to the browser. Effort: Large.
  - **E.** Keep the facts that the protocol requires, and remove the other
    mockup details. Effort: Large.
- **Left out:** Guesses in the browser, because they cannot establish the
  facts.
- **Wider fix:** None.
- **Recommendation:** B. One contract supplies the approved facts, with no
  extra GitHub reads in the browser.
- **Action:** ask the user, because the choice is API work or a change of
  the approved design.

### 102. The warm runner control has two meanings

**Answer, 8 November 2026:** A.

- **Severity:** Low. **Effort:** Small.
- **Source:** Implementation, Milestone 20, pull request #50, its item 2.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/screens/machines/machine-detail.tsx:134`.
- **Background:** A warm runner waits on a machine for a job. The machine
  keeps a saved number of them. Draining keeps that saved number.
- **Problem:** The mockup shows zero in the control of a draining or
  offline machine. The API keeps the saved number, such as two.
- **Impact if no action:** The control differs from the mockup, and an edit
  from a shown zero could overwrite the saved number.
- **Root question:** What does the warm runner control show?
- **Options:**
  - **A.** Show the saved number in every state, and update the mockup.
    Effort: Small.
  - **B.** Hide the control in those states, and show the ready count when
    known. Effort: Small.
  - **C.** Show the saved number and a separate ready count. Effort: Small.
- **Left out:** None.
- **Wider fix:** None.
- **Recommendation:** A. The control then edits the real saved number.
- **Action:** ask the user, because the approved mockup and the product
  differ.

### 103. The preview release server ignores the agent pre-release

**Answer, 8 November 2026:** A now, and B as a follow-up item in parallel:
`follow-ups/release-server-prerelease-channel.md`. `ciproxy-agent-v0.1.0`
is now a full release, and the preview release server serves it.

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation, the Milestone 13 live check, 8 November 2026.
- **Category:** Product bug.
- **Location:** the pinned `cli-release-server`, `github/releases.rs:21`.
- **Background:** Item 99 = A published the agent as the GitHub pre-release
  `ciproxy-agent-v0.1.0`. The release server asks GitHub for the latest
  release, and GitHub leaves pre-releases out of that answer.
- **Problem:** The release server finds no release, so the install fails.
- **Impact if no action:** The live check cannot install the agent.
- **Root question:** How does the preview install the agent release?
- **Options:**
  - **A.** Mark `ciproxy-agent-v0.1.0` as a normal release. It then shows as
    the latest release of the public repository. Effort: Small.
  - **B.** Add a pre-release channel or a pinned tag to the release server of
    `futex-ai/cli-release`, and a new pinned server image. Effort: Medium.
- **Left out:** A copy by hand, because it skips the customer install path.
- **Wider fix:** B.
- **Recommendation:** A. It is one change on GitHub, and no customer uses the
  release server before launch.
- **Action:** ask the user, because A changes the pre-release choice of item
  99.

### 104. The test host disk is below the install minimum

**Answer, 8 November 2026:** A.

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation, the Milestone 13 live check, 8 November 2026.
- **Category:** Process.
- **Location:** the host `ciproxy-spike-2` in the GCP project
  `internal-498318`.
- **Background:** The install needs at least 250 GB on the agent's file
  system. Each default job VM takes about 81 GB. The host has a 200 GiB
  boot disk, which also holds the image build outputs.
- **Problem:** The installer refuses the host.
- **Impact if no action:** No live machine check can run.
- **Root question:** Where does the test host get its job disk?
- **Options:**
  - **A.** Grow the boot disk to 500 GiB, and delete build outputs that can
    be rebuilt. A disk cannot shrink again. Effort: Small.
  - **B.** Attach a separate 500 GiB data disk at `/var/lib/ciproxy`.
    Effort: Medium.
- **Left out:** A smaller image reserve, because it changes an approved
  default.
- **Wider fix:** None.
- **Recommendation:** A. The host is deleted after the tests, so the
  growth costs about USD 30 a month only while the tests run.
- **Action:** ask the user, because the host is in another project, and
  both options add storage cost.

### 105. The sandbox GitHub login cannot write workflow files

**Answer, 8 November 2026:** A.

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation, the Milestone 13 live check, 8 November 2026.
- **Category:** Process.
- **Location:** `~/.local/bin/gh-user` of the orchestrator sandbox.
- **Background:** The live check needs its workflow file on each test
  branch. GitHub needs the `workflow` scope to write a workflow file.
- **Problem:** The sandbox login has `repo` and `read:org` only, so GitHub
  refuses the writes.
- **Impact if no action:** The check cannot start its jobs.
- **Root question:** Which credential writes the workflow files?
- **Options:**
  - **A.** Sign the sandbox in again with the `workflow` scope. Effort:
    Small.
  - **B.** Create a fine-grained token for the test repositories only, with
    Contents and Workflows write. Effort: Small.
  - **C.** The user pushes the prepared files. Effort: Small.
- **Left out:** None.
- **Wider fix:** None.
- **Recommendation:** A. One more GitHub approval, and the sandbox does the
  rest.
- **Action:** ask the user, because only the user can grant the scope.

### 106. The Docker disk checks need a paid workspace

**Answer, 8 November 2026:** A, after pull request #39 serves Checkout.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, the Milestone 13 live check, 8 November 2026.
- **Category:** Process.
- **Location:** the billing tier of the test workspace "Firna-AI".
- **Background:** Docker disks are a Team and Business feature. The test
  workspace is on Free.
- **Problem:** The Docker disk check and both Docker recovery checks of
  Milestone 25 cannot run.
- **Impact if no action:** These cases stay unproven before launch.
- **Root question:** How does the live check prove the paid features?
- **Options:**
  - **A.** Upgrade the test workspace to Team with a Stripe test-mode
    checkout, after pull request #39 serves Checkout. Effort: Small.
  - **B.** Keep Free, and leave these cases unproven. Effort: Small.
- **Left out:** A tier set by hand in the database, because it skips the
  billing path.
- **Wider fix:** None.
- **Recommendation:** A. The test-mode checkout is also a Milestone 21
  check.
- **Action:** ask the user, because A changes the plan of a workspace.

### 107. The cache firewall proof takes over the metadata route

**Answer, 8 November 2026:** A.

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Implementation, the Milestone 13 live check, 8 November 2026.
- **Category:** Test.
- **Location:** `crates/ciproxy-agent/scripts/cache_egress_network.py:79`.
- **Background:** The proof adds its own route to the cloud metadata
  address. Its guide says that the proof fails when the host already has
  that route. A GCP host has it, with a lower priority than the proof's
  route.
- **Problem:** On a GCP host, the proof can divert the host's real metadata
  and DNS traffic.
- **Impact if no action:** The proof cannot run safely on the GCP test
  host.
- **Root question:** How does the proof check the metadata block on a cloud
  host?
- **Options:**
  - **A.** Change the proof so that it keeps the provider's route and DNS,
    and still checks the firewall. Effort: Medium.
  - **B.** Run the proof on another host without that route. Effort: Medium.
- **Left out:** Skipping the metadata check, because it weakens the proof.
- **Wider fix:** A.
- **Recommendation:** A. It works on the hosts that customers use.
- **Action:** ask the user, because A changes a proof fixture of the
  security fix, and B needs another machine.

### 108. Pending credits can change the quoted charge

**Answer, 8 November 2026:** C with D, as a third fix round of the pull
request #39.

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
- **Source:** Review, pull request #39, after its second fix round.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-billing/src/stripe/http/preview.rs:77`.
- **Background:** An add-on change, such as a machine removal, can leave a
  pending credit in Stripe. Stripe's preview counts it, but the payment of
  an upgrade does not.
- **Problem:** A review can show USD 115, and the payment can charge
  USD 125. Two reviewers confirmed this from Stripe's documentation. No
  live payment reproduced it.
- **Impact if no action:** A charge can exceed the reviewed amount.
- **Root question:** How does the review match Stripe's upgrade charge?
- **Options:**
  - **A.** Take the pending items out of the preview. Effort: Medium.
  - **B.** Compute today's charge from Stripe's lines and the customer
    balance. Effort: Medium.
  - **C.** Send the same offset in the paid change, and let Stripe compute
    the amount. Effort: Medium.
  - **D.** Compare the invoice with the quote after payment, and alert on a
    difference. It only adds to a fix. Effort: Small.
  - **E.** Invoice pending items before the upgrade. Effort: Medium.
  - **F.** Invoice add-on changes at once. Effort: Large.
- **Left out:** Payment modes that apply the plan before payment, because
  they break the bank confirmation flow.
- **Wider fix:** D.
- **Recommendation:** C with D. Stripe keeps computing the amounts, and an
  alert catches a later mismatch.
- **Action:** ask the user, because the two fix rounds of #39 are used, and
  the finding is critical.

### 109. The spend limit notice lacks the day it was reached

**Answer, 8 November 2026:** C.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Implementation, Milestone 22, pull request #51.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/screens/settings/overflow.tsx:95`.
- **Background:** The approved mockup of the spend limit notice gives the
  day the limit was reached and the day Boat resumes.
- **Problem:** The API gives no day when the limit was reached.
- **Impact if no action:** The product and the mockup differ.
- **Root question:** Which days does the notice show?
- **Options:**
  - **A.** Only the resume day, and update the mockup. Effort: Small.
  - **B.** Store the day the limit was reached, and show it. Effort: Large.
  - **C.** The budget month and the resume day, as Jobs shows them, and
    update the mockup. Effort: Small.
- **Left out:** The browser's clock, because it names the visit, not the
  event.
- **Wider fix:** None.
- **Recommendation:** C. It matches Jobs with data that the API has.
- **Action:** ask the user, because the approved mockup changes.

### 110. History returns every matching job in one response

**Answer, 8 November 2026:** B.

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
- **Source:** Implementation, Milestone 21, pull request #53, its item 16.
- **Category:** Performance.
- **Location:** `crates/ciproxy-api/src/handlers/jobs.rs:32`, and
  `crates/ciproxy-store-pg/src/job_views.rs:35`.
- **Background:** Each read of History, of the live queue, and of a job page
  builds every retained job of the workspace before any filter. A job is
  retained for 90 days. The live queue reads again every 5 seconds. The
  approved mockup shows pages and a total.
- **Problem:** A large workspace can make these reads slow.
- **Impact if no action:** Slow Jobs pages on large workspaces.
- **Root question:** How does History deliver the jobs of a large plan
  window?
- **Options:**
  - **A.** Measure first at Business volume, and change only when slow.
    Effort: Medium.
  - **B.** Pages without a total: the store applies the filters, the order,
    and a limit, and "Show more" loads the next page. Effort: Medium.
  - **C.** Pages with a total, as the mockup shows. Effort: Medium.
  - **D.** One full list, with the filters in the store. Effort: Medium.
  - **E.** A capped list, with a hint to narrow the filter. Effort: Medium.
- **Left out:** None.
- **Wider fix:** B.
- **Recommendation:** B. Each read stays bounded, also the live queue.
- **Action:** ask the user, because the approved mockup's total changes.

### 111. The banners lack the plan and the passed limit

**Answer, 8 November 2026:** A.

- **Severity:** Low. **Effort:** Medium.
- **Source:** Implementation, Milestone 21, pull request #53, its item 17.
- **Category:** Mockup.
- **Location:** `crates/ciproxy-api/src/wire/pages.rs:36`.
- **Background:** The approved banners name the plan, the passed limit, and
  the day a plan ended. The banner response gives fewer facts.
- **Problem:** The product banners are true, but less precise.
- **Impact if no action:** The product and the mockups differ.
- **Root question:** What does each banner name about its cause?
- **Options:**
  - **A.** Use today's wording, and update the mockups. Effort: Small.
  - **B.** Add typed facts to the banner response, and update the
    protocol. Effort: Medium.
  - **C.** Read Plan and usage beside the banner. Effort: Medium.
- **Left out:** A day computed from the banner's end time, because a month
  end can give a wrong day.
- **Wider fix:** None.
- **Recommendation:** B. One response keeps every page consistent.
- **Action:** ask the user, because the approved mockups change.

### 112. Jobs notices need facts that the API lacks

**Answer, 8 November 2026:** D.

- **Severity:** Low. **Effort:** Medium.
- **Source:** Implementation, Milestone 21, pull request #53, its item 18.
- **Category:** Mockup.
- **Location:** `plans/follow-ups/jobs-notice-facts.md` on the #53 branch.
- **Background:** The approved Jobs examples name suspension details,
  refusal reasons, the actual Boat size, and resource shortages.
- **Problem:** The API gives fewer facts.
- **Impact if no action:** The product and the examples differ.
- **Root question:** What must Jobs explain about why a job waits or where
  it ran?
- **Options:**
  - **A.** Use today's wording, and update the examples. Effort: Small.
  - **B.** Add suspension, refusal, and run-size facts. Effort: Large.
  - **C.** Save the scaler's first reason. Effort: Medium.
  - **D.** Save the actual size choice when a job starts. Effort: Medium.
  - **E.** D, and typed reasons for temporary refusals. Effort: Large.
  - **F.** Add shortage facts for each kind of place. Effort: Medium.
  - **G.** Show the largest job that each kind of place can run.
    Effort: Medium.
- **Left out:** Fit computed in the browser, because it can differ from the
  backend rules.
- **Wider fix:** None.
- **Recommendation:** D. It records what really ran.
- **Action:** ask the user, because the approved examples change.

### 113. The label dialog lacks the largest-job figures

**Answer, 8 November 2026:** A.

- **Severity:** Low. **Effort:** Small.
- **Source:** Review, Milestone 21, pull request #53, its item 19.
- **Category:** Mockup.
- **Location:** `plans/follow-ups/label-dialog-fit-offers.md` on the #53
  branch.
- **Background:** The approved label dialog shows the largest job that own
  machines and Boat can run. The API does not give those figures.
- **Problem:** The product cannot show them.
- **Impact if no action:** Only that extra text differs.
- **Root question:** What does the dialog say about the capacity for a
  label?
- **Options:**
  - **A.** Keep the fit counts and the placement, and remove the extra
    text from the mockup. Effort: Small.
  - **B.** Add the figures to the API, shared with option G of item 112.
    Effort: Medium.
- **Left out:** Figures computed in the browser, for the reason of item 112.
- **Wider fix:** None.
- **Recommendation:** A. The protocol requires only the counts and the
  placement.
- **Action:** ask the user, because the approved mockup changes.

### 114. The release channel change waits for a merge

- **Severity:** High. **Effort:** Small.
- **Source:** Implementation, plan item 103, option B, 8 November 2026.
- **Category:** Process.
- **Location:** `futex-ai/cli-release` pull requests #6 and #7, and
  `deploy/scripts/mirror-release-image.sh`.
- **Terms:**
  - **Release channel:** the new `CLI_RELEASE_CHANNEL` setting of the
    release server. `stable` serves full releases only, and `prerelease`
    also serves pre-releases.
  - **Pinned revision:** the `cli-release` commit that
    `mirror-release-image.sh` builds into the release server image.
- **Background:** Pull request #7 adds the release channel. Its review
  found no problem, and every check passed except one Clippy error. That
  error also fails on `main`, and the user's pull request #6 fixes it.
  ciproxy builds the server image from the pinned revision, so it needs no
  published `cli-release` release.
- **Problem:** The change is not on `main`, so ciproxy has no merged
  revision to pin.
- **Impact if no action:** After launch, a test release of the agent
  reaches every customer machine, because both environments follow full
  releases only.
- **Root question:** How does ciproxy get the reviewed channel change into
  its release server image?
- **Options:**
  - **A.** Merge #6, then update #7 and merge it when its checks pass.
    Publish no release. ciproxy then pins the merged commit. Effort: Small.
  - **B.** Do A, and also merge the release pull request that the release
    automation opens. This publishes the shared crates and the server
    image of `cli-release` for every product that uses it. Effort: Small.
  - **C.** Leave both pull requests open, and pin the reviewed head
    `55eae44`. The pin then names a commit outside `main`. Effort: Small.
- **Left out:** Closing #7 and marking each test release as a full release,
  because that fails the goal after launch.
- **Wider fix:** None. The channel is the wider fix of item 103.
- **Recommendation:** A. It puts the reviewed change on `main`, and ciproxy
  needs no release.
- **Action:** ask the user, because the user approves each merge and each
  release in `futex-ai/cli-release`.
