# Doc rules skip the crate guides

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), review round 2, the
  wider fix D of the check of finding 8.
- **Location:** `skills/documentation/SKILL.md` in the repository
  `futex-ai/skills`, its opening rules and its search step for a removal
  or a rename. The case in this repository:
  `crates/ciproxy-github/scale-sets.md`, section "Workspace ownership".
- **Reason:** The recommended rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.

- **Terms:**
  - **`documentation` skill:** the shared rules for docs in
    `skills/documentation/SKILL.md` of `futex-ai/skills`. Agents read it in
    every repository.
  - **Crate guide:** a Markdown file beside a crate's `README.md` that is
    not itself a `README.md`, such as `crates/ciproxy-github/scale-sets.md`.
- **Background:**
  - Pull request #142 gave each environment its own runner group in an
    organization, and updated the "Runner groups" section of
    `crates/ciproxy-github/scale-sets.md`.
  - Its "Workspace ownership" section still said that all workspaces of
    one GitHub organization share its managed runner group. Review round 2
    found it. The fix limited the sentence to one environment.
  - The `documentation` skill lists `docs/`, protocol docs, mockups, and
    every `README.md` as part of the spec. It does not list crate guides.
  - Its search step for a removal or a rename covers `docs/`, `plans/`,
    and every `README.md`. It has no step for a change that splits or
    narrows a shared thing.
- **Problem:** A change that splits or narrows a shared thing can leave a
  crate guide that still states the old sharing, and no rule sends the
  writer to the guide.
- **Impact if no action:** Crate guides keep statements that contradict
  the code, until a review or a reader finds them.
- **Root question:** How does a change that narrows who shares a thing
  reach every doc that states it?
- **Options:**
  - **A.** Change nothing. Each review finds the case again. Effort:
    Small.
  - **B.** Add two rules to the `documentation` skill: list crate guides
    among the docs that are part of the spec, and when a change splits or
    narrows a shared thing, search every doc and crate guide for each
    statement of who shares it or how many exist. The change goes through
    the `writing-skills` skill of `futex-ai/skills`. Effort: Small.
- **Left out:** A test that reads the docs. The `testing` skill says that
  doc prose never gets a test.
- **Wider fix:** B. It reaches every crate guide of every repository.
- **Previous examples:** Review finding 8 of pull request #142, as the
  Background says.
- **Related:** `plans/follow-ups/markdown-check-crate-guides.md` asks
  whether the gate checks the links and the lint of crate guides. It does
  not check their content.
- **Recommendation:** B. Two short rules close the gap for every crate
  guide.
- **Action:** ask the user, because the rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.
