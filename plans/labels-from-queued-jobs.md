# Labels from queued jobs

Status: Active. On branch
`calummoore/github-actions-workflow-commit-main-ga4d`, no pull request yet.
The pull request that completes the work deletes this plan, as the `plans`
skill says.

A job uses ciproxy when its `runs-on` names a ciproxy label. GitHub sends
each scale set only the jobs of its own label name. So a job whose label
the workspace lacks, such as a typo or a new name, never reaches ciproxy.
The job waits silently, and GitHub cancels it after 24 hours. This change
lets ciproxy see such jobs through the GitHub `workflow_job` webhook. When
a missing label starts with `ciproxy`, ciproxy creates it from a template
label that a prefix rule picks. When ciproxy cannot create it, the
dashboard warns. ciproxy ignores every label that does not start with
`ciproxy`, because other runners serve those labels. The change adds no
GitHub App permission. It does not read workflow files, and it does not
re-run or cancel jobs.

Contract owners:

- [Labels](../docs/protocol/labels.md).
- [Security](../docs/protocol/security.md), "GitHub App permissions".
- The [`ciproxy-github` README](../crates/ciproxy-github/README.md).

## Decisions

User decisions, 23 November 2026, from a brainstorm in this workspace.
The brainstorm notes are under `.context/unknown-label-detection/`.

- **1. How ciproxy detects a job that waits for a label that no runner
  serves.** Chosen: **A**, the `workflow_job` webhook. The App already has
  "Actions", read, which allows this event, and the App already takes
  webhooks. Rejected: **B**, "Contents", read, and the `push` webhook,
  because it lets the App read all source code, every install must accept
  a new permission, and a static check cannot resolve `runs-on` values such
  as `${{ matrix.os }}`. Rejected: **C**, a customer-side lint, because it
  works only when the customer adds it. Left out: polling the REST API,
  because it gives the same data later and costs rate limit.
- **2. Whether ciproxy creates a missing `ciproxy` label.** Chosen: **A**,
  auto-create, on by default, with a dashboard notice. Rejected: **B**,
  off by default, because most customers never find the setting. Rejected:
  **C**, a one-click "Create label" button only, because the workflow then
  is not the only setup step. Accepted risk: a typo becomes a real label,
  and any person with push access to an allowed repository can add labels.
- **3. Where the template lives.** Chosen: **A**, the template is an
  existing label, and a new label copies its platform and size. Rejected:
  **B**, a separate form with its own size, because it duplicates the label
  dialog. Rejected: **C**, always copy `ciproxy`, because the customer
  cannot choose.
- **4. Names for macOS.** Chosen: **A**, more than one template, with the
  user's change: the customer controls the prefixes in Settings, for
  example "`ciproxy-macos-` uses `ciproxy-macos`". Rejected: **B**, always
  the template platform, because a macOS job then gets a Linux VM and
  fails. Rejected: **C**, no auto-create for `macos` names, because the
  user wants these names to work too.
- **5. Whether auto-create brings back a deleted label.** Chosen: **A**,
  yes, because the workflow still asks for it. Rejected: **B**, a warning
  only.

### Design calls

My calls, 23 November 2026:

- **Switch.** Settings, under Labels, has a switch "Create labels from
  jobs". It is on in each workspace. When it is off, a matching job gets
  the warning only. Reason: "on by default" in Decision 2A needs a way to
  turn it off.
- **Prefix rules.** Settings, under Labels, gets a "New labels" list. Each
  rule has a prefix and a template label. The longest matching prefix
  wins. Every prefix starts with `ciproxy`. A workspace starts with two
  rules: `ciproxy` uses `ciproxy`, and `ciproxy-macos` uses
  `ciproxy-macos`. The `ciproxy` rule cannot be removed. Reason: this joins
  Decision 3A and Decision 4A. The template stays a label, and the
  `ciproxy` rule is the "use for new labels" choice of Decision 3A.
- **Name match.** ciproxy compares label names without case, as GitHub
  does. The unique index of active label names then uses the lower-case
  name. Reason: GitHub matches `runs-on` labels without case, and
  `ciproxy-Foo` and `ciproxy-foo` must give one label. Today the index
  `labels_active_workspace_name` is case-sensitive.
- **One label for each job.** ciproxy acts only on a job whose `runs-on`
  has exactly one label. A job with more labels gets the warning, with the
  reason that a ciproxy job names one label. Reason: each ciproxy scale set
  has one label, so GitHub never gives such a job to a ciproxy runner.
- **Events.** ciproxy acts only on the `queued` action of `workflow_job`.
  It ignores `in_progress` and `completed`. It ignores a job whose label
  the workspace already has. Reason: only a queued job can wait for a
  label.
- **Template values.** A new label copies every value of its template:
  the platform, the size, "Your machines", "Boat", and "When nothing
  fits". Reason: the template then fully describes the new label.
- **The macOS rule stays.** The customer can change the template of the
  `ciproxy-macos` rule, but cannot remove the rule. Reason: without it,
  macOS names fall to the `ciproxy` rule and get a Linux label, which
  Decision 4 rejected.
- **Repository access.** Only jobs of the repositories that the workspace
  allows can create a label or a warning. Public repositories never count.
  Reason: [Repository access](../docs/protocol/security.md#repository-access)
  already states this for jobs.
- **Name check.** An automatic label gets the same GitHub name check as a
  label from the dashboard. When GitHub refuses the name, ciproxy shows the
  warning and creates no label. Reason: the labels doc lets GitHub decide
  each name.
- **One label for many jobs.** Jobs that queue at the same time with the
  same missing name create one label. Reason: a label name is unique in a
  workspace.
- **Notice.** The dashboard tells the workspace about each automatic label:
  its name, its template, and the repository and workflow of the job. The
  mockup milestone sets the place and the words.
- **One create path.** The dashboard and the webhook create a label
  through one function: the GitHub name check, the store write, and the
  scale set reconcile. Today this flow lives in the API handler
  `crates/ciproxy-api/src/handlers/labels.rs`, `create`. Reason: both
  paths then keep the same checks.
- **Origin.** A label row records where it came from: the dashboard, or a
  job with its repository and workflow name. Reason: the notice and the
  label list show it.
- **Missing labels.** ciproxy records each `ciproxy` name that it could
  not create, with the reason, the repository, the workflow, the last time
  that it saw the name, and a count. A Jobs notice shows each record until
  the label exists or 24 hours pass after its last job. Reason: GitHub
  cancels the job after 24 hours.
- **Lost deliveries.** The `workflow_job` event joins the event list of
  the webhook recovery in `crates/ciproxy-github/src/redelivery.rs`.
  Recovery skips a `workflow_job` delivery older than 24 hours. Reason: a
  lost delivery otherwise leaves the job waiting, and GitHub cancels the
  job after 24 hours. The recovery pass reads the whole delivery history,
  and `workflow_job` adds about three deliveries for each job. Milestone 6
  checks that the pass stays inside the GitHub rate limit.
- **No new permission.** The App subscribes to the `workflow_job` event in
  its GitHub settings. "Actions", read, already allows it, so existing
  installs accept nothing new. Reason: GitHub ties the event to that
  permission.

### Open decisions

These decisions need the user's answer. The milestones that depend on
them say so.

#### 6. A prefix rule whose template label is deleted

The options check added E and F. It changed the recommendation from C to
E.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-store-pg/src/dashboard/resources.rs:25`,
  `retire`.
- **Background:** A prefix rule names a template label. The customer can
  delete every label except `ciproxy`, also `ciproxy-macos`. A delete is a
  soft delete: `retire` sets `deleted_at`, turns off "Your machines" and
  "Boat", and keeps the platform and the size. Decision 5A says that
  auto-create brings back a deleted label.
- **Problem:** After a delete, the rule names a label that is not active.
- **Impact if no action:** a job that matches the rule gets no label and
  no clear reason, or it falls to the `ciproxy` rule and gets the wrong
  platform.
- **Root question:** Which platform and size does a rule give its new
  labels after the customer deletes its template label?
- **Options:**
  - **A.** The rule stays and shows "No template". A matching name gets the
    warning and no label. The customer picks a new template. Effort:
    Small.
  - **B.** The rule moves to the `ciproxy` template. A macOS name then gets
    a Linux label, and its jobs fail. Effort: Small.
  - **C.** The delete dialog lists the rules that use the label, and the
    customer picks a new template or "No template" for each. Effort:
    Medium.
  - **D.** Each rule keeps a copy of the platform and size of its template.
    A delete leaves the copy in place, so the deleted label can come back
    with its last size. Effort: Medium.
  - **E.** The rule keeps its link to the deleted label. ciproxy uses an
    active label of the same name when one exists. Otherwise it copies the
    platform and size of the deleted row, with "Your machines" and "Boat"
    as the platform starts them for a new label. The rule list marks the
    template as deleted. Cost: the deleted template keeps giving its last
    size until the customer picks a new one. Effort: Small.
  - **F.** ciproxy refuses to delete a template label until the customer
    changes each rule that uses it. The refusal names the rules. Cost: a
    new refusal on an existing path, and a labels doc change. Effort:
    Small.
- **Left out:** None.
- **Wider fix:** E. It reads the saved label row, so it works for every
  delete path and every rule.
- **Previous examples:** None found.
- **Related:** Decision 5A. Options D and E bring back a deleted template
  label with its old size.
- **Recommendation:** E. The soft delete already keeps the values, so no
  job gets the wrong platform and the delete needs no new step. The first
  recommendation was C.
- **Action:** ask the user, because the options have real trade-offs and
  no requirement covers this boundary.

#### 7. A limit on automatic labels

The options check widened the root question and added D, E, and F. It
changed the recommendation from B to D.

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Security.
- **Location:** `crates/ciproxy-store-pg/src/scale_sets.rs:123` and
  `crates/ciproxy-control-plane/src/listeners.rs:171`.
- **Background:** Any person with push access to an allowed repository can
  make ciproxy create a label. Each label gets one GitHub scale set in each
  organization of the workspace. Each scale set runs one listener in the
  control plane, and each listener keeps one Postgres connection. A
  listener is the task that reads the job messages of one scale set.
- **Problem:** Nothing limits how many labels jobs can create. For
  example, `runs-on: ciproxy-${{ github.head_ref }}` creates one label for
  each branch.
- **Impact if no action:** one workspace can fill its label list and use
  up control plane connections that all workspaces share.
- **Root question:** How does ciproxy limit what labels from jobs cost,
  for the workspace and for the shared control plane?
- **Options:**
  - **A.** No limit. The customer deletes unwanted labels. Effort: Small.
  - **B.** At most 10 automatic labels in each 24 hours for each
    workspace. After that, a matching job gets the warning only. Cost: a
    slow stream of names has no end. Effort: Small.
  - **C.** At most 25 labels in a workspace in total, automatic or not.
    After that, a matching job gets the warning only. Cost: the dashboard
    also refuses a new label at the limit. Effort: Small.
  - **D.** Like C, but the limit stops only automatic labels. ciproxy
    counts every active label of the workspace. At 25, a matching job gets
    the warning only. An Admin can still add labels in the dashboard.
    Effort: Small.
  - **E.** Past the limit, the missing label notice shows a "Create label"
    button to an Admin. Cost: an API action, a mockup state, and a button.
    Effort: Medium.
  - **F.** ciproxy deletes an automatic label that gets no job for 30
    days. Decision 5A brings it back when a job asks for it again. Cost: a
    periodic sweep, lost size edits, and the first job wait of Decision 8
    at each return. Effort: Medium.
- **Left out:** owner approval of each label, because Decision 2A chose no
  manual step. Only jobs of the default branch, because a new label
  usually first appears in a pull request.
- **Wider fix:** D. A total limit stops both a burst and slow growth.
- **Previous examples:** None found.
- **Recommendation:** D. It bounds the shared control plane cost and adds
  no refusal to the dashboard. The first recommendation was B. E can come
  later.
- **Action:** ask the user, because a limit is a new rejection that no
  requirement covers.

#### 8. Jobs that wait before their label exists

The options check widened the root question and added D and E. It changed
the recommendation after the spike from A to E or B.

- **Severity:** Medium. **Effort:** depends on the spike in Milestone 3.
- **Source:** Design.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-github/src/scale_sets/queue.rs:111`.
- **Background:** ciproxy creates a label and its scale set after a job
  queues. It is not known if GitHub then gives that job to the new scale
  set. This also holds for each job of a matrix, for jobs that waited
  while the switch was off, and for a deleted label that comes back. In
  [issue 4715](https://github.com/actions/actions-runner-controller/issues/4715)
  of the GitHub runner controller, an existing scale set had no capacity.
  Its queued jobs waited until one new runner started on it. The listener
  reads the job counts that GitHub sends with each message, but no code
  uses them.
- **Problem:** If GitHub does not give a waiting job to the new scale set,
  the job waits up to 24 hours, and then GitHub cancels it.
- **Impact if no action:** the customer sees a new label, but the first run
  of the label fails.
- **Root question:** How does a job that already waits when ciproxy
  creates its scale set get a runner?
- **Options:**
  - **A.** Nothing more. The notice tells the customer to cancel and re-run
    the run. Effort: Small.
  - **B.** ciproxy starts a runner for the new scale set at once. The
    runner takes the waiting job if GitHub offers it. Cost: one VM start
    for each new scale set. Effort: Medium.
  - **C.** Ask for "Actions", write. Cancel the run, and re-run it. Cost:
    every install must accept the new permission, and the cancel also stops
    the other jobs of the run. Effort: Medium.
  - **D.** ciproxy reads the job counts when the session opens and with
    each message. When GitHub counts more jobs than ciproxy tracks, it
    starts one runner for each extra job. Cost: it helps only if the counts
    include a job that queued before the scale set existed. Effort: Medium
    if B's runner path exists, else Large.
  - **E.** After ciproxy creates a scale set, it registers one runner on it
    and starts no VM. It removes the registration after a few minutes. If
    GitHub then offers the waiting job, the normal job path starts a real
    runner. Cost: none in VMs. Risk: it is not known if a registration
    alone wakes the job. Effort: Small.
- **Left out:** a signal before the job queues, because no event names the
  labels before `queued`, and Decision 1 rejected reading workflow files.
- **Wider fix:** D, if the spike shows that the counts include the waiting
  job. It also covers a listener restart.
- **Previous examples:** None found.
- **Recommendation:** decide after the spike. If GitHub gives the job to
  the new scale set, no option is needed. Otherwise E, if one registration
  wakes the job, or else B. The first recommendation was A, but then the
  first run of each new label fails.
- **Action:** ask the user, because the answer depends on the spike.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The Settings Labels tab gets the switch and the prefix rules. The label
list shows automatic labels. The Jobs page gets the notices for automatic
and missing labels. Each screen has a mobile and a web variant.

- [ ] Add the "New labels" section to
      `docs/mockups/src/screens/settings/labels.tsx`: the switch "Create
      labels from jobs" and the list of prefix rules.
- [ ] Add a dialog screen that adds or edits a prefix rule: the prefix and
      the template label. Show the state where the prefix does not start
      with `ciproxy`.
- [ ] Mark an automatic label in
      `docs/mockups/src/screens/settings/label-list.tsx` with its
      repository and workflow.
- [ ] Show a rule whose template label is deleted, as Decision 6
      chooses. This TODO waits for Decision 6.
- [ ] Add the Jobs notices: a new automatic label, and a missing label for
      each reason. The reasons are the switch is off, GitHub refused the
      name, the job names more than one label, and the limit of Decision
      7. The limit state waits for Decision 7.
- [ ] Add the Mokly entries under
      `docs/mockups/src/entries/app/settings/` and
      `docs/mockups/src/entries/app/jobs/`. Keep each page at five screens
      or fewer.
- [ ] Run `npm --prefix ts run mockups:check`. Smoke-test the changed
      pages.
- [ ] Update the design and the TODOs of this plan for each problem that
      the mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

The protocol docs state the new requirements and the decisions in force.
The crate docs state the new webhook event.

- [ ] Add a section "Labels from jobs" to `docs/protocol/labels.md`: the
      detection, the switch, the prefix rules, the origin, the notices, and
      Decisions 2 to 7 with their rejected options. Change "The customer
      adds a label in the dashboard" to name both ways.
- [ ] Update `docs/protocol/security.md`, "GitHub App permissions": the
      "Actions" read permission also gives the `workflow_job` event, and
      ciproxy reads only its labels, repository, and workflow name.
- [ ] Update the App settings list in `crates/ciproxy-api/README.md` and
      `crates/ciproxy-control-plane/README.md`: subscribe to
      `workflow_job`.
- [ ] Update `crates/ciproxy-github/scale-sets.md` and
      `crates/ciproxy-control-plane/github-workers.md`, which say that the
      App needs no optional event subscription.
- [ ] Update `crates/ciproxy-github/README.md` with the new event and the
      create path. Fix its stale list of three default labels to five, at
      the label count and at the free job disk of each default label.
- [ ] Update the site docs `ts/site/src/content/docs/docs/reference/labels.mdx`
      and `ts/site/src/content/docs/docs/reference/github-app.md`.

## Milestone 3: Spike, the first job of a new label

Depends on: none

This answers the unknown fact of Decision 8. It uses a test GitHub App
and a test organization, and it changes no product code.

- [ ] Queue a job with a label that has no scale set. Then create the
      scale set with the code of `crates/ciproxy-github/src/app/bindings.rs`
      and open its message session.
- [ ] Record whether GitHub sends the waiting job to the new scale set,
      and after how long. Record the job counts that GitHub sends with the
      session and with each message.
- [ ] When GitHub does not send the job, register one runner on the scale
      set and start no VM. Record whether GitHub then offers the job.
- [ ] When that does not wake the job, start one runner, and record whether
      it takes the job.
- [ ] Save the evidence under `.context/labels-from-queued-jobs/`. Add the
      result to Decision 8, and ask the user for the answer.

## Milestone 4: Store

Depends on: 2

- [ ] Add a migration: the workspace switch on the settings row, the
      prefix rule table, the label origin columns, and the missing label
      table. Seed the two starting rules for each workspace. Make the unique
      index of active label names use the lower-case name.
- [ ] Add the starting rules where `default_labels` in
      `crates/ciproxy-github/src/app/installation.rs` makes the default
      labels.
- [ ] Add the store interface and the Postgres code for the rules, the
      switch, the origin, and the missing labels, with tests.
- [ ] Add the store side of Decision 6 to the label delete in
      `crates/ciproxy-store-pg/src/dashboard/resources.rs`, `retire`.
      This TODO waits for Decision 6.
- [ ] Add the count of Decision 7, under the workspace lock. This TODO
      waits for Decision 7.
- [ ] Add a store test with two names that differ only in case.

## Milestone 5: One create path

Depends on: none

This keeps the behaviour of the dashboard.

- [ ] Move the create flow out of `crates/ciproxy-api/src/handlers/labels.rs`,
      `create`, into one function that the API and the webhook call.
- [ ] Keep the API tests green, and add a test of the moved function.

## Milestone 6: The `workflow_job` webhook

Depends on: 4, 5

- [ ] Add the `workflow_job` payload type to
      `crates/ciproxy-github/src/app/webhook.rs`. Read only the action, the
      labels, the installation, the repository, and the workflow name.
- [ ] Dispatch the event in `crates/ciproxy-github/src/hooks.rs`, and add it
      to the event list in `crates/ciproxy-github/src/redelivery.rs`.
- [ ] Find the workspace from the installation. Skip a repository that the
      workspace does not allow, and skip a public repository.
- [ ] For a job with one missing `ciproxy` label, pick the rule, then
      create the label through the create path, or record a missing label.
      Record a job with more than one label as a missing label.
- [ ] Check the limit of Decision 7 before the GitHub name check, because
      the name check already creates the scale sets. Check it again under
      the workspace lock.
- [ ] Skip a `workflow_job` delivery older than 24 hours in the recovery
      pass. Count the extra history pages that the event adds, and check
      that the pass stays inside the GitHub rate limit.
- [ ] Test the cases: an existing label, a new label, the macOS rule, the
      switch off, a refused name, a job with two labels, two names that
      differ only in case, two jobs with one new name, a deleted template,
      a deleted label that comes back, and the limit.

## Milestone 7: API

Depends on: 4

- [ ] Add the endpoints that read and change the switch and the prefix
      rules. Add the origin to the label view and the missing labels to the
      Jobs data.
- [ ] Add the API side of Decision 6 to the label delete. This TODO waits
      for Decision 6.
- [ ] Run `npm --prefix ts run api:generate`.

## Milestone 8: Dashboard

Tags: ui

Depends on: 1, 7

- [ ] Build the "New labels" section and the rule dialog in
      `ts/app/src/screens/settings/`, as the mockups show.
- [ ] Show the origin in the label list and the delete dialog of
      Decision 6.
- [ ] Add the notices to `ts/app/src/view/jobs-notices.ts` and
      `ts/app/src/screens/jobs/live-notices.tsx`.
- [ ] Add tests beside the view code, and check the screens in a browser.

## Milestone 9: The first job, after the spike

Depends on: 3, 6

- [ ] Build the option of Decision 8 that the user chooses. Remove this
      milestone when the spike shows that GitHub sends the waiting job to
      the new scale set.

## Milestone 10: Verification, commit, review, and pull request

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

- [ ] Subscribe the production GitHub App to the `workflow_job` event in
      its GitHub settings.
- [ ] Smoke-test in production: push a workflow with a new `ciproxy` label
      to an allowed repository, and check the label and the notice.
