# Fake lists omit what the service returns

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), review round 2, the
  wider fix D of the check of finding 6.
- **Location:** `skills/testing/SKILL.md` in the repository
  `futex-ai/skills`, section "Write one test". The case in this
  repository: `crates/ciproxy-github/tests/integration/resource_http.rs`
  and `crates/ciproxy-github/tests/integration/fake_http.rs`.
- **Reason:** The recommended rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.

- **Terms:**
  - **`testing` skill:** the shared rules for tests in
    `skills/testing/SKILL.md` of `futex-ai/skills`. Agents read it in every
    repository.
  - **Fake list:** a list response of a test double that stands in for a
    remote service, such as the runner groups of a GitHub organization.
- **Background:**
  - `RestClient::sync_group` in `crates/ciproxy-github/src/rest/groups.rs`
    lists the runner groups of an organization and searches the list for
    the full managed name.
  - Until review round 2 of pull request #142, the fake GitHub server of
    the integration tests listed only the group of the test, or nothing.
    GitHub always lists the organization's `Default` group, first in its
    REST example.
  - On 2026-11-23, a check made the lookup inspect only the first listed
    group, in a copy of the tree. Every test still passed.
  - The fix put the `Default` group first in each fake list, and added a
    case where the managed group sits between `Default` and the group of
    another environment.
- **Problem:** When a fake list holds only the entry that the test needs,
  code that reads the wrong entry, or only the first entry, still passes.
  The failure proofs of the `testing` skill do not change the position of
  the target in a list, so they miss this.
- **Impact if no action:** Each review must find such a fake again. A
  lookup that misses its target in a real list can reach `main`.
- **Root question:** How do tests prove that code finds its target in a
  list that also holds the entries the real service returns?
- **Options:**
  - **A.** Change nothing. Each review finds the case again. Effort:
    Small.
  - **B.** Add a rule to the `testing` skill: a fake list that code
    searches holds the entries that the real service always returns, such
    as a default entry. The target sits between other entries. Later
    reads of the fake show the fake's own writes. The rule guides the
    authors of every repository, but it enforces nothing. The change goes
    through the `writing-skills` skill of `futex-ai/skills`. Effort:
    Small.
  - **C.** Fix each fake of each repository when a review finds it.
    Effort: Medium.
- **Left out:** A mutation-testing gate. It is a new gate, and it does not
  move a target in a list.
- **Wider fix:** B. It reaches every repository and every fake list.
- **Previous examples:** Review finding 6 of pull request #142, as the
  Background says.
- **Related:** `plans/follow-ups/runner-group-list-second-page-test.md` is
  a second gap of the same fake: no test lists a second page.
- **Recommendation:** B. One rule reaches each fake list, also in code that
  a later change adds.
- **Action:** ask the user, because the rule changes the shared skills in
  `futex-ai/skills`, another repository that holds the working rules of
  every repository.
