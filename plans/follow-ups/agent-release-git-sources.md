# Git pin changes can miss agent releases

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 1102.
- **Source:** Review.
- **Category:** Process.
- **Location:** `Cargo.toml`, the `cli-updater`, `cli-release-interface`,
  and `internal-error` pins of `[workspace.dependencies]`,
  `release-plz.toml`, `.github/workflows/agent-release.yml`, the
  release-plz step that pins version 0.4.3, `crates/ciproxy-agent/Cargo.toml`,
  and `crates/ciproxy-agent/updates-and-releases.md`.
- **Reason:** The best option, E, adds a gate phase, and only the user can
  add a gate.

- **Terms:**
  - **release-plz:** the tool that opens the version pull request and tags
    the agent release. The release workflow pins its version 0.4.3.
  - **Git pin:** a dependency that a manifest takes from a Git repository
    at one commit, with `rev`.
  - **Version pull request:** the pull request that release-plz opens from
    a `release-plz-` branch to raise the shared workspace version.
  - **Updater:** `cli-updater`, the crate that updates the installed
    agents. It comes from `futex-ai/cli-release`.
  - **Agent tag:** the tag `ciproxy-agent-vVERSION` of each agent release.
- **Background:**
  - The agent, the cache sidecar, and the protocol crate share one
    workspace version and one agent tag. All three compare their changes
    with the last agent tag, as `crates/ciproxy-agent/updates-and-releases.md`
    says.
  - The root `Cargo.toml` pins three crates of the organization by Git
    commit in `[workspace.dependencies]`. `cli-updater` and
    `cli-release-interface` come from `futex-ai/cli-release`.
    `internal-error` comes from `futex-ai/internal-error`.
  - The agent manifest inherits all three with `workspace = true`. The
    `rust` skill puts the Git crates of the organization in the root
    `[workspace.dependencies]` table.
  - release-plz 0.4.3 raises a version only for a commit that changes a
    file of the package, or for a changed locked version of a dependency.
    It reads only the package directory and its README.
  - It ignores Git sources on purpose, because a registry package loses
    them when it is published. It compares locked dependencies by name and
    version only.
  - The upstream crates keep one version across commits. Every cached
    commit of both repositories says 0.1.0. The lock file holds
    `internal-error` 0.1.0 at two different commits.
  - So a change of a root Git pin that keeps the versions changes no file
    of the agent package and no locked version.
  - The cause is the root pins together with release-plz 0.4.3. The
    setting `git_only = true` is not the cause, because the registry mode of
    release-plz uses the same two comparisons.
  - `release-plz.toml` sets `release_always = false` and
    `changelog_update = false`. So the release step skips a merge that does
    not come from a `release-plz-` branch. With no changelog to change,
    release-plz also opens no version pull request for a version change
    alone.
  - On 8 November 2026 the user chose to merge the release server changes
    of `futex-ai/cli-release` and to publish no release of it. ciproxy pins
    the merged commit.
  - The releases v0.1.1 and v0.1.2 got their tag and release by hand, after
    the version commits `889ca518` and `4a950ae2`.
  - Each agent release restarts the agent and the cache sidecar on every
    machine, in `crates/ciproxy-agent/src/cache_service.rs`.
  - The `testing` skill makes the behaviour of a third-party tool a
    one-time proof, not a gate test.
- **Problem:** A change of a root Git pin that keeps the crate versions
  raises no agent version. So release-plz opens no version pull request,
  and no release ships the changed updater.
- **Impact if no action:** An updater fix can stay away from the installed
  machines until another change of a shipped package raises the version.
- **Root question:** How should changes to shipped Git dependencies reliably
  trigger a release?
- **Options:** A second review pass on 17 November 2026 corrected options A
  to D. Each correction follows its option.
  - **A.** Raise the workspace version by hand with each relevant Git pin
    change. Cost: manual release work. Effort: Small.
    - Correction: a version increase alone does not release. With
      `release_always = false`, the release step skips a merge that does
      not come from a `release-plz-` branch.
    - With `changelog_update = false`, the version pull request has no file
      to change, so release-plz opens none.
    - The raised version then stays without a tag. release-plz keeps that
      version for later changes too, so it opens no version pull request
      for them either.
    - So A must also push the tag and create the release by hand, as for
      v0.1.1 and v0.1.2. Or A must open the increase from a `release-plz-`
      branch.
    - The second pass read this in the release-plz source. Nobody ran it.
  - **B.** Move the Git pins into the agent manifest. Effort: Small.
    - Correction: the agent also inherits `internal-error`. So B must move
      all three pins to cover every current Git source. `cli-updater` has
      no Git dependency of its own. The sidecar and the protocol crate have
      none.
    - B departs from the layout of the `rust` skill. The next shipped Git
      crate that follows that layout opens the gap again.
  - **C.** One release preparation helper that compares the complete
    dependency identities, with their Git commits, against the last agent
    tag. Add a regression case. Effort: Large.
    - Correction: the option does not say how a detected change becomes a
      version pull request. release-plz raises a version only for a package
      commit or a changed locked version.
    - So the helper must open the pull request itself from a `release-plz-`
      branch. Or it must commit a change into the agent package before
      release-plz runs.
    - Its regression case tests the helper, not release-plz.
    - With this work, C is Large, not Medium. Round 1 of the same review
      graded the same approach as Large.
    - When the helper sits inside `crates/ciproxy-agent/`, each edit of the
      helper also counts as an agent change, because release-plz compares
      the files of the package.
  - **D.** Fix the Git source comparison in release-plz upstream, and pin
    its fixed release. Cost: an upstream change and release.
    Effort: Large.
    - Correction: release-plz ignores Git sources on purpose, because a
      registry package loses them when it is published. So the upstream
      change must compare Git sources only between source trees, in the
      `git_only` mode.
  - **E.** Record the shipped Git sources inside the agent package, and
    check the record in the gate.
    - Add an `xtask` command that reads `cargo metadata --locked`. It walks
      the normal and build dependencies of `ciproxy-agent` and
      `ciproxy-cache-sidecar`.
    - It writes each Git source that these crates reach, with its resolved
      commit, to one generated file in `crates/ciproxy-agent/`. Today these
      are `cli-updater`, `cli-release-interface`, `internal-error`, and
      `internal-error-derive`.
    - The file holds no workspace crate version. The version pull request
      changes those versions, so they would make the drift check fail on
      every release.
    - A new gate phase regenerates the file and fails on a difference, as
      the `api-types` phase does.
    - A pin change then changes a file of the agent package. release-plz
      compares those files and opens the version pull request on its normal
      path. No custom release logic is needed.
    - The same walk can also fail when a workspace crate that the two
      binaries use is not release-enabled with the agent tag.
    - Test the generator on a metadata fixture. Prove once, with
      release-plz 0.4.3 in a scratch clone, that a pin change with the
      regenerated file raises the version. Keep that proof out of the
      repository.
    - Cost: a new gate phase, which needs the user's approval. Each pin
      change must also regenerate the file, and the gate says so.
    - Effort: Medium.
  - **F.** Make each pinned upstream change carry a new crate version.
    - Pin `cli-release` only at its release commits, for example with
      `tag = "vX.Y.Z"` in place of `rev`. A `cli-release` release raises its
      shared workspace version, so `cli-updater` and
      `cli-release-interface` get a new version.
    - Raise the version of `internal-error` with each change that the agent
      pins.
    - release-plz then sees a changed locked version and opens the version
      pull request.
    - F drops the assumption that the upstream crates keep one version
      across commits. The source of that assumption is the code of the two
      upstream repositories. It is their choice, not a requirement.
    - Cost: the user must merge release work in two other repositories. F
      changes the practice of the decision of 8 November 2026, which pins
      merged commits and publishes no release. A pin of a commit with no new
      version opens the gap again, and nothing warns.
    - Effort: Medium.
- **Left out:**
  - `dependencies_update = true` and `release_always = true`: they fail
    the goal. `dependencies_update` refreshes the dependencies but does not
    fix the comparison. `release_always` decides only whether a merge that
    is not from a release pull request gets a tag.
  - B with a gate that keeps shipped Git crates in shipped manifests: E is
    better on every point. It needs the same approval, keeps the layout of
    the `rust` skill, and also covers Git sources that the binaries reach
    only through other crates.
  - A hash of all Rust inputs, as `deploy/scripts/image_context.py` does
    for the hosted image: E is better on every point. The hash also changes
    for pins that only the hosted service uses, such as the Boat pin
    changes `55a629dd` and `2412c025`. Each extra release restarts the
    agent and the cache sidecar on every machine.
  - Publishing the crates of the organization to a registry: E is better
    on every point for this goal. E gives the same reliability with no
    registry and no publishing in two other repositories. It also keeps the
    rule of the `rust` skill on Git crates.
  - A release-plz setting: not a real choice. No 0.4.3 setting adds a path
    outside the package to the change walk, or compares Git sources.
  - The hourly update check on the machines: not a real choice. It acts
    only after a release exists.
- **Wider fix:** E, changed from C. E reads the resolved dependency graph.
  So it also catches a change from a new Git crate, an indirect Git source,
  or a `[patch]` override. C is the wider fix with no new gate.
- **Previous examples:** Round 1 of the same review found that a change of
  the cache sidecar alone raised no agent version. release-plz processed
  only the agent package. Commit `bdaa149` of 17 November 2026 fixed it.
  It is the same class: release-plz sees only a part of what the two
  shipped binaries compile.
- **Recommendation:** E, changed from C. E closes the class with the change
  check that release-plz already has. It runs on each pull request, needs
  no custom release logic, and can also check the release setup of the
  crates that the binaries use. Without a new gate, C is the best option.
  B, extended to `internal-error`, is the Small direct fix for the current
  Git sources.
- **Action:** ask the user, because E adds a new gate phase, and a new gate
  needs the user's approval. Without the gate, apply C.
