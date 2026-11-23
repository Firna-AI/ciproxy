# GitHub fallback picker

Status: Active. On branch `calummoore/github-picker-f9sj`, no pull request
yet. The pull request that completes the work deletes this plan, as the
`plans` skill says. Decisions 2, 4, 5, 6, and 7 still wait for the user.

A job that names a ciproxy label waits in the GitHub queue while ciproxy
cannot start it. A macOS job never overflows, so it waits until a Mac is
free. Every job waits while the control plane is down. This change adds an
opt-in picker. A short first job of the workflow asks ciproxy whether the
later jobs can start soon. ciproxy answers with the ciproxy label when an
own machine or Boat can take them. Otherwise, and when ciproxy does not
answer, the picker gives a fallback label that the workflow names, such as
`macos-latest`. A workflow without the picker keeps today's behaviour.

The change does not move a job after it is queued. GitHub has no API that
changes the labels of a queued job, and it can cancel only a whole run.
The change adds no macOS overflow provider, and no new GitHub App
permission.

The pick is a forecast, not a reservation. GitHub gives an idle runner to
any waiting job of its label and organization, as
[Own machines first](../docs/protocol/job-routing.md#own-machines-first)
says. So a job from a workflow without the picker can take the place that a
pick saw.

Example workflow, after the design calls below:

```yaml
jobs:
  pick:
    runs-on: ubuntu-slim
    permissions:
      id-token: write
    outputs:
      runner: ${{ steps.pick.outputs.runner }}
    steps:
      - id: pick
        uses: Firna-AI/ciproxy/actions/pick@v1
        with:
          label: ciproxy-macos
          fallback: macos-latest
  test:
    needs: pick
    if: ${{ !cancelled() }}
    runs-on: ${{ needs.pick.outputs.runner || 'ciproxy-macos' }}
```

Contract owners:

- [Labels](../docs/protocol/labels.md).
- [Job routing](../docs/protocol/job-routing.md).
- [Security](../docs/protocol/security.md) and
  [Security data](../docs/protocol/security-data.md).
- A new protocol doc, `docs/protocol/picker.md`.
- The [architecture doc](../docs/architecture.md).
- The [repository README](../README.md), for the list of open directories.

## Decisions

### 1. How a busy job still starts promptly

**Answer, 23 November 2026:** C, our own picker. The user's words: "I still
think there is value in our own, because it allows scheduling to be more
dynamic, i.e. we may provision to the cloud, which would not show up in the
self hosted option."

This reverses the decision of 3 November 2026 in
[Jobs that use ciproxy](../docs/protocol/labels.md#jobs-that-use-ciproxy).
That decision rejected "a label router that picks a provider label before
each job queues, because it needs a router job and an edit in each job".
The ciproxy label stays the main interface. The picker is an addition for
the workflows that want a fallback.

The other approaches, and why they lost:

- A. A docs guide for a community picker action, such as
  `mikehardy/runner-fallback-action`. It reads only the list of self-hosted
  runners, so it cannot see that ciproxy can start a Boat VM. Each customer
  must also keep a token that can read the runners.
- B. ciproxy writes one organization variable for each label. All jobs of a
  burst read the same value. The value goes stale when ciproxy is down. It
  needs the App permission "Variables: write", which can change every
  organization variable.
- D. A macOS overflow backend that bills by the minute, such as Namespace.
  It needs a second vendor account and another image, and it does not help
  when ciproxy is down.
- Left out: change nothing, because macOS jobs still wait. Hosted larger
  runners in the ciproxy runner group, because GitHub Free has none and
  GitHub does not document the routing. A ciproxy runner that runs the job
  on a hosted runner, because job code and secrets must go only from GitHub
  to the runner. Cancel and re-run, because GitHub cannot cancel one job.
  Macs rented by the day, because the 24-hour minimum cannot absorb a burst.

### 2. When a job is "soon" for the picker

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** None.
- **Background:** The picker must decide whether ciproxy can start the job
  soon enough. Linux labels have the workspace overflow wait, from 0 to 120
  seconds, before Boat starts. macOS labels have no overflow wait.
- **Problem:** No requirement says when ciproxy may answer with its own
  label, or how long a picked job may wait for a ciproxy runner.
- **Impact if no action:** A rule that is too strict sends jobs to paid
  hosted runners while a Mac is about to be free. A rule that is too loose
  sends jobs to a full queue with no fallback.
- **Root question:** When may ciproxy answer with its own label, and on
  what evidence?
- **Options:**
  - **A.** Only "now": a free place after the queued jobs and the other
    picks, a ready warm runner, or a Boat start. This is the simplest rule,
    but it falls back more often. Effort: Small.
  - **B.** The workflow gives a wait, such as `wait: 60s`, and the default
    is "now". ciproxy answers with its label when it predicts a place
    within the wait. A wrong prediction sends the job to a full queue with
    no fallback. Effort: Medium.
  - **C.** A setting for each label in the dashboard. The customer changes
    it in one place, but it needs a dashboard change and a prediction.
    Effort: Medium.
  - **D.** The workflow gives a wait, and the default is "now". During the
    wait, the action asks again every few seconds. ciproxy answers with its
    label only for a place that it sees free. The picker job runs for up to
    the wait on a paid runner. Effort: Small.
  - **E.** The picker uses the workspace overflow wait, with the loop of D.
    It needs no new setting, but it changes the meaning of "Start on Boat
    after a job waits" and of the job-routing rule that no overflow wait
    applies to macOS. Effort: Medium.
- **Left out:** A limit in queued jobs, not in seconds, because one job
  ahead on a Mac means the wait of one whole job.
- **Wider fix:** None, because this is a design question.
- **Previous examples:** None found.
- **Recommendation:** D. It keeps the `wait` input and the "now" default,
  and no wrong prediction can send a job to a full queue. The plan first
  recommended B. The options check found D.
- **Action:** ask the user, because this adds behaviour at the boundary
  that no requirement covers.

### 3. Linux labels that allow Boat

Moved to the design calls on 23 November 2026, because the labels doc and
the user's reason in decision 1 settle it.

### 4. What the dashboard shows for fallback jobs

- **Severity:** Low. **Effort:** Medium.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** None.
- **Background:** The Jobs page, the live queue, and the savings report
  show only jobs that ran on ciproxy. ciproxy records each pick that
  reaches it. A timeout, an outage, or a missing token never reaches it.
- **Problem:** No requirement says what the customer sees about picks and
  fallbacks.
- **Impact if no action:** The customer cannot see how often ciproxy was
  full, or that one more Mac would help.
- **Root question:** What must the customer see about fallbacks to add a
  machine, fix a workflow, or judge the cost?
- **Options:**
  - **A.** Show nothing. GitHub shows the runner of each job. Effort:
    Small.
  - **D.** The action writes the answer and the reason of each fallback
    into the GitHub job summary. It gives no total over time. Effort:
    Small.
  - **B.** Show the number of picks and fallbacks for each label, for the
    last 30 days, in Settings under Labels. Effort: Medium.
  - **E.** Show the counts of B on the Machines page, for every role. It
    needs the same work as B. Effort: Medium.
  - **G.** Add one picker-job minute at the `ubuntu-slim` price to "the
    cost with ciproxy" in the savings report. It changes a requirement of
    [Metering](../docs/protocol/metering.md). Effort: Medium.
  - **C.** Show each fallback on the Jobs page with its reason. The savings
    report also needs a rule for these jobs. Effort: Large.
  - **F.** Read the jobs of each picked run with a fallback through the
    existing "Actions: read" permission. Show each fallback job with its
    label, times, and result, and price it in the savings report. Effort:
    Large.
- **Left out:** The live queue, because it shows the current state, and a
  fallback is a past event.
- **Wider fix:** None, because this is a design question.
- **Previous examples:** None found.
- **Recommendation:** B. A count for each label shows best when one more
  machine would help. D can join B at Small effort, because only the
  action sees the fallbacks that never reach ciproxy.
- **Action:** ask the user, because this is new user-visible behaviour.

### 5. What a pick does to routing

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** [Job routing](../docs/protocol/job-routing.md).
- **Background:** ciproxy starts runners for waiting jobs and for the warm
  runner count. A Linux job goes to Boat only after the overflow wait. The
  picked jobs queue a few seconds after the pick.
- **Problem:** No requirement says whether a pick starts a runner before
  its job queues, or only counts against the free places of later picks.
- **Impact if no action:** Two picks can both see the same free place, and
  one job then waits with no fallback.
- **Root question:** What does a pick change in routing before its jobs
  queue?
- **Options:**
  - **A.** A pick only counts. For 2 minutes, the endpoint subtracts it
    from the free places of later picks. The scaler does not change. The
    job waits for a VM to start after it queues. Effort: Small.
  - **B.** A pick starts runners on own machines, but never on Boat. This
    changes the routing rules for own machines. Effort: Medium.
  - **C.** A pick is demand everywhere, Boat included. A pick that gets no
    job can cost the customer a Boat start fee. Effort: Medium.
  - **D.** A separate label and scale set for picked jobs, so that no other
    job can take a held place. It adds a scale set for each label and
    organization. Effort: Large.
- **Left out:** None.
- **Wider fix:** None, because this is a design question.
- **Previous examples:** None found.
- **Recommendation:** A. It changes no routing rule in force and adds no
  Boat cost.
- **Action:** ask the user, because B, C, and D change job-routing rules
  in force, and A leaves a race that the user may not accept.

### 6. A run that has no OIDC token

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** None.
- **Background:** The action proves its repository with a GitHub OIDC
  token. A pull request from a fork gets no token, and a Dependabot run
  probably gets none.
- **Problem:** No requirement says which label such a run gets.
- **Impact if no action:** The action cannot be built.
- **Root question:** Where does a run go when it cannot ask ciproxy?
- **Options:**
  - **A.** The fallback label. These runs always pay for a hosted runner,
    also when own machines are free. Effort: Small.
  - **B.** The ciproxy label. These runs wait as today. Effort: Small.
  - **C.** An action input chooses, with A as the default. Effort: Small.
- **Left out:** None.
- **Wider fix:** None, because this is a design question.
- **Previous examples:** None found.
- **Recommendation:** A. A run that cannot ask ciproxy is like a run that
  gets no answer.
- **Action:** ask the user, because A and B have real trade-offs between
  cost and wait.

### 7. A limit on picks

- **Severity:** Low. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** None.
- **Background:** Any workflow of an allowed repository can call the pick
  endpoint. A refused pick sends the job to a paid hosted runner.
- **Problem:** No requirement limits picks. A limit is a new rejection.
- **Impact if no action:** A broken workflow can call the endpoint in a
  loop.
- **Root question:** How does ciproxy stop one workflow from flooding the
  picker?
- **Options:**
  - **A.** No rejection. One pick for each run attempt and label. A repeat
    gets the stored answer. A `count` above 256, the GitHub matrix limit,
    counts as 256 with a warning. Effort: Small.
  - **B.** A limit for each repository, with a stated value. Effort: Small.
  - **C.** A limit for each workspace, with a stated value. Effort: Small.
- **Left out:** None.
- **Wider fix:** None, because this is a design question.
- **Previous examples:** None found.
- **Recommendation:** A. It stops repeated picks and refuses no real pick.
- **Action:** ask the user, because B and C add a new rejection.

## Design calls

My calls, 23 November 2026:

- Linux labels that allow Boat: the picker counts a Boat start as a place.
  The user named "provision to the cloud" as the reason for the picker,
  and [Own machines, Boat, or both](../docs/protocol/labels.md#own-machines-boat-or-both)
  already lets the customer turn Boat off for a label. Rejected: GitHub in
  place of Boat, because it ignores the Boat switch. Rejected: an action
  input `boat: false`, because it adds a second control beside the Boat
  switch. A Boat refusal after the pick still leaves the job waiting.
- The picker is a GitHub Action under `actions/pick/`, not a `ciproxy pick`
  CLI. A CLI needs a download and a release for each platform. The action
  is a composite action with `curl` and `jq`, so the repository holds no
  bundled JavaScript. GitHub still downloads the repository archive for
  each run, about 3.4 MB.
- `actions/` gets its own Apache-2.0 `LICENSE`, because it runs on the
  customer side.
- The action proves its repository with a GitHub OIDC token with the
  audience `ciproxy`. It needs no secret and gives ciproxy no rights. The
  endpoint refuses a public repository with the claim
  `repository_visibility`, as
  [Repository access](../docs/protocol/security.md#repository-access) says.
- The key cache and the token checks of `ciproxy-cache-sidecar` move into
  a shared open crate with a neutral error type. The sidecar and the
  control plane both use it. The picker adds the audience check. The
  issuer check also accepts the enterprise issuer form
  `https://token.actions.githubusercontent.com/<enterpriseSlug>`.
- The endpoint lives on the control-plane host, outside the dashboard API,
  at a versioned path such as `/pick/v1`. Its callers are workflows, not
  dashboard sessions.
- The workflow names the label and the fallback label. The action needs
  the fallback label when ciproxy does not answer, so ciproxy cannot own
  it.
- The action prints the fallback label, with a warning, on a timeout, an
  error, an unknown repository, a repository without ciproxy access, a
  public repository, an inactive organization, a suspended App, or an
  unknown label. Each case is the same as no answer. The timeout of one
  request is 10 seconds, so a down control plane costs the picker job at
  most 10 seconds over the wait of decision 2.
- The workflow puts the ciproxy label after `||` and adds
  `if: ${{ !cancelled() }}`. GitHub skips a job whose needed job failed, so
  the `||` default works only with that condition. A job with other needs
  must also check their results. The guide says so.
- One pick serves the jobs of one label that queue right after the picker.
  One picker job for a matrix costs less than one for each job. The input
  `count` gives the number of jobs. The output `runners` is a JSON list
  with one label for each job, read at `strategy.job-index`. The output
  `runner` is the first item. The guide names the jobs that a pick does
  not serve: a later wave, a second label, and a "Re-run failed jobs" run,
  which reuses the old answer.
- A job on a fallback label is not a ciproxy start, so it gets no charge.
  [Starts](../docs/protocol/metering.md#starts) already says so.
- The guide recommends `ubuntu-slim` for the picker job. It costs a third
  of `ubuntu-latest`, and a pick needs much less than its 15-minute limit.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

Decision 4 chooses whether this milestone has work. With option A or D,
delete this milestone. Otherwise mock the screens that the option names.

- [ ] Wait for decision 4.
- [ ] Mock the fallback counts, or the fallback jobs, as decision 4 says.
- [ ] Run the mockup check named in `AGENTS.md`. Smoke-test the changed
      pages.
- [ ] Update the design and the TODOs of this plan for each problem that
      the mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

The protocol docs state the picker requirements and decisions 1 to 7.

- [ ] Wait for decisions 2, 5, 6, and 7.
- [ ] Add `docs/protocol/picker.md`: the request, the answer, the wait,
      the fallback cases, what a pick changes in routing, and the limit.
- [ ] Update [Jobs that use ciproxy](../docs/protocol/labels.md#jobs-that-use-ciproxy)
      with decision 1.
- [ ] Update `docs/protocol/job-routing.md` with decision 5, and with the
      Boat start as a place for a picked Linux job.
- [ ] Update `docs/protocol/security.md` with the OIDC proof of the
      picker. State that it adds no App permission.
- [ ] Add the retention period of the stored picks to
      `docs/protocol/security-data.md`.
- [ ] Add the picker to `docs/architecture.md`, under the components, the
      boundaries, and the open-source boundary.
- [ ] Add `actions` and `crates/ciproxy-image-vm` to
      [Licence](../README.md#licence). The second one is missing today.

## Milestone 3: Shared OIDC token checks

Depends on: 2

The GitHub token checks of the cache sidecar move into an open crate that
the control plane can also use.

- [ ] Move the key cache, the key source, and the issuer check into an
      open crate with its own Apache-2.0 `LICENSE` and a neutral error
      type.
- [ ] Add the audience check and the enterprise issuer form.
- [ ] Keep the sidecar tests green, and add tests for the new checks.

## Milestone 4: The pick endpoint

Depends on: 3

The control plane answers a pick from the token, the routing state, and
the Boat state.

- [ ] Map the repository owner to an organization and a workspace. Check
      the repository access, the visibility, and the label.
- [ ] Count the free places after the queued jobs and the active picks of
      the label and organization. Add a ready warm runner and a Boat start
      when the label allows Boat.
- [ ] Answer as decision 2 says. Apply decision 5 and decision 7.
- [ ] Store each pick, with a migration, and delete it after its
      retention period.
- [ ] Add tests for each answer and each refusal.
- [ ] Update the README of each changed crate.

## Milestone 5: Picks in the scaler

Depends on: 4

Decision 5 chooses whether this milestone has work. With option A, delete
this milestone.

- [ ] Wait for decision 5.
- [ ] Add the picks to the snapshot as decision 5 says. Match a queued job
      to its pick by run and label in the snapshot builder.
- [ ] Add tests for each routing change.
- [ ] Update `crates/ciproxy-scaler/README.md`.

## Milestone 6: The picker action

Depends on: 2

The composite action asks ciproxy and prints the label for each job.

- [ ] Add `actions/LICENSE`, and `actions/pick/action.yml`, its script,
      and its README.
- [ ] Apply decision 6 to a run without a token.
- [ ] Add a test of the script with a stub endpoint: the ciproxy answer,
      the fallback answer, the wait loop of decision 2, a timeout, an
      error, a missing token, and a matrix count.
- [ ] Probe on a test repository: `runs-on` reads the `runners` list at
      `strategy.job-index`, and `ubuntu-slim` gets an OIDC token. Check
      whether a Dependabot run gets one.
- [ ] Add the release of the action: the `v1` tag and the rule that moves
      it.

## Milestone 7: User guide

Depends on: 4, 5, 6

The site docs explain the picker for customers.

- [ ] Add a site docs page for the picker, with the example workflow, the
      cost of the picker job, the fallback cases, and the jobs that a pick
      does not serve.

## Milestone 8: Fallback data

Depends on: 1, 4

Decision 4 chooses whether this milestone and Milestone 9 have work.

- [ ] Wait for decision 4.
- [ ] Add the data of decision 4 to the dashboard API and regenerate the
      API types.

## Milestone 9: Fallback data in the dashboard

Tags: ui

Depends on: 8

The dashboard shows the fallback data as the mockups of Milestone 1 show
it.

- [ ] Build the screens of Milestone 1.

## Milestone 10: Verification, commit, review, and pull request

Depends on: every other milestone

The complete gate, the review, and the pull request.

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

- [ ] Run the picker on a test repository against production while the
      test Mac is busy. Check that the job runs on `macos-latest`.
