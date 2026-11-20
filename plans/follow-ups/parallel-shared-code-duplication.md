# Parallel branches build the same base code

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`: the blocked wider fix D of
  finding 605, which finding 403 shares.
- **Source:** Review.
- **Category:** Process.
- **Location:** None in this repository. The change goes in
  `delegating/SKILL.md` of `futex-ai/skills`, under "Independence test".
- **Reason:** The change is in `futex-ai/skills`, not in this repository.
  The user chose A on 20 November 2026, in its own workspace there.

- **Terms:**
  - **`delegating` skill:** the shared procedure that splits work across
    agents and workspaces. It lives in `futex-ai/skills`, outside this
    repository.
  - **Independence test:** the list of conditions in the `delegating`
    skill that must all hold before agents work in parallel.
  - **Primary branch:** the branch of a plan. Every workspace branch starts
    from it and merges into it.
  - **Work item:** a milestone, or a group of TODOs of one milestone that
    touches its own files.
- **Background:**
  - The independence test of the `delegating` skill starts agents in
    parallel only when all of these hold:
    - The fault or the question of each task is known.
    - Each task is clear without the context of the other tasks.
    - No task needs the result of another task.
    - The tasks change different files and share no port, database,
      fixture, or build output.
    - No task needs a view of the whole system.
  - Two work items that each need the same new base control pass this
    test, because each writes its own copy in its own files.
  - The `product-ui` skill asks an agent to check `@firna/ui` and the
    app's own components before it builds a control. That rule works only
    on code that the agent can see. A branch cannot see the new files of a
    parallel branch.
  - The `delegating` skill already starts a milestone only after each
    milestone on its `Depends on:` line has merged.
  - The final review of 17 November 2026 found two cases of this class:
    - Dashboard base controls. On 8 November 2026, three parallel branches
      built two sets of panels, detail rows, status labels, text links,
      page headers, dialog frames, and link tabs: `58b12be` and `69b4f1c`
      from `a942f98`, and `166b2c9` from `52704fb`. The copies drifted in
      Back behaviour, mobile spacing, text sizes, and status text.
      `plans/follow-ups/dashboard-duplicate-controls.md` merges them.
    - The monthly price calculation. On 4 November 2026, billing and
      metering each built the tier prices, the add-on prices, and the
      monthly total on parallel branches: `4304e3f` and `a0a57e9`, both
      from `4bfa4e5`. The review sent it to a fix in the same branch, with
      one shared price table.
  - The copies are separate implementations, not copied text.
- **Problem:** The independence test checks shared files and results, but
  not a shared new piece. So work items that each need the same new base
  control or calculation start in parallel, and each builds its own copy.
- **Impact if no action:** Parallel branches keep building duplicate base
  controls and calculations. Each pair needs a later migration of Medium
  effort, and the copies drift until then.
- **Root question:** How do parallel work items avoid building the same new
  base control or calculation twice?
- **Options:**
  - **A.** Change nothing. The review of the primary branch finds each
    duplicate, and a follow-up item merges it. Cost: each duplicate costs a
    later migration, and its copies drift until then. Effort: Small.
  - **B.** Add one line to the independence test of the `delegating`
    skill. The line says that work items that each need the same new base
    control or calculation do not start in parallel until that piece is on
    the primary branch. Cost: less parallel work at the start of a batch.
    The orchestrator must see the shared piece when it splits the work.
    Effort: Small.
  - **C.** Add a planning rule to the `plans` skill. A plan puts each new
    base control or calculation that two milestones need in an earlier
    milestone. The dependent milestones name it on their `Depends on:`
    line. Cost: the planner must foresee the shared piece. A piece that the
    work finds later is not covered. Effort: Small.
  - **D.** Add a check to the merge step of the `delegating` skill. When a
    workspace branch comes back, the orchestrator compares its new controls
    and calculations with those of the primary branch and of the other open
    branches. Cost: it finds a duplicate only after the work, so one branch
    redoes its part. It is a detection fix. Effort: Small.
- **Left out:**
  - A duplicate-code detector in the gate: it fails the goal, because the
    copies are separate implementations, not copied text. It is also a new
    gate, which needs the user.
  - A rule in the `AGENTS.md` of this repository: that file holds only the
    rules that are specific to this repository. The delegation procedure
    lives in the shared skills, and the same class can happen in every
    repository.
- **Wider fix:** B. It stops the class at the step where the work splits,
  also for a piece that the plan did not foresee. C stops it only for the
  pieces that the plan foresees. D detects it after the work.
- **Previous examples:** The dashboard base controls, from commits
  `58b12be`, `69b4f1c`, and `166b2c9`, and the monthly price calculation,
  from commits `4304e3f` and `a0a57e9`.
- **Related:** `plans/follow-ups/dashboard-duplicate-controls.md`, a shared
  wider fix. It merges the dashboard copies inside this repository. Neither
  item waits for the other.
- **Recommendation:** B. One line at the step where the orchestrator splits
  the work stops both known cases, also when the plan did not foresee the
  shared piece.
- **Action:** fix after merge, apply A, because the user chose A on
  20 November 2026, as its own workspace in `futex-ai/skills`. Delete this
  item when that pull request merges.
