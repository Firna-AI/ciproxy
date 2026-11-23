# Machine tags in the label order

Status: Active. On branch `calummoore/machine-group-creation-5ao7`, no pull
request yet. The pull request that completes the work deletes this plan, as
the `plans` skill says.

A customer cannot choose which of their own machines take a label's jobs
first. The scaler starts a runner on the first own machine that fits, in an
order that the customer does not see or set. A label also cannot leave some
own machines out. This change adds machine tags. A machine can have several
tags, and each tag can be a place in the label order. A label can then run
on the `hetzner` machines first, then on the `gcp` machines, then on Boat.
A label that lists only `hetzner` never runs on other own machines.

The label order comes from the unmerged plan
`plans/overflow-provider-interface.md` on branch
`calummoore/ciproxy-providers-interface-s8rp`. That plan adds a label order
of "own machines" and providers. This plan splits "own machines" into "All
your machines" and one place for each tag. It does not cover a cost or speed
figure for each machine, or tags that the install command sets.

The request came on 23 November 2026: "allow the creation of machine groups,
so that you can control within your own hosted machines the
ordering/priority of different options". The user's answers are under
[User decisions](#user-decisions).

Contract owners:

- [Labels](../docs/protocol/labels.md): the label order and its places.
- [Job routing](../docs/protocol/job-routing.md): where a job runs, and the
  warm runners.
- [Agent](../docs/protocol/agent.md): the machine page and its tags.
- The [scaler README](../crates/ciproxy-scaler/README.md).

## Design

### Tags and places

- A machine has zero or more tags. The customer sets them on the machine
  page.
- A label order entry is "All your machines", one tag, or one provider.
- "All your machines" matches every own machine of the label's platform.
- A tag entry matches each own machine of the label's platform with that
  tag.
- A job goes to the next entry when the current entry cannot take it now,
  as the provider plan says. Between two own-machine entries, it moves at
  once.
- Each scaler pass places a waiting job on the first own-machine entry
  with room. So a slot that frees on an earlier entry takes the next job
  there again.
- The overflow wait applies only when a job leaves own machines for a
  provider.

### My design calls, 23 November 2026

- Product copy says "tag", and the docs say "machine tag" where the word
  needs a qualifier. Reason: GitHub already uses "label" and "runner group",
  and ciproxy uses "label" for `runs-on`.
- The provider plan's "own machines" entry becomes "All your machines". The
  migration keeps it in each order. Reason: every existing label keeps its
  routing, and a machine with no tags still runs jobs.
- A machine that matches two entries of one order counts at the first.
  Reason: the order then reads from the top.
- A tag is 1 to 32 characters of lowercase letters, digits, and hyphens,
  unique in the workspace. Reason: it reads well in a list and needs no
  escaping.
- A tag exists while one machine has it or one label order lists it. "Add a
  place" offers each tag of the workspace. Reason: no separate tag screen.
- A tag entry that matches no connected machine stays in the order and is
  skipped. Its row says "No machines have this tag". Reason: this matches
  the rule for a provider that is not connected.
- When the customer removes a tag from its last machine, label orders keep
  the entry. Reason: a machine can get the tag again, and the order needs no
  edit.
- A new tag joins no label order. Reason: "All your machines" already
  covers the machine, and the customer chooses where a tag goes.
- A macOS label's order can hold "All your machines" and tags, but no
  provider. Reason: macOS never overflows, and Macs also need an order.
  This replaces the provider plan's call that a macOS order has no handle,
  Remove, or Add, and Milestone 1 changes its mockup.
- Inside one entry, the scaler keeps today's machine order. Reason: the
  user asked for control between sets of machines, not inside one.
- Tags are on every tier. Reason: they cost nothing to run. A Free
  workspace has one machine, so it gains little.
- The install command gets no tag option in this change. Reason: the
  machine page covers it, and the install option can come later.
- The branch merges the provider branch before Milestone 1. Reason: the
  mockups and docs build on the label order of that branch.

### Data shape

- `machine_tags` holds the workspace, the machine, and the tag. The pair of
  machine and tag is unique.
- `label_places` gets a tag column. A row is `machine` for "All your
  machines", a provider, or a tag. The unique key of a place becomes the
  label, the place, and the tag, so one label can list several tags.
- The scaler's `eligible` check uses the label's entries in place of the
  own-machines switch. So no runner, warm runners included, starts on a
  machine that the order leaves out.
- The scaler snapshot gives each machine its tags, and each label its order
  of entries.

### Risks

- A warm runner on a later entry can take a job before a new runner starts
  on an earlier entry. Item 3 decides the rule.
- When the order of `ciproxy` lists a provider before own machines, warm
  runners on own machines still take its jobs first. This belongs to the
  provider plan, and item 3 does not cover it.
- The label order changes routing for every label. The migration must keep
  the current behaviour of each existing label exactly.

## User decisions

- **Item 1, decided on 23 November 2026:** C, machine tags. A machine can
  have many tags, and each order entry is a tag. Rejected: A, one machine
  priority for the whole workspace, because a label cannot leave a machine
  out. Rejected: B, groups where each machine is in one group, because a
  machine cannot serve two sets. Rejected: D, one group order for the whole
  workspace.
- **Item 2, decided on 23 November 2026:** A. A job moves to the next
  own-machine entry at once. The overflow wait applies only when a job
  leaves own machines for a provider. Rejected: B, the overflow wait at each
  entry, because it slows jobs with no saving. Rejected: C, a wait for each
  entry, because it adds a setting.
- **Item 4, decided on 23 November 2026:** B. The plan, the mockups, and the
  protocol docs come now on this branch. The code starts after the provider
  branch merges. Rejected: A, wait for the merge. Rejected: C, milestones in
  the provider plan, because a large change grows larger.

## Open decisions

### 3. Warm runners can break the tag order

- **Severity:** Medium. **Effort:** Medium.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/job-routing.md:156`, "Warm runners", and
  `crates/ciproxy-scaler/src/warm.rs`.
- **Background:** Each label has its own order. Warm runners serve only
  `ciproxy` on Linux machines and `ciproxy-macos` on Macs. So the order of
  that one label is the order that the warm runners must follow. Each
  machine keeps 2 warm runners by default. GitHub gives a job of that label
  to any ready warm runner at once.
- **Problem:** When the machines are idle, every machine fills up to its
  count. A warm runner on a machine of a later entry then takes jobs before
  a new runner starts on a machine of an earlier entry.
- **Impact if no action:** Many `ciproxy` jobs ignore the order, and the
  customer does not see why.
- **Root question:** How many warm runners sit on each entry of the warm
  label's order, so that the order holds?
- **Options:**
  - **A.** Keep the count of each machine. Warm runners skip the machines
    that the order leaves out, as every runner does. The machine page tells
    the customer to set the count to 0 on the machines of later entries.
    Effort: Small.
  - **B.** Keep the count of each machine as an upper limit. A machine of a
    later entry keeps warm runners only while the machines of earlier
    entries cannot hold their counts. When room comes back on an earlier
    entry, ciproxy stops the idle warm runners of later entries. Effort:
    Medium.
  - **D.** Any label can have a warm count. ciproxy places the warm runners
    of each label by that label's order, with the count of each machine as
    an upper limit. This replaces the rule that only `ciproxy` and
    `ciproxy-macos` have warm runners. Effort: Large.
  - **E.** Each entry of the warm label's order gets its own warm count, and
    the count of each machine goes away. This replaces the decision of one
    count for each machine, and it adds a control to the label order.
    Effort: Large.
  - **F.** Machines of later entries keep booted job VMs that are not
    registered with GitHub. A VM registers only when the scaler sends a job
    to it. A job on a later entry then waits one scaler pass and the
    registration. The agent needs a new runner state. Effort: Large.
- **Left out:** No warm runners when a label lists tags, because jobs lose
  the start at once.
- **Wider fix:** B. The order then holds with no extra setup.
- **Previous examples:** None found.
- **Recommendation:** B. It keeps today's controls, and the order holds
  without extra setup. D can follow when other labels need warm runners.
- **Action:** ask the user, because B changes the warm-runner rule in
  force, and no decision covers that.

Options check, 23 November 2026: it added E and F, and brought back the old
C as E. It rewrote B, because the first text of B kept today's behaviour on
idle machines. It found that A cannot be "no change", because the scaler's
`eligible` check must skip the machines that the order leaves out. The
recommendation stays B.

### 5. What one tag entry matches

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `docs/protocol/labels.md:178`, "Own machines, Boat, or
  both".
- **Background:** Item 1 chose tags, and each entry of a label order can be
  a tag. A size check already keeps a job off a machine that is too small.
  GitHub runner labels in `runs-on` match a runner that has all the named
  labels.
- **Problem:** No rule says if one entry can name more than one tag.
- **Impact if no action:** The store shape and the order row design cannot
  be planned.
- **Root question:** How does a customer prefer a set of machines that no
  single tag names, such as the Hetzner machines in one data centre?
- **Options:**
  - **A.** One tag for each entry. The customer adds a tag such as
    `hetzner-fsn` to those machines. Effort: Small.
  - **B.** One or more tags for each entry. The entry matches the machines
    that have all of them. Effort: Medium.
  - **C.** An entry holds a tag rule with "and", "or", and "not". Effort:
    Large.
- **Left out:** "Any of these tags" in one entry, because two entries next
  to each other give the same result. An entry that names one machine,
  because a tag on that machine gives the same result.
- **Wider fix:** None.
- **Previous examples:** GitHub runner labels in `runs-on`, which match all
  the named labels.
- **Recommendation:** A. It keeps each order row short. A later move to B
  needs only a migration.
- **Action:** ask the user, because B saves tags on large fleets, and A is
  simpler to read. Neither is clearly best.

## Milestone 1: Mockups

Tags: mockup

Depends on: none

Before this milestone, merge `calummoore/ciproxy-providers-interface-s8rp`
into this branch. The machine screens show tags, and the label order offers
tags.

- [ ] On the machine page, add a tag editor. Show a machine with no tags,
      with two tags, and a tag name error.
- [ ] On the Machines list, show the tags of each machine and a tag filter.
- [ ] In the label order, rename the own-machines entry to "All your
      machines". Let "Add a place" offer each tag. Show a tag row with its
      machine count and fit, and a tag that no machine has.
- [ ] Show a macOS label whose order holds tags.
- [ ] Update the job page and the Jobs notices so that they name the tag
      entry that ran a job and why.
- [ ] Show the warm runner copy of the machine page under the open choice of
      item 3.
- [ ] Update the site labels reference and the add-machine guide.
- [ ] Run the mockup check named in `AGENTS.md`. Smoke-test the changed
      pages.
- [ ] Update the design and the TODOs of this plan for each problem that the
      mockups show.

## Milestone 2: Documentation and protocol

Depends on: 1

- [ ] In `docs/protocol/labels.md`, add tags to the label order, with "All
      your machines" and the rules for a tag entry.
- [ ] In `docs/protocol/job-routing.md`, state the move between entries and
      where the overflow wait applies. State the warm runner rule of item 3.
- [ ] In `docs/protocol/agent.md`, state the tags of a machine.
- [ ] Update `crates/ciproxy-scaler/README.md` with entry placement.

## Milestone 3: Store and schema

Depends on: 2, and the merge of the provider branch into `main`

- [ ] Add `machine_tags` and the tag column of `label_places` in one
      migration. Keep every existing order unchanged.
- [ ] Add the store reads and writes of tags and of tag entries.
- [ ] Add store tests for the tag rules and the migration.

## Milestone 4: Scaler

Depends on: 3

- [ ] Give the snapshot the tags of each machine and the entries of each
      label.
- [ ] Place own-machine runners entry by entry. Move between own-machine
      entries at once, and apply the overflow wait before a provider.
- [ ] Place warm runners as item 3 decides.
- [ ] Add scaler tests: two tags in order, a machine with two tags, a tag
      with no machine, and a label that leaves own machines out.

## Milestone 5: Control plane and dashboard API

Depends on: 4

- [ ] Build the snapshot from the store tags and entries.
- [ ] Add the API to read and set the tags of a machine, and tag entries in
      the label order. Regenerate the API types.
- [ ] Give the job facts the entry that ran each job.

## Milestone 6: Dashboard

Tags: ui

Depends on: 1, 5

- [ ] Build the tag editor, the Machines list tags and filter, the tag
      entries of the label order, and the job reasons from the mockups.

## Milestone 7: Site

Depends on: 2

- [ ] Update the labels reference and the add-machine guide in `ts/site`.

## Milestone 8: Verification, commit, review, and pull request

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
