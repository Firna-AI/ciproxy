# Follow-up items keep replaced decisions

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`: the blocked wider fix D of
  finding 1207, with `plans/follow-ups/cache-sync-github-deletions.md` as
  its second case.
- **Source:** Review.
- **Category:** Process.
- **Location:** None in this repository. The change goes in
  `references/follow-up-items.md` of the `plans` skill in
  `futex-ai/skills`.
- **Reason:** The change is in `futex-ai/skills`, not in this repository.
  The user chose A on 20 November 2026, in its own workspace there.

- **Terms:**
  - **Follow-up item:** one file under `plans/follow-ups/` that records work
    that waits until after a merge. It holds every fact that the work
    needs.
  - **`plans` skill:** the shared procedure for plans and follow-up items.
    It lives in `futex-ai/skills`, outside this repository. Its file
    `references/follow-up-items.md` says how to add and write an item.
  - **Protocol doc:** a file under `docs/protocol/` that holds the
    requirements and the decisions in force.
- **Background:**
  - A follow-up item copies the decisions and the facts that its work
    needs, so that it stands alone after the plan goes.
  - A later change can replace such a decision, or edit the protocol
    section that an item names. No rule then tells the author of the
    change to update the item.
  - The current rules do not cover this class:
    - The search of the `git-commit` skill covers removed or renamed names.
    - The alignment rule of the `documentation` skill covers `docs/` and
      the READMEs.
    - The rules of `references/follow-up-items.md` cover only how to write
      an item.
  - The final review of 17 November 2026 found two cases:
    - `plans/follow-ups/job-facts-from-the-hook.md` still said that the
      GitHub API gives the full ref of a job. On 5 November 2026 the user
      moved the full ref to the checked runtime token. Commit `2a61e1c9`
      fixed the architecture pages, but not the item.
    - That item also said that a new fact source lets the App drop
      "Actions" read. But the GitHub cache entry list and the job-time
      recovery also use that permission.
    - `plans/follow-ups/cache-sync-github-deletions.md` said that the App
      has runner access only, and that "Actions" read would be a new
      permission. The App has had "Actions" read since 4 November 2026,
      and it already lists the GitHub cache entries with it.
  - The final review gave each of the two items a direct fix.
- **Problem:** When a change replaces a decision or edits a protocol
  section, the follow-up items that copy it keep the old facts. No rule
  asks the author of the change to update them.
- **Impact if no action:** A worker who takes such an item starts from a
  wrong trust boundary, permission, or decision. The worker can plan work
  that a current decision rules out.
- **Root question:** How do follow-up items stay current when a later change
  replaces a decision or a protocol section that they copy?
- **Options:**
  - **A.** Change nothing. A review finds each stale item, as the final
    review did. Cost: a worker can start from a stale item before a review
    finds it. Effort: Small.
  - **B.** Add one rule to `references/follow-up-items.md` of the `plans`
    skill. The rule: a change that edits a protocol section or replaces a
    decision also updates each follow-up item whose **Location** or body
    names it. Cost: the change is in the shared skills, so it needs the
    user. The rule is a reminder, not a check. Effort: Small.
  - **C.** Give each follow-up item a field that lists the protocol sections
    and the decisions that its work depends on. A change then searches
    `plans/follow-ups/` for the section or the decision that it edits.
    Cost: a new field in the template of the `plans` skill, a change of
    every existing item, and the same need for the user. The field can go
    stale too. Effort: Medium.
- **Left out:**
  - Links in place of copied facts: it fails the goal. The follow-up rules
    require each item to copy the decisions that its work depends on, so
    that it stands alone after the plan goes.
  - A rule in the `AGENTS.md` of this repository: that file holds only the
    rules that are specific to this repository. The follow-up procedure
    lives in the shared skills, and the same class can happen in every
    repository.
- **Wider fix:** B. It acts at the change that makes an item stale, in
  every repository. C needs the same search plus a new field in every
  item.
- **Previous examples:** The two cases above, both found on 17 November
  2026.
- **Related:**
  - `plans/follow-ups/job-facts-from-the-hook.md`: the first case of this
    class. Its direct fix does not wait for this item.
  - `plans/follow-ups/cache-sync-github-deletions.md`: the second case of
    this class. Its direct fix does not wait for this item.
- **Recommendation:** B. One rule at the step that makes an item stale
  covers both known cases, with no new field.
- **Action:** fix after merge, apply A, because the user chose A on
  20 November 2026, as its own workspace in `futex-ai/skills`. Delete this
  item when that pull request merges.
