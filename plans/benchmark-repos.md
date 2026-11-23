# Benchmark repositories

Status: Active. On branch `calummoore/benchmark-repos-ciproxy-gm3g`, no pull
request yet. The pull request that completes the work deletes this plan, as
the `plans` skill says.

ciproxy has no repeatable way to run the same CI work on each runner
provider. This change sets up private benchmark repositories. They run the CI
jobs of real open-source projects, in several language stacks, on each
provider. They check that the full flow of each provider works. They also
measure the queue time, the run time, and the cost of each job. The first
round covers four legs: GitHub-hosted runners, Blacksmith, ciproxy on a GCP
own machine, and ciproxy on Boat only. The Hetzner leg follows when its
server is ready. Public claims from the results are later work.

The repositories live outside this repository. This repository is public,
and its rules keep measurements out of it. This plan is the record of the
decisions.

Contract owners: none in this repository. The README of the benchmark
source repository states the method. No protocol doc changes, because the
benchmark changes no product behaviour.

## Terms

- **Leg:** one provider in the benchmark, such as "ciproxy on GCP".
- **Sengi suite:** `sengirunners/benchmarks`, a public Apache-2.0 suite from
  the runner company Sengi. It runs the CI jobs of 12 open-source projects,
  each pinned to the commit of a release, from one workflow that takes a
  runner input.
- **Round:** one warm-up run, five measured runs, and one cold run of every
  project on one leg.
- **Main preview:** the deployed test environment of ciproxy at
  `https://br-main.preview.app.ciproxy.dev`.

## Decisions

User decisions, 23 November 2026, from the brainstorm of the same day:

- Item 1, A: copy the Sengi suite into private repositories, and set the
  label of each leg. Rejected: B, our own repositories and collector, and C,
  synthetic probes only. Reason: the suite gives 12 checked, pinned
  workloads and a published method at the lowest effort.
- Item 2, A: the user ordered a Hetzner AX42 for the Hetzner leg. Blacksmith
  gets a payment method for its free minutes. The GCP leg uses the existing
  GCP account. Rejected: B, a one-month server, and C, no Hetzner leg.
- Item 3, deferred: the user wrote "lets just do gcp for now, then we can
  add hetzner later". So the first round has one own machine, on GCP. How
  the Hetzner and GCP legs stay apart stays open until the Hetzner server is
  ready. The options were A, drain one machine for each leg, B, one
  organization and one workspace for each machine, and C, sort the jobs
  afterwards. The recommendation was B. Milestone 7 waits for it.
- Item 4, B: internal results now, public claims later. Each round runs
  all 12 projects of the suite. Public numbers name the suite and leave out
  the nested GCP leg. Rejected: A, six projects for internal checks only.
- Item 5, A: the GCP leg runs on a new nested-KVM VM, and its results carry
  the word "nested". Rejected: B, a GCP bare-metal instance, because of its
  cost.
- Item 6, C, 23 November 2026: a new GitHub organization with its own
  main-preview workspace on Team holds the benchmark repositories. Rejected:
  A, `firna-test-org` as it is, B, `Firna-AI`, D, `firna-test-org` for the
  benchmark only, E, `futex-ai` or `mokly-ai`, and F, two organizations.
  The user did not choose the wider fix G, a guide of the test
  organizations. Reason: only a new organization gives a workspace that no
  other work shares.

### 6. Which GitHub organization holds the benchmark repositories?

The options check of 23 November 2026 corrected the background and options
A to C, and added the options D to G. Its summary is in
`.context/benchmark-repos/options-check-item-6.md`.

- **Severity:** High. **Effort:** Small.
- **Source:** Design.
- **Category:** Process.
- **Location:** None.
- **Terms:**
  - **Workspace:** the ciproxy account of one customer. It holds GitHub
    organizations, own machines, labels, overflow settings, and a tier.
- **Background:**
  - ciproxy runs the jobs of private repositories only.
  - An organization belongs to one workspace at most. The labels and the
    overflow settings of a workspace serve all its organizations.
  - A Free workspace gets no Docker disk, so the benchmark needs a paid
    tier. Team is sufficient.
  - In the main preview, `Firna-AI` is on Business and holds the main test
    machine. `firna-test-org` has been on Team since 17 November 2026.
  - The open Boat checks of the MVP plan change the overflow settings of
    both workspaces. The overflow smoke deletes the sandbox of each Boat
    start of its workspace during a check.
- **Problem:** The benchmark needs a workspace whose labels, machines,
  overflow settings, and Boat sandboxes no other work changes.
- **Impact if no action:** No repository and no leg can start.
- **Root question:** Which GitHub organization gives the benchmark a
  workspace that no other work uses or changes?
- **Options:**
  - **A.** Use `firna-test-org` as it is. It exists and is on Team. The open
    Boat checks share its workspace, so a check can change the overflow
    settings or delete a benchmark sandbox. Our GitHub credential cannot
    see it today. Effort: Small.
  - **B.** Use `Firna-AI`. It is on Business, and its App access is limited
    to selected repositories. The main test machine is in its workspace, so
    a `ciproxy` job can run there. The Boat checks share it too. Effort:
    Small.
  - **C.** A new organization, such as `ciproxy-benchmarks`, with a new
    main-preview workspace from "Install on another organization", on Team
    in Stripe test mode. Nothing else shares it, and it fits option B of
    item 3. The user creates it on the web, or in the enterprise account
    through the API. It can need a payment method for the minutes of the
    GitHub-hosted leg. Effort: Small.
  - **D.** Make `firna-test-org` the benchmark's own organization. Move the
    open checks that use it elsewhere, and remove its other machines and
    settings. It needs no new install or Checkout. The moved checks need a
    new home. Effort: Medium.
  - **E.** Use `futex-ai` or `mokly-ai`, with the preview App and Blacksmith
    on the benchmark repositories only. The benchmark shares the members and
    the Actions minutes of another product. Effort: Small.
  - **F.** C for the ciproxy legs only. The GitHub-hosted and Blacksmith legs
    stay in `Firna-AI`. The scripts and credentials then have two owners,
    and the isolation is not better. Effort: Small.
  - **G.** As a wider fix, add a short guide that lists each test
    organization, its workspace, its tier, its machines, and the check that
    owns it. Add the rule that work which changes a workspace runs only in
    a workspace that it owns. Effort: Small.
- **Left out:**
  - The user's personal account, because ciproxy makes no workspace for a
    personal account.
  - A label that selects a machine or an organization, because C gives the
    same isolation with no product change.
  - A second ciproxy deployment, because C gives the same isolation with no
    new environment.
  - Production, because it is not deployed yet.
- **Wider fix:** G. It prevents the class: two kinds of work that change the
  same workspace.
- **Previous examples:** Item 60 created `Firna-AI/ciproxy-spike-refused`.
  Item 99 created two private test repositories in `Firna-AI`. Item 120
  created `firna-test-org`. The only record of the test workspaces is in
  `plans/ciproxy-mvp.md`, which goes when that plan is deleted.
- **Recommendation:** C, with G. C is the only option that no other work
  shares, and G keeps the next check from reusing the benchmark workspace.
- **Action:** ask the user, because the option needs a new organization that
  only the user can create, and G adds a new process rule.

## Design

My calls, 23 November 2026:

- **Repositories.** One private source repository, `bench-source`, holds
  the copy of the suite, the collector, and the results. Each leg gets its
  own private repository: `bench-github`, `bench-blacksmith`,
  `bench-ciproxy-gcp`, and `bench-ciproxy-boat`, and later
  `bench-ciproxy-hetzner`. A sync script pushes the same workflow to each
  leg. Reason: each repository has its own GitHub cache, so one leg cannot
  warm or evict the cache of another leg.
- **Copy of the suite.** The copy keeps the `LICENSE` and the `NOTICE` of
  the suite and names its source commit. Reason: Apache-2.0 requires them.
- **Extra project.** The suite adds the gate of this repository at a
  pinned commit, as a job with a `services: postgres` container. Reason:
  no suite project uses a service container, and ciproxy must run one.
- **Sizes.** Each leg uses 2 vCPU and 8 GB: `ubuntu-24.04`,
  `blacksmith-2vcpu-ubuntu-2404`, the ciproxy label `ciproxy` with its Boat
  switch off, and a new label `ciproxy-boat` with the same size and own
  machines off. Each job records `nproc`, the memory, and the free disk.
  Reason: Boat can start a larger sandbox than the label asks for, and the
  results must show it. Only `ciproxy` gets warm runners.
- **Method.** One round per leg: one warm-up run that is not measured, five
  measured runs, and one cold run after the GitHub cache and the machine
  cache are cleared. One leg runs at a time.
- **Metrics.** The collector reads the GitHub jobs API. Queue time is
  `started_at` minus `created_at`. Run time is `completed_at` minus
  `started_at`. It joins the job view of the dashboard for the machine, the
  Boat size, and the cache source. It drops each sample that ran in the
  wrong place and counts it. Cost is the billed minutes times the list
  price for a hosted leg. For Boat, it is the dated list price of the
  job's Boat size, times its queue time plus its run time. For an own
  machine, it is the monthly machine cost divided by use. Each price
  names its date.
  My call, 23 November 2026, after finding 5 of the first review of
  `bench-source`: Boat bills a monthly plan plus overage and puts deleted
  sandboxes in one line, so no Boat bill states the cost of a round or a
  job. This replaces "the Boat bill for Boat". A round bill stays an
  optional cross-check.
- **Environment.** The main preview runs the ciproxy legs. Production
  repeats them during its smoke test.
- **GCP machine.** An `n2-standard-16` VM with nested KVM and a 500 GB
  balanced persistent disk in `us-central1` of `internal-498318`. Reason: 16
  vCPU and 64 GB match the threads and the memory of the Hetzner AX42, and
  the MVP plan keeps test machines in that project. The VM stops between
  rounds.
- **Credentials.** No key, token, or cookie goes into a file of any
  repository or into a log. The collector reads them from the environment.

## Milestone 1: Accounts and the workspace

Depends on: none

The user creates the accounts. The agent checks each step.

- [ ] User: create the organization of item 6.
- [ ] User: install the main-preview App on it with "Install on another
      organization", so that it gets a new workspace. "Add an organization"
      would add it to an existing workspace.
- [ ] User: move the workspace to Team with a Stripe test-mode Checkout.
- [ ] User: give the GitHub credential of the agent access to the
      organization. Add a payment method for Actions minutes if the
      GitHub-hosted leg needs more than the included minutes.
- [ ] User: install the Blacksmith App on the organization, and add a
      payment method.
- [ ] User: connect a Boat account to the workspace, with a spend limit.
- [ ] Check each step through the API of the main preview and of GitHub.

## Milestone 2: The benchmark source repository

Depends on: 1

- [x] Copy the suite at a pinned commit, with its `LICENSE` and `NOTICE`,
      into a local repository `bench-source`.
- [x] Add the gate of this repository as the extra project.
- [x] Add the step that records `nproc`, the memory, and the free disk.
- [x] Write the README: the legs, the projects, the method, the metrics,
      and the commands.
- [ ] Review the local repository with the `implementation-review` skill,
      and apply the fix rule.
- [ ] Create the private repository `bench-source` in the organization of
      item 6, and push the reviewed commits.

## Milestone 3: The leg repositories and labels

Depends on: 2

- [x] Write the sync script.
- [ ] Create the four leg repositories with the sync script.
- [ ] Turn off the Boat switch of `ciproxy`. Add the label `ciproxy-boat`.
- [ ] Run one project once on each leg as a smoke check.

## Milestone 4: The GCP own machine

Depends on: 1

- [ ] Create the VM of the design, with the user's approval of the
      credentials.
- [ ] Install the agent with the install command of the dashboard. Check
      that the machine connects and has its image.

## Milestone 5: The collector

Depends on: none

- [x] Write the collector: dispatch, wait, read the jobs API and the job
      view, drop the wrong-place samples, and write the results.
- [x] Add tests for the parse and the sample rules with stand-in responses.

## Milestone 6: The first round

Depends on: 3, 4, 5

- [ ] Run one round on each of the four legs.
- [ ] Record each failure of a ciproxy leg as a finding, with a
      follow-up item in this repository when it is a product bug.
- [ ] Write the results table in `bench-source`. Save the run logs under
      `.context/benchmark-repos/`.

## Milestone 7: The Hetzner leg

Depends on: 6. Waits for the Hetzner server and for item 3.

- [ ] Prepare the server as `crates/ciproxy-agent/host-preparation.md`
      says, and install the agent.
- [ ] Add the leg as item 3 decides, and run one round.

## Milestone 8: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the Markdown check named in `AGENTS.md`.
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

- [ ] Public claims from the results, as item 4 says: all 12 projects, the
      name of the suite, and no nested GCP numbers.
- [ ] Repeat the ciproxy legs on production during its smoke test.
