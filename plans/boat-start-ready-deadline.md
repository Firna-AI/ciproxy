# Boat start ready deadline

Status: Active. On branch `conductor/ciproxy-boat-start-ready-deadline`, no
pull request yet. The pull request that completes the work deletes this
plan, as the `plans` skill says.

A Boat start can fail after its launch and send no exit report. Then the
control plane counts its runner as ready, a waiting job claims it, and
ciproxy starts no other runner for that job. The job can wait until the
sandbox lifetime of 6 hours ends. This change gives each launched Boat
start a ready deadline of 10 minutes from its launch. When the runner has
taken no job by the deadline while a job waits for it, ciproxy retires the
runner and deletes its sandbox, and the job gets another runner. A runner
that took a job keeps running. A start with no waiting job keeps the idle
rules. The change is in the control plane only. It needs no image or guest
change. It does not add a ready report or a failure report from the
sandbox.

The change also updates three follow-up records, as the decisions below
say.

Contract owners:

- [Sandbox end](../docs/protocol/overflow-sandbox-end.md).
- The [scaler README](../crates/ciproxy-scaler/README.md).
- The [overflow README](../crates/ciproxy-overflow/README.md).
- The [control-plane README](../crates/ciproxy-control-plane/README.md).

## Decisions

- **2026-11-22, the ready deadline:** option B of
  `plans/follow-ups/boat-launched-runner-readiness.md`, with a deadline of
  10 minutes from the launch. This is an agent call. The orchestrator of
  this work made it under the user's delegation of 22 November 2026. On
  that day the user answered the open items with "these are all annoying
  questions - I don't care about it". The user did not choose an option.
  - Rejected: A, a failure report from the sandbox for failures that the
    guest cannot report. It misses a hang and a lost report. It also
    misses each failure after the guest removes its registration file,
    because the callback address and token are then gone.
  - Rejected: C, a ready report from the guest, with a deadline. It needs
    a new guest report, a new callback, and a new image. B gives the same
    result sooner, with a control-plane change only.
  - Reason: B covers each cause, also a hang. A lasting cause can repeat
    paid starts until the spend limit stops them, as with each option.
- **2026-11-22, `image-checks-in-gate.md`:** no new gate check now,
  because a new gate needs the user's explicit approval. The item stays
  open.
- **2026-11-22, `boat-client-diagnostic-stderr.md`:** the change in
  `futex-ai/sandbox` waits until after the launch. PR #96 fixed the warning
  that caused the failure at its source. The item stays open.
- **2026-11-22, `agent-release-git-sources.md`:** apply option A. The
  agent release guide gets the step "raise the agent version by hand when
  a shared library moves". The item is then deleted.

The orchestrator recorded the three record decisions under the same
delegation.

## Design

The calls below are the orchestrator's design calls, made on 2026-11-22.

- The start record saves its launch time as a fact. The overflow service
  sets it with its injected clock, when it saves the phase `Running`. Both
  paths that save a launch use one method that sets the phase and the time
  together, so that a later launch path cannot forget the time. The record
  is JSON in the store, so the new field needs no migration. Reason: the
  store keeps facts, and the policy stays in one place.
- The pure scaler holds the 10-minute rule. Each Boat runner in the scaler
  snapshot carries its launch time on the snapshot clock. At
  `now >= launch time + 10 minutes`, an idle launched Boat runner retires
  with a new stop reason, and no waiting job claims it. Reason: the scaler
  already decides claims and retirement, and `BoatRunnerAuthority::retire`
  reruns the same decision before it removes a runner. So one rule reaches
  both the runtime pass and the overflow cleanup.
- The rule applies to each idle launched Boat runner, also with no waiting
  job. Reason: without a waiting job the idle rule deletes the same
  sandbox, so the outcome is the same, and one condition is simpler.
- The replacement starts in the same scaler decision as the retirement,
  through the existing placement rules. Reason: a retiring runner is not
  supply, as for each other stop reason. When GitHub gave the runner the
  job first, the GitHub-first removal keeps it, and the idle rule then
  removes the extra runner.
- A `Running` record without a launch time gets no ready deadline. Reason:
  only records saved before this change lack the time, and ciproxy is not
  in production.
- The step for option A of `agent-release-git-sources.md` goes into
  `crates/ciproxy-agent/updates-and-releases.md`. Reason: the agent README
  sends release readers to that guide.
- This change also deletes `plans/follow-ups/boat-start-report-credential-scope.md`.
  Reason: its only subject is the text of option A in the readiness item,
  which this change deletes with option A rejected.
- Tests use the injected clocks of the scaler snapshot, the overflow
  service, and the control-plane runtime. No test sleeps, retries, or
  starts a live Boat sandbox. The Boat smokes stay on hold.

## Milestone 1: Protocol rule and follow-up records

Depends on: none

The sandbox end doc states the ready deadline and its decision. The three
follow-up records get their updates. The `docs implementor` does this work.

- [ ] Add the ready deadline rule and its decision line to
      `docs/protocol/overflow-sandbox-end.md`.
- [ ] Update `plans/follow-ups/image-checks-in-gate.md`. Its **Action**
      line says that no new gate check comes now, and the item stays open.
- [ ] Update `plans/follow-ups/boat-client-diagnostic-stderr.md`. The
      change waits until after the launch, PR #96 fixed the warning at its
      source, and the item stays open.
- [ ] Write the step of option A of
      `plans/follow-ups/agent-release-git-sources.md` in
      `crates/ciproxy-agent/updates-and-releases.md`. Delete the item.
- [ ] Delete `plans/follow-ups/boat-start-report-credential-scope.md`.
- [ ] Run the Markdown check.

## Milestone 2: Launch time and ready deadline

Depends on: none

The start record saves its launch time, and the scaler retires a launched
Boat runner that took no job by its deadline. The `non-ui implementor`
does this work.

- [ ] Add the launch time to the start record, with the method that saves
      a launch.
- [ ] Save the launch time with the overflow clock in both launch paths.
      Extend the existing overflow service tests of both paths.
- [ ] Add the launch time to the scaler's Boat runner, the 10-minute
      deadline, and the new stop reason. Add scaler unit tests at the
      boundary.
- [ ] Pass the launch time from the start record to the scaler snapshot in
      the control plane.
- [ ] Add a control-plane runtime test: a launched start with a waiting
      job keeps its runner at 9 minutes 59 seconds, and at 10 minutes its
      sandbox is deleted and another start follows.
- [ ] Run the targeted tests of each changed crate and the early check.

## Milestone 3: README alignment

Depends on: 2

The READMEs of the changed crates describe the launch time and the ready
deadline. The `docs implementor` does this work.

- [ ] Update the READMEs and guides of the changed crates.
- [ ] Run the Markdown check.

## Milestone 4: Verification, commit, review, and pull request

Depends on: 1, 2, 3

- [ ] Delete `plans/follow-ups/boat-launched-runner-readiness.md`.
- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Open a draft pull request into `main`. Its description starts with
      "Depends on #116".
- [ ] Review the complete local diff with the `implementation-review`
      skill after the push. While #116 is open, the base is
      `origin/fix/boat-exit-report-claim-first`. Report the findings, then
      apply the fix rule: fix the `fix now` findings, record follow-up
      items under `plans/follow-ups/` for the rest, re-review once, and
      report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Update the pull request. Its description gives the decisions and the
      design calls of this plan with their dates, a permalink to this plan,
      and the follow-up items that the change adds and deletes.
- [ ] When #116 has merged, merge `origin/main` with the `git-merge`
      skill, run the complete gate, push, and mark the pull request ready.
