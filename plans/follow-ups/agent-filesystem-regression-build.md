# The gate never compiles the filesystem regressions

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Outside the scope.
- **Added:** 2026-11-06, from `conductor/ciproxy-mvp-milestone-13-e2e`, the
  check of item 7 in the first owner review of pull request #31. That check
  found it outside the scope.
- **Source:** Review.
- **Category:** Test.
- **Location:** `xtask/src/check/phases.rs:60-69` and `:81`,
  `crates/ciproxy-agent/Cargo.toml:41-53`,
  `crates/ciproxy-agent/tests/root_disk_regression.rs:8-13`,
  `crates/ciproxy-agent/tests/storage_regression.rs:9-20`, and
  `crates/ciproxy-agent/README.md:228-241`.
- **Reason:** The gap predates these fixes, and a later change can update
  the shared developer instructions without changing the gate.

- **Terms:**
  - **Gate:** `cargo xtask check`, the repository checks that also run in
    GitHub Actions, with its phases in `xtask/src/check/phases.rs`.
  - **Opt-in targets:** The agent test targets `storage-regression` and
    `root-disk-regression`, which build only with the
    `filesystem-regression` feature of `ciproxy-agent`.
  - **Path include:** A `#[path = "../src/<file>.rs"] mod <name>;` line,
    which compiles a copy of an agent source file inside a test target.
- **Background:**
  - The opt-in targets test real growth and bounded writes on disposable
    ext4 and XFS files. They need root, loop mounts, and XFS tools.
  - `crates/ciproxy-agent/image-budget.md` keeps them out of the gate's test
    run on purpose. Its two "Disposable" regression sections give their run
    commands.
  - `crates/ciproxy-agent/Cargo.toml:41-53` gives each target
    `required-features = ["filesystem-regression"]`. Cargo skips such a
    target without an error or a warning when the feature is off.
  - The `rust-clippy` phase runs
    `cargo clippy --workspace --all-targets -- -D warnings`. The
    `rust-test` phase runs `cargo test --workspace`. Neither turns on a
    feature, and the agent README's development commands do not either.
  - The agent library exposes only `root_sizing`. The other agent modules
    are private to the binary, so the targets use path includes.
  - `root_disk_regression.rs` includes `error.rs`, `process.rs`, and
    `vm_root.rs`. `storage_regression.rs` includes `budget_file.rs`,
    `error.rs`, `host_system.rs`, `image_space.rs`, `sparse_copy.rs`, and
    `storage_sizing.rs`.
  - Only manual runs compile the targets.
    `crates/ciproxy-agent/scripts/storage-regression.py` exits 77 with
    `SKIP` before it compiles anything when root or a tool is missing.
  - `filesystem-regression` is the only feature in the workspace.
  - This item did not compile the targets, so their current build state is
    unknown.
- **Problem:** No gate phase compiles the two opt-in targets. A change to an
  included file can break their build while every check passes. One example
  is code that moves from an included file into a new module. The included
  file then imports a module that the target lacks.
- **Impact if no action:** The break shows only at the next manual run on a
  root host, which can come long after the change. That run must first find
  and repair the break. Each break that lands while this item waits adds to
  that repair.
- **Root question:** How does the repository find a change that breaks the
  build of the opt-in filesystem regressions?
- **Options:**
  - **A.** Document the gap under `## Development` in
    `crates/ciproxy-agent/README.md`. Add `--features filesystem-regression`
    to its clippy command, which needs no root. Run that command once, and
    repair each break that it finds. List the included files, and tell
    authors to run the command after they change one. A break still passes
    the gate when an author skips the command. Effort: Small.
  - **B.** Add `--all-features` to the clippy command of the `rust-clippy`
    phase. The gate then compiles and lints both targets without running
    them. Update the command in `xtask/src/check/phases.rs`, its doc
    comment, and `xtask/README.md`. The first run can report errors and
    warnings in code that no check has built. It adds a new build error to
    the gate. Effort: Small.
  - **C.** Build the targets by default. Remove the feature, and mark each
    regression `#[ignore]` with its reason. Their run commands then add
    `-- --ignored`. Move `storage-regression` from its own `main` to the
    standard test harness, so that `cargo test` lists it as ignored. Every
    Cargo command then compiles the targets, so the gate gets a new build
    error too. Effort: Medium.
- **Left out:**
  - Change nothing. It fails the goal, because a break stays hidden until
    the next manual run. Drop this item to keep that state.
  - Move the included modules into the agent library. It fails the goal,
    because a changed signature still breaks the targets without a failed
    check. It also puts host-only code into a library that three hosted
    crates link.
  - A compile step in `.github/workflows/ci.yml` only. B is better on every
    point, because the gate runs the same checks locally and in CI.
  - Run the regressions in CI. It needs root, loop mounts, and XFS tools on
    the runner, and B finds the same build breaks without them.
- **Wider fix:** B. It catches a build break of every feature-gated target
  in the gate, also of later ones. C fixes only these two targets.
- **Previous examples:** None found. Neither target changed after commits
  `b5d5fa2` and `d6d18e6` added them on 2026-11-05.
- **Recommendation:** A. It needs no gate change and reaches each author who
  reads the agent README before a change. B best protects the codebase, but
  it makes the gate stricter, so it needs the user's approval.
- **Action:** fix after merge, apply A, because the shared developer
  instructions are outside these fixes. B and C remain options that need
  the user's approval before a later change can add them.
