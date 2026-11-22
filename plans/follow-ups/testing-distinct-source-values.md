# Tests give two sources the same value

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from `conductor/ciproxy-boat-start-ready-deadline`,
  review round 1, the blocked wider fix F of finding 2.
- **Location:** `skills/testing/SKILL.md` in the repository
  `futex-ai/skills`, sections "Prove that each test can fail" and "Write one
  test". For option C in this repository: the test clocks at
  `crates/ciproxy-overflow/tests/service/harness.rs:54` and
  `crates/ciproxy-control-plane/tests/runtime/overflow_ports.rs:29`.
- **Reason:** The recommended rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.

- **Terms:**
  - **`testing` skill:** the shared rules for tests in
    `skills/testing/SKILL.md` of `futex-ai/skills`. Agents read it in every
    repository.
  - **Failure proof:** the step of the `testing` skill that breaks the code
    on purpose and sees the test fail.
  - **Dispatch time:** `starts_at` of a Boat start record, the time when
    ciproxy sent the create request to Boat.
  - **Launch time:** `launched_at` of a Boat start record, the time when
    ciproxy recorded the launch of the runner. The 10-minute ready deadline
    counts from it.
- **Background:**
  - The `testing` skill asks for literal expected values that come from the
    spec or a hand-worked example.
  - Its failure proof breaks the code by a revert, a flipped condition, or a
    returned default, and then sees the test fail.
  - It has no rule for code that picks one of several values of the same
    type, such as two times or two IDs.
  - On 2026-11-22, review round 1 of
    `conductor/ciproxy-boat-start-ready-deadline` found that the tests of the
    saved launch time gave the dispatch time and the launch time the same
    clock value.
  - A check then made both launch paths save `starts_at` in a copy of the
    tree. Every assertion still passed. The only failure was the unimock
    error "Mock for Clock::now was never called".
  - A projection that read `starts_at` instead of `launched_at` passed every
    control-plane test.
  - The fix moved the test clock between dispatch and launch, and it added a
    projection case.
- **Problem:** When a test gives two values of the same type one literal,
  code that uses the wrong value still passes. The failure proofs of the
  `testing` skill do not swap one source for the other, so they miss this.
- **Impact if no action:** Each review must find such a test again. A missed
  case lets a mix-up of two values reach `main`. In the example above, a
  slow-starting runner could lose part of its 10 minutes.
- **Root question:** How do tests prove that the code uses the right one of
  several values of the same type?
- **Options:**
  - **A.** Change nothing. Each review finds the case again. Effort: Small.
  - **B.** Add a rule to the `testing` skill: when the code picks one of
    several values of the same type, such as two times or two IDs, give each
    value a different literal, and swap the source in the failure proof. It
    guides the authors of every repository, but it enforces nothing. The
    change goes through the `writing-skills` skill of `futex-ai/skills`,
    with its recorded cases and checks. Effort: Medium.
  - **C.** Give such values different defaults in the test fixtures of each
    repository, for example a fake Boat launch that moves the test clock
    forward. Each repository and each pair of values needs its own change.
    The tests that assert a time after a launch then need new expected
    values. Effort: Medium.
- **Left out:**
  - A mutation-testing phase in the gate, such as `cargo-mutants`. Such a
    tool replaces function results and operators, but it does not swap one
    value for another, so this mix-up survives it. It is also a new gate.
  - A type that accepts only a clock reading from the launch save. A type
    cannot prove where a time value came from.
- **Wider fix:** B. It covers every repository and every pair of values,
  also in code that a later change adds. C covers only the fixtures that a
  repository changes.
- **Previous examples:** `f3d4b14`, of 5 November 2026, made a delayed retry
  save its own dispatch time instead of the time of the first attempt. Its
  test at `crates/ciproxy-overflow/tests/service/preparation_tests.rs:28-33`
  moves the clock to tell the two times apart.
- **Recommendation:** B. One rule reaches every repository and each pair of
  values, also in code that a later change adds.
- **Action:** ask the user, because the rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.
