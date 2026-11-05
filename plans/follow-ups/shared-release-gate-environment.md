# Release and CI gate setup diverge

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from
  `conductor/ciproxy-mvp-milestone-9-agent`, review item 8's wider fix.
- **Source:** Review.
- **Category:** Process.
- **Location:** `.github/workflows/ci.yml`,
  `.github/workflows/agent-release.yml`, and `.github/actions/`.
- **Reason:** Sharing setup changes ordinary CI as well as release
  preparation, so it belongs in a separate change.

- **Terms:**
  - **Repository gate:** `cargo xtask check` runs the repository checks,
    including the Rust store tests.
  - **PostgreSQL:** The database required by the store integration tests.
  - **Composite action:** Shared GitHub Actions steps that run inside the
    calling job and preserve its check name.
  - **release-plz:** The tool that creates version pull requests and
    release tags for the agent.
- **Background:** Ordinary CI and agent release preparation both run the
  complete gate. Store tests need PostgreSQL 15, `DATABASE_URL`, and libpq.
  Release preparation must pass that gate before release-plz runs.
- **Problem:** Each workflow keeps a separate copy of service and tool
  setup. The release workflow originally omitted PostgreSQL after the
  store tests merged. Its direct fix added a read-only database-backed
  gate job, but both setup copies remain.
- **Impact if no action:** A future gate dependency can block version
  pull requests and releases while ordinary CI remains green.
- **Evidence:** Store setup in
  `crates/ciproxy-store-pg/tests/store/support/mod.rs` requires
  `DATABASE_URL`. Its failure text is
  `DATABASE_URL is required for real PostgreSQL store tests`. Both current
  workflow gate jobs now declare the same PostgreSQL service, environment,
  Rust setup, Node setup, npm installation, and complete-gate command.
- **Requirement:** Keep the complete gate in `AGENTS.md` and
  `xtask/README.md`. Preserve existing check names, phase selection,
  permissions, and concurrency policy. Dependency installation and gate
  execution stay outside release jobs with repository write access.
- **Root question:** How can complete-gate callers share their required
  environment without changing gate policy or check names?
- **Options:**
  - **A.** Keep matching setup declarations in both workflows. Future
    changes still need two edits. Effort: Small.
  - **B.** Extract setup and gate execution into a composite action under
    `.github/actions/`. Both existing gate jobs call it. The action starts
    and removes PostgreSQL with Docker because composite actions cannot
    declare service containers. Effort: Medium.
  - **C.** Require successful ordinary CI for the exact release commit.
    This adds cross-workflow coordination and must handle cancelled runs.
    Effort: Medium.
  - **D.** Move release-plz into CI after its gate job. CI currently cancels
    older runs on the same branch. With `release_always = false`, cancelling
    a version-merge run can lose its release. Changing that cancellation
    policy needs a separate user decision. Effort: Medium.
  - **E.** Let the gate start PostgreSQL when `DATABASE_URL` is absent.
    This adds Docker to local gate requirements and changes documented
    setup. It needs a separate user decision and shares no Node or Rust
    setup. Effort: Medium.
  - **F.** Run only agent and protocol checks before release-plz. This
    weakens the documented release gate and needs a separate user
    decision. Its second check list can also drift. Effort: Small.
- **Left out:** Skipping store tests permits a pass without their checks.
  A test of workflow YAML literals does not execute the gate. A reusable
  workflow changes caller check names and can break branch protection.
  Running no release check fails to check the exact release commit.
- **Wider fix:** B. One composite action supplies the gate environment to
  both callers without changing their check names.
- **Previous examples:** Agent release preparation omitted PostgreSQL
  when store integration tests joined the workspace. The direct fix
  copied the existing CI setup into a separate read-only gate job.
- **Recommendation:** B. Shared setup prevents caller drift while keeping
  the existing release and CI policy.
- **Action:** fix after merge, apply B. Reuse the existing checks and add
  no gate, stricter validation, permission, or cancellation-policy change.
- **Done when:** Both workflows use one composite action for their gate
  setup and execution. The action cleans up its PostgreSQL container after
  success or failure. Ordinary CI still reports `Repository gate`, and
  release preparation still reports `Release repository gate`.
  Release-plz still depends on the successful read-only gate job. Actual
  complete-gate runs pass in both callers. No test reads workflow YAML to
  assert step names or order.
