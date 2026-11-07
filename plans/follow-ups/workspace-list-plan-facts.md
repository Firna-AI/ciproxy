# Two workspace lists lack a shared plan summary

- **Severity:** Low. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-07, from `conductor/mvp-m19-ui`, pull request #36, the
  second owner review at `d304418`. A check of its finding 2, which the
  branch review numbers 12, proposed this shared summary as a wider fix.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-api/src/wire/links.rs:99`,
  `crates/ciproxy-api/src/wire/access.rs:56`,
  `ts/app/src/view/add-organization.ts:56`, and
  `ts/app/src/view/shell.ts:96`.
- **Reason:** This optional work applies only when both open decision items
  get answers that show the plan facts:
  [workspace-picker-plan-facts.md](workspace-picker-plan-facts.md) and
  [workspace-switch-plan-facts.md](workspace-switch-plan-facts.md).

- **Terms:**
  - **Plan facts:** the tier and the organization count of a workspace. The
    tier is the Free, Team, or Business plan. The count holds every linked
    organization, also an inactive one, but not a removed one.
  - **Workspace picker:** the list on the Add an organization return page,
    where a person with several Admin workspaces picks one.
  - **Workspace switch:** the menu of the top bar that lists the workspaces
    of the person.
- **Background:**
  - The approved mockups show the plan facts on each row of both lists,
    as in "Team · 1 organization". The picker rows come from `choiceText`
    in `docs/mockups/src/screens/settings/choose-workspace.tsx:50-57`. The
    switch rows come from
    `docs/mockups/src/components/shell/workspace-menu.tsx:33-40`.
  - The app shows neither fact. A picker row shows the name and, on a full
    Free plan, a hold note. A switch row shows the role, or "No role yet".
  - No API summary of either list carries the facts: `EligibleWorkspace`
    at `crates/ciproxy-api/src/wire/links.rs:97-106`, and `WorkspaceView`
    at `crates/ciproxy-api/src/wire/access.rs:54-69`.
  - Both handlers already read both facts and keep only a boolean, at
    `crates/ciproxy-api/src/handlers/link_views.rs:84-94` and
    `crates/ciproxy-api/src/handlers/sessions.rs:95-99` and `:121-122`.
  - `PlanTier` in `crates/ciproxy-api/src/wire/billing.rs:10-20` already
    names the tiers on the wire.
  - Each decision item can be applied on its own. This item blocks neither
    of them.
- **Problem:** When both lists show the plan facts, each list can get its
  own API fields and its own app formatter for the same facts.
- **Impact if no action:** The two lists can drift apart, and a new
  workspace list can copy either path.
- **Root question:** How do both workspace lists get the plan facts from one
  source, when both show them?
- **Options:**
  - **A.** Keep separate fields and a separate formatter for each list.
    Effort: Small.
  - **B.** Build one Rust summary of the plan facts with one helper. Carry
    it in `EligibleWorkspace` and in `WorkspaceView`, and format it with
    one app formatter. The later of the two changes can add it, or a small
    change can join them after both. Effort: Small.
- **Left out:** A ready display line from the API: B is better on every
  point, because B gives typed values, and product copy stays in the app.
  This item has two approaches only, because the decision items hold what
  each list shows.
- **Wider fix:** B. Both lists then read one type, so neither list can omit
  a fact that the other list shows.
- **Previous examples:** None found. A note of the task that built the app
  says that a shell banner also omits the tier and the count. Nobody
  checked that note.
- **Related:**
  - [workspace-picker-plan-facts.md](workspace-picker-plan-facts.md) and
    [workspace-switch-plan-facts.md](workspace-switch-plan-facts.md): this
    item waits for both answers. Neither of them waits for this item.
  - Option E of the picker item is B of this item, together with the
    changes of both lists.
- **Recommendation:** B. One summary keeps both lists on the same facts.
- **Action:** ask the user, because the work depends on the open answers of
  the two decision items. This item needs no answer of its own. When both
  answers show the plan facts, apply B. When either answer removes the
  facts from its design, delete this item.
