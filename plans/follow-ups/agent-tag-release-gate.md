# Agent tag runs publish without the gate

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from `conductor/ciproxy-boat-start-ready-deadline`,
  review round 1, finding 4: the blocked wider fix E of finding 1.
- **Location:** `.github/workflows/agent-release.yml:7` (the manual
  trigger), `.github/workflows/agent-release.yml:13-15` (the release gate
  runs only for `main`), `.github/workflows/agent-release.yml:198-199`
  (`publish` needs only the two package jobs), and
  `crates/ciproxy-agent/updates-and-releases.md` (its first section and its
  section "When a shared library moves" describe the gate and the tag run).
- **Reason:** The recommended option adds a check to every agent release,
  and the always-on rules reserve each new gate for the user.

- **Terms:**
  - **Release gate:** the job `Release repository gate` of
    `.github/workflows/agent-release.yml`. It runs the complete gate,
    `cargo xtask check --keep-going`, with PostgreSQL.
  - **Repository gate:** the CI job of `.github/workflows/ci.yml` that runs
    the same complete gate for each push and pull request.
  - **Tag run:** a run of `.github/workflows/agent-release.yml` for a
    `ciproxy-agent-v*` tag. It builds the release archives and publishes the
    release.
  - **release-plz:** the release tool that the `version` job runs. It opens
    version pull requests, and it creates the agent tag and a draft release.
  - **Hand tag:** a `ciproxy-agent-v*` tag that a person creates, not
    release-plz.
- **Background:**
  - The release workflow runs for pushes to `main`, for pushed
    `ciproxy-agent-v*` tags, and for manual runs through `workflow_dispatch`
    at line 7.
  - The release gate runs only for `main`, by the condition at line 15. The
    `version` job needs it, so release-plz runs only after the release gate
    passes.
  - `release-plz.toml` sets `release_always = false`. So release-plz creates
    the agent tag and a draft release only for a merge from a branch whose
    name starts with `release-plz-`.
  - After a merge with a merge commit, release-plz tags the last commit of
    the pull request, not the merge commit where the release gate ran.
  - For example, the tag `ciproxy-agent-v0.1.5` marks `8d8e017`, and the
    release gate ran on the merge commit `aeab3ae`. The tagged commit has
    passing repository gate checks from CI. Its only release gate check is
    skipped, in its own tag run.
  - A tag run runs only `package`, `package-macos`, and `publish`.
    `publish` needs only the two package jobs, at lines 198-199. It uploads
    the archives and publishes the draft release.
  - The ruleset `main-pull-request-and-gate` of `main` requires a passing
    repository gate before a pull request merges. No ruleset covers tags.
  - The section "When a shared library moves" of
    `crates/ciproxy-agent/updates-and-releases.md` sends that release
    through a pull request from a `release-plz-` branch. No step of the
    guide creates a hand tag.
- **Problem:** A tag run publishes the tagged commit with no release gate
  result. A manual run on an agent tag does the same. So a hand tag on any
  commit publishes a release that no release gate checked.
- **Impact if no action:** A hand tag, or a manual run on a tag, can publish
  an agent build that fails the gate. Machines on that release channel then
  install it at their next hourly update check.
- **Root question:** How does every published agent release come only from
  a commit that passed the release gate?
- **Options:**
  - **C.** Keep the guide route and change nothing more. The documented
    release then passes the release gate first. A hand tag, or a manual run
    on a tag, still publishes with no gate result. Effort: Small.
  - **D.** Set `release_always = true` in `release-plz.toml`. After the
    release gate passes on `main`, release-plz then tags and drafts each
    untagged agent version. So a version raise in the pull request that
    moves a pin releases with no hand step. It changes when releases happen,
    so the user must approve it. Release-plz warns of a race when a squash
    merge lands before its release run ends. It does not stop a hand tag or
    a manual run on a tag. Effort: Small.
  - **E.** Run the existing `gate` job also for `ciproxy-agent-v*` tag refs,
    and add `gate` to the `needs` of `publish`. Each tag run then publishes
    only a commit that passed the release gate in that run. This holds also
    after a hand tag or a manual run on a tag. Each release runs the
    complete gate once more, and publishing waits for it. Update
    `crates/ciproxy-agent/updates-and-releases.md` where it describes the
    gate and the tag run. No test reads the workflow YAML, so the next
    release proves the change. Effort: Small.
  - **F.** Add a GitHub tag ruleset for `ciproxy-agent-v*` that requires a
    passing gate check before anyone creates a matching tag. It needs admin
    access and adds a rejection. It lives outside the reviewed code, and a
    bypass actor can skip it. It does not cover a manual run on an existing
    tag. The commit that release-plz tags has no passing release gate check.
    So the rule must require the repository gate, which also passes on
    branches outside `main`. Effort: Small.
  - **G.** Add a read-only job to the tag run that waits for the release
    gate result and fails unless it passed. `publish` then needs this job.
    It needs no second gate run, but it needs API polling, a time limit, and
    handling of reruns. The commit that release-plz tags has no release gate
    run of its own. So the job must read the result of the merge commit
    that holds it. Effort: Medium.
- **Left out:**
  - Options A and B: a written step, or a shared release checklist, that
    tells a person to check the release gate result before a hand tag. C is
    better on every point, because it needs no hand check.
  - A tag rule that only limits who may create agent tags. It checks no gate
    result, so it fails the goal.
  - A hand release as a pre-release first. It still publishes an unchecked
    build to `main-preview`, so it fails the goal.
- **Wider fix:** E. It closes the class for every tag run, also after a hand
  tag or a manual run on a tag. It checks the exact commit that the tag
  marks.
- **Previous examples:**
  - The pre-releases 0.1.1 (16 November 2026, commit `889ca51`), 0.1.2
    (17 November, `4a950ae`), 0.1.3 (18 November, `6ed25c3`), and 0.1.4
    (20 November, `b751ef5`) came from hand tags. Their commits first
    reached `main` with the merge commit `3bdc477` of PR #97, on 21 November
    2026. So no release gate ran on them before their release.
  - On 22 November 2026, review round 1 of
    `conductor/ciproxy-boat-start-ready-deadline` found the same gap in the
    first version of the guide step, which created a hand tag. That branch
    moved the guide to the `release-plz-` route.
  - `plans/follow-ups/shared-release-gate-environment.md` records an
    earlier gap in the same release gate: it omitted the database setup that
    its checks needed.
- **Related:** `plans/follow-ups/shared-release-gate-environment.md`, the
  same code. Its option B moves the setup of the `gate` job into a shared
  action, and E runs that job also in tag runs. The two changes do not
  conflict.
- **Recommendation:** E. It closes the class for every tag run, also after
  a hand tag or a manual run on a tag, with a Small change.
- **Action:** ask the user, because E adds a check to every agent release,
  and a new gate needs the user's approval.
