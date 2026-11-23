# Shared templates omit the orchestrator's call

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-callback-diagnosis`, review
  round 2, finding 11. Its option C is the wider fix D of finding 10, the
  attribution in two Boat decision records.
- **Source:** Review.
- **Category:** Process.
- **Location:** `futex-ai/skills`: `plans/SKILL.md`,
  `plans/templates/plan.md`, and `delegating/SKILL.md`.
- **Reason:** The rules live in the shared-skills repository
  `futex-ai/skills`, so the fix needs a branch of that repository.

- **Terms:**
  - **Shared skills:** the agent instructions in the `futex-ai/skills`
    repository that the agents of each repository read, such as `plans`.
  - **User:** the human who owns the product, answers the decisions, and
    approves each protocol change.
  - **Orchestrator:** the agent session that runs a task for the user and
    starts delegates for its parts.
  - **Delegate:** an agent that the orchestrator starts for one part of a
    task, such as a subagent or a workspace agent.
  - **Design call:** a choice that the writing agent makes itself under the
    decision test, recorded with its date and reason.
- **Background:**
  - The plan template asks for two kinds of record: "Record each user
    decision with its date, the alternatives, and the reason. Record each
    design call that you made yourself with its date and reason, marked as
    your call." (`plans/templates/plan.md`). `plans/SKILL.md` gives the
    same two kinds.
  - `delegating/SKILL.md` says: "The orchestrator is the session that owns
    the plan and the primary branch."
  - It also lists the blocking decisions that go to the user:
    - a deletion against `origin/main`,
    - a requirement change in a protocol doc,
    - an irreversible or outward-facing action,
    - a High finding with `Action: ask the user`,
    - a choice where any assumption wastes the work.
  - An orchestrator also makes task calls for its delegates, such as a
    scope, a run order, a budget of paid resources, or a grade. No shared
    skill names such a call. A search on 21 November 2026 found none of
    "orchestrator's call", "owner call", "agent call", and "on behalf". It
    covered the `plans`, `delegating`, `decisions-and-findings`, and
    `models` skills.
  - So a delegate that gets such a call has two slots: a user decision, or
    its own design call. Neither fits.
- **Problem:** Records name the wrong party for a call. Each previous
  example gave the user a call or a request that no user answer backs.
  Each needed a later correction.
- **Impact if no action:** Later records can again name the wrong approver.
  A reader can then ask the wrong party, or take an agent's call as the
  user's approval.
- **Root question:** How do the shared skills keep apart the user's
  decisions, the orchestrator's calls, and the writer's own design calls?
- **Options:**
  - **A.** Add an attribution rule only, to `plans/SKILL.md`. Record an
    orchestrator's call as such, with its date. Attribute a call to the
    user only when the user made it. The plan template still shows two
    slots, so a writer who follows it meets no third slot. Effort: Small.
  - **B.** Add a template slot only, to `plans/templates/plan.md`: "Record
    each call of the orchestrator with its date and reason, marked as the
    orchestrator's call." The delegation rules stay silent, so a delegation
    prompt can still blur who made a call. Effort: Small.
  - **C.** Define the role, add the template slot, and add delegation
    guidance.
    - `delegating/SKILL.md`: an orchestrator's call covers only the
      authorized scope and budget of its task, such as its run order. Each
      blocking decision and each protocol decision stays with the user.
    - `delegating/SKILL.md`: a delegation prompt marks each user answer
      that it passes on, with its date. A delegate attributes a call to the
      user only when the prompt marks it so.
    - `plans/SKILL.md` and `plans/templates/plan.md`: a third record kind,
      the orchestrator's call, beside the user decision and the design call.
    - Review the change under the `writing-skills` skill.
    - A rule and a slot make the right role easy to choose. They cannot
      stop a wrong attribution. Effort: Small.
- **Left out:**
  - Change nothing, because it fails the goal. The class repeated four
    times.
  - A phrase lint or a gate for words such as "the user asked", because no
    check can know who made a call. A new lint or gate also needs the user.
  - An automatic inference of authority from the sender of a message,
    because a message can speak for the user without the user's answer.
  - A note in one repository only, because the class comes from the shared
    templates and recurs in each repository that uses them.
- **Wider fix:** C. It is the class fix for each later record. It is a
  writing rule and a template slot, not a structural fix, because no type
  or tool can know who made a call.
- **Previous examples:**
  - `26672f95`, 19 November 2026: "User's call" became "My call" for a
    docs mockup fix.
  - `34bc6209`, 21 November 2026: two follow-up items lost "the user
    approved only the locale fix". They also lost "asked to see each other
    live failure with its options". Their Action lines changed from "ask
    the user" to "fix now". The commit says that a new budget needs no user
    decision.
  - `88df2d91`, 21 November 2026: `plans/ciproxy-mvp.md` got an
    attribution correction. Commit `995e4f88` had attributed calls of the
    orchestrator to the user. The orchestrator had chosen two options and
    set the scope and the six-sandbox limit of a Boat task.
  - The first text of the [lock-timeout item][lock-timeout-item] and of
    `plans/follow-ups/boat-live-completion-acceptance.md` said that the
    user asked to see the options. No retained record showed who asked.
    Their later text names the user for the two decisions, and the
    orchestrator for the run order and the sandbox budget.
- **Recommendation:** C. It gives the role a definition, a place in each
  record, and a rule for delegates. It grants no agent the user's approval
  powers.
- **Action:** fix after merge, apply C, because the source lives in the
  shared-skills repository.

[lock-timeout-item]: https://github.com/Firna-AI/ciproxy/blob/2976db158049aefce6cfe96cf6337d12afc9be64/plans/follow-ups/boat-exit-report-lock-timeout.md
