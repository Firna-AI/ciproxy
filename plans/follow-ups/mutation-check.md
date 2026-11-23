# No check finds a test that skips part of its rule

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch
  `calummoore/default-labels-rename`, the wider fix of review finding 2.
- **Location:** `xtask/src/check/phases.rs` and `xtask/README.md`.
- **Reason:** Each option that detects the class adds a gate phase or a
  review step with new dev dependencies, and the user decides on those.

- **Terms:**
  - **Gate:** `cargo xtask check`, the repository check that CI runs on
    each push. `xtask/README.md` lists its phases.
  - **Mutant:** a copy of the code with one small change, such as a
    removed comparison. A mutant survives when every test still passes.
  - **Mutation check:** a tool that makes and tests mutants, such as
    StrykerJS for TypeScript and cargo-mutants for Rust.
  - **Weak test:** a test whose inputs let it pass without a part of the
    rule that it claims to check.
- **Background:**
  - The shared `testing` skill asks each author to prove that a test can
    fail. Authors usually prove it by breaking the whole behaviour once.
  - A rule with several parts can still have a part that no test checks.
    Three cases in this repository show the class:
    - The label list order test in `ts/app/_tests_/settings-labels.test.ts`
      first passed with the vCPU key or the disk key of `compareLabels`
      removed, and with memory before vCPU. Its inputs never let those
      keys decide. The branch `calummoore/default-labels-rename` added
      two inputs, and six mutants of the order now fail the test.
    - `plans/follow-ups/image-build-oidc-case.md`: a subtest passes
      whenever an earlier assertion passes, so it checks no provider
      condition.
    - `plans/follow-ups/boat-fallback-journal-assertion.md`: a check of the
      Boat runner entry passes with a changed fallback command.
  - In each case, a reviewer traced or ran a mutant by hand.
- **Problem:** No tool finds a surviving mutant. A weak test reaches `main`
  unless a reviewer tries the right mutant.
- **Impact if no action:** Weak tests keep reaching `main`. A later change
  can then break a rule that its test seems to cover.
- **Root question:** How does the repository catch a test that passes
  without a part of the rule that it checks?
- **Options:**
  - **A.** Change nothing. Reviewers keep tracing mutants by hand. Effort:
    Small.
  - **B.** Add one rule to "Prove that each test can fail" in the shared
    `testing` skill of `Firna-AI/skills`: when a rule has several parts,
    such as the keys of an order, break each part on its own. The change
    is outside this repository, and it changes the rules of each
    repository that uses the skill. It detects nothing. Effort: Small.
  - **C.** Add a gate phase that runs a mutation check on the lines that a
    branch changes, with StrykerJS for TypeScript and cargo-mutants for
    Rust. The phase fails when a mutant survives. Cost: new dev
    dependencies, a slow gate phase, and triage of mutants that keep the
    behaviour. Effort: Large.
  - **D.** Add the same mutation check as a command that the review runs
    on the changed lines, outside the gate. It catches a weak test only
    when a reviewer runs it. Cost: new dev dependencies and a slower
    review. Effort: Medium.
- **Left out:** A property test for each rule: each rule needs its own
  property and a new library, and C or D finds the same gaps for every
  rule.
- **Wider fix:** C. It catches an unproven part of any rule, whatever its
  shape.
- **Previous examples:** `plans/follow-ups/image-build-oidc-case.md`,
  `plans/follow-ups/boat-fallback-journal-assertion.md`, and the label list
  order test of `calummoore/default-labels-rename`.
- **Recommendation:** D. It finds the class with the tools of C, without a
  slow gate phase, and it can become a gate phase later. C best protects
  the codebase.
- **Action:** ask the user, because C adds a gate phase, and D adds a
  review step with new dev dependencies.
