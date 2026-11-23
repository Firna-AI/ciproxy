# Docs restate external rules without conditions

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), review round 1, the
  wider fix E of the check of finding 5.
- **Location:** `skills/documentation/SKILL.md` in the repository
  `futex-ai/skills`. The case in this repository:
  `crates/ciproxy-github/scale-sets.md`, section "Runner groups".
- **Reason:** The recommended rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.

- **Terms:**
  - **`documentation` skill:** the shared rules for docs in
    `skills/documentation/SKILL.md` of `futex-ai/skills`. Agents read it in
    every repository.
  - **Scale set:** a GitHub pool of runners in a runner group that serves
    the jobs of one label.
- **Background:**
  - On 2026-11-23, the first text of the "Runner groups" section of
    `crates/ciproxy-github/scale-sets.md` said: "When both environments
    have an online scale set for the label of a job, GitHub gives the job
    to either environment. GitHub has no setting that controls this
    choice."
  - It cited only GitHub's runner scale-set guide. That guide's rule holds
    only when both scale sets are eligible for the job. GitHub's guide
    "Choosing the runner for a job" documents `runs-on` with `group` and
    `labels` for self-hosted runners.
  - Review round 1 of pull request #142 found the overstated condition. The
    fix narrowed it to eligible scale sets, added the group rule with its
    limits, and linked the exact section of each GitHub page.
  - The `documentation` skill asks docs to match the behaviour of the
    implementation. It says nothing about the rules of an external
    service.
- **Problem:** A doc that restates a rule of an external service can drop
  the conditions or the setup of its source, and no rule asks the writer
  to keep them.
- **Impact if no action:** Readers act on a rule that the service does not
  have, such as a missing way to pin a job to one runner group.
- **Root question:** How does a doc keep the conditions of an external
  rule that it restates?
- **Options:**
  - **A.** Change nothing. Each review finds the case again. Effort:
    Small.
  - **B.** Add a rule to the `documentation` skill: a doc that restates a
    rule of an external service links the exact section, and keeps the
    conditions and the setup of the source. The change goes through the
    `writing-skills` skill of `futex-ai/skills`. Effort: Small.
- **Left out:** A test that reads the docs. The `testing` skill says that
  doc prose never gets a test.
- **Wider fix:** B. It reaches every doc of every repository.
- **Previous examples:** Review finding 5 of pull request #142, as the
  Background says.
- **Recommendation:** B. One sentence in the shared skill covers each
  restated external rule.
- **Action:** ask the user, because the rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.
