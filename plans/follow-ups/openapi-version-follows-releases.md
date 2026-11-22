# Every version raise needs a hand API regeneration

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from `conductor/ciproxy-boat-start-ready-deadline`,
  review round 2, finding 9: the blocked wider fix C of finding 5.
- **Location:**
  - `crates/ciproxy-api/src/openapi.rs:19` writes the workspace version into
    `info.version` of the API reference.
  - `crates/ciproxy-api/Cargo.toml:3` takes the workspace version.
  - `ts/api/generated/openapi.json:6` holds the saved `info.version`.
  - `crates/ciproxy-agent/updates-and-releases.md:17-31` describes the
    normal release-plz route. Its section "When a shared library moves"
    describes the hand route and the takeover by release-plz.
  - `ts/api/README.md` for option C, and `release-plz.toml:8` for option D.
- **Reason:** The recommended option changes what live code on `origin/main`
  writes into `ts/api/generated/openapi.json`, so it needs the user's
  approval.

- **Terms:**
  - **release-plz:** the release tool in the `version` job of
    `.github/workflows/agent-release.yml` that writes version pull requests
    and creates agent tags.
  - **Version pull request:** a pull request from a branch whose name starts
    with `release-plz-` that raises the shared workspace version.
  - **API reference:** `ts/api/generated/openapi.json`, the OpenAPI document
    that `npm --prefix ts run api:generate` writes from the Rust routes.
  - **`api-types` phase:** the gate phase that generates the API reference
    again and fails when the saved files differ.
  - **Repository gate:** the required CI check of `.github/workflows/ci.yml`
    that runs the complete gate, with the `api-types` phase, for each pull
    request.
- **Background:**
  - The root `Cargo.toml` sets the shared version under
    `[workspace.package]`. The agent, the cache sidecar, the protocol crate,
    and `ciproxy-api` take it with `version.workspace = true`.
  - `crates/ciproxy-api/src/openapi.rs:19` writes
    `env!("CARGO_PKG_VERSION")` into `info.version`. So each version raise
    changes the API reference.
  - The `api-types` phase then fails until someone runs
    `npm --prefix ts run api:generate` and commits the result.
  - The ruleset that `deploy/scripts/github_ruleset.py` writes requires the
    repository gate before a merge to `main`. So a version pull request
    without the regeneration cannot merge.
  - release-plz writes its version pull requests from `main` and never runs
    the generator.
  - No code reads `info.version`. Only the developer binary
    `ciproxy-openapi` prints the document. `ts/api/generate.ts` saves that
    output and compares it with the saved file.
  - The hand raises from 0.1.1 to 0.1.4 changed the API reference in the
    same commit as the version.
  - On 22 November 2026, the release-plz commit `6debc52` raised the version
    to 0.1.5 and left the API version at 0.1.4. Commit `8d8e017` then added
    the regeneration by hand to that pull request, PR #110.
  - release-plz 0.4.3 looks for commits from a person in an open version
    pull request. It skips the first commit, and it ignores logins that end
    in `[bot]`.
  - When no such commit is left, release-plz rewrites the pull request in
    place. Otherwise it closes the pull request and opens a new one.
  - A hand regeneration commit is a commit from a person after the first
    commit. So the next release-plz update closes that pull request and
    opens a new one without the regeneration.
  - On the same day, review round 2 of
    `conductor/ciproxy-boat-start-ready-deadline` found the same gap in the
    hand release route of `crates/ciproxy-agent/updates-and-releases.md`.
    release-plz can take over the hand pull request, and the pull request
    that remains lacks the regeneration.
  - That branch added the regeneration steps to that takeover case of the
    guide. The normal release-plz route at
    `crates/ciproxy-agent/updates-and-releases.md:17-31` still does not say
    that each version pull request needs the regeneration.
- **Problem:** Each agent version raise needs a hand run of the API
  generator, or the required repository gate fails. release-plz never runs
  the generator. The release guide asks for it only on the hand route.
- **Impact if no action:** Each release-plz version pull request fails the
  repository gate until a person regenerates the API reference on its
  branch. A later release-plz update can drop that regeneration, so the
  release waits for the same hand step again.
- **Root question:** How does every agent version raise, by a person or by
  release-plz, pass the `api-types` check?
- **Options:**
  - **A.** Document the regeneration for every version pull request in
    `crates/ciproxy-agent/updates-and-releases.md`. Add it to the normal
    release-plz route, as the hand route already has it. It keeps a hand
    step on each release. A later release-plz update can close a pull
    request with the regeneration commit, so the step can repeat. The
    required repository gate still stops a missed regeneration. Effort:
    Small.
  - **B.** Make the release workflow regenerate the API reference and commit
    it after release-plz opens or updates a version pull request. The
    `version` job already has write access, so the job needs no new write
    access. A push with `GITHUB_TOKEN` starts no CI run, so the required
    repository gate never reports. A push with `RELEASE_PLZ_TOKEN` starts
    CI, but the job then runs `npm ci` and a Cargo build next to a token
    that can publish releases. Effort: Medium.
  - **C.** Write a fixed contract version, such as `1`, into `info.version`
    at `crates/ciproxy-api/src/openapi.rs:19`, in place of
    `env!("CARGO_PKG_VERSION")`.
    - Regenerate the API reference once.
    - In `ts/api/README.md`, say that the contract version does not follow
      releases.
    - In the guide, remove the regeneration steps. Tell operators to merge
      the release-plz pull request that remains after a takeover.
    - A version raise then changes only `Cargo.toml` and `Cargo.lock`. So
      every version pull request passes the repository gate with no hand
      commit, on every route.
    - It also removes the hand commit that the normal route needs now, as
      `8d8e017` was for PR #110.
    - Cost: the API reference no longer shows which workspace version made
      it. No code reads that field.
    - It changes the output of live code on `origin/main`.
    - Effort: Small.
  - **D.** Set `release_always = true` in `release-plz.toml`.
    - Tell operators to raise the version by hand from a branch whose name
      does not start with `release-plz-`. Remove the takeover case from the
      guide.
    - release-plz then never takes over the hand pull request, so its hand
      regeneration stays. The gated `version` job tags the untagged version
      after the merge.
    - Cost: the version pull requests of release-plz still need a hand
      regeneration.
    - Each merge that raises the version then releases at once. The
      release-plz 0.4.3 docs also warn of a race when a squash merge lands
      before `release-plz release` ends.
    - It changes when releases happen.
    - Effort: Small.
- **Left out:**
  - Skipping or loosening the `api-types` phase. It breaks the `AGENTS.md`
    rule that the phase fails on a difference.
  - A drift check that ignores the version, at `ts/api/generate.ts:56`. It
    keeps a false version in the saved contract, and C gives the same pass.
  - A release-plz hook that runs the generator. release-plz 0.4.3 has no
    setting that runs a command before it commits.
  - A separate workflow on `release-plz-` pull requests. It is B with
    another trigger, and it has the same costs.
  - A hand tag. The raise must still merge through a pull request that the
    repository gate checks, so a hand tag fixes nothing. It would also undo
    the guide route that sends each release through the gated `version`
    job.
  - A fixed `version` for `ciproxy-api` in its own `Cargo.toml`. C is better
    on every point. C has the same effect, keeps every crate on the
    workspace version, and puts the fixed value next to the field that it
    sets.
  - Stopping release-plz version pull requests with `command: release`. D
    is better, because D also ends the takeover and keeps the automatic
    version pull requests. This approach makes every agent release a hand
    release.
  - Another release tool. C is better on every point, because it fixes the
    cause with the current tool, at Small effort.
- **Wider fix:** C. It removes the hand step on every route, also if
  release-plz changes how it takes over a pull request.
- **Previous examples:**
  - `6debc52` raised the version to 0.1.5 without the regeneration, and
    `8d8e017` in PR #110 added it by hand.
  - The hand raises `889ca51`, `4a950ae`, `6ed25c3`, and `b751ef5`, from
    0.1.1 to 0.1.4, each changed the API reference too.
  - On 22 November 2026, review round 2 of
    `conductor/ciproxy-boat-start-ready-deadline` found the gap in the
    takeover case of the hand release route.
- **Related:** `plans/follow-ups/agent-tag-release-gate.md`, the same code.
  Its option D sets the same `release_always = true` in `release-plz.toml`.
  An answer that applies D in one item applies it in both. The two
  recommendations, C here and E there, do not conflict.
- **Recommendation:** C. It removes the hand regeneration on every route
  with a Small change, and no code reads the field that it changes.
- **Action:** ask the user, because C changes the output of live code on
  `origin/main`, and such a change needs the user's approval.
