# The workspace switch shows roles, not plan facts

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-07, from `conductor/mvp-m19-ui`, pull request #36. The
  check of finding 2 of the second owner review found this new problem at
  `d304418`, inside the diff of that branch.
- **Source:** Review.
- **Category:** Mockup.
- **Location:** `ts/app/src/view/shell.ts:96`,
  `ts/app/src/components/shell/menus.tsx:47`,
  `docs/mockups/src/components/shell/workspace-menu.tsx:35`, and
  `crates/ciproxy-api/src/wire/access.rs:56`.
- **Reason:** Only the user can say which side of the mismatch is right,
  and the check found it after the last review round of that branch.

- **Terms:**
  - **Workspace switch:** the menu of the top bar of the app. It lists the
    workspaces of the signed-in person.
  - **Tier:** the Free, Team, or Business plan of a workspace. `PlanTier` in
    `crates/ciproxy-api/src/wire/billing.rs:13` names it on the wire.
  - **Organization count:** the linked organizations of a workspace, also
    the inactive ones, but not the removed ones.
  - **`WorkspaceView`:** the summary of each workspace in the session
    response, at `crates/ciproxy-api/src/wire/access.rs:54-69`.
- **Background:**
  - The approved design is the "Switch workspaces" mockup, at
    `docs/mockups/src/entries/app/access/workspace-menu.mockup.tsx`. Its
    rows come from `docs/mockups/src/components/shell/workspace-menu.tsx:33-40`.
    Each row shows the name and a caption such as "Team · 2 organizations".
  - The mockup shows a workspace where the person is an Admin, and one
    where the person is a Viewer. It shows no row for a workspace where the
    person has no role yet.
  - `workspaceMenu` in `ts/app/src/view/shell.ts:88-103` builds the rows of
    the app. Each detail is the role name, or "No role yet".
    `ts/app/src/components/shell/menus.tsx:47` shows that detail as the
    second line of the row.
  - The session lists every workspace whose organizations contain the
    person, also one where the person has no role.
  - `WorkspaceView` has no tier and no count. The session handler already
    reads both, at `crates/ciproxy-api/src/handlers/sessions.rs:95-99` and
    `:121-122`. It keeps only `organization_limit_exceeded`.
  - `docs/protocol/security.md:89-91` says that a member with no role asks
    for access by signing in, and sees only a page that asks an Admin for
    access.
  - The account menu already shows the role in the current workspace,
    through `accountMenu` in `shell.ts`.
  - `ts/app/README.md:10` says that the screens follow the mockups.
    `docs/mockups/README.md:4` says that the mockups are part of the spec.
  - The task that built the switch had no grant to change Rust, the
    generated API, or the mockups. It reported the gap as an API limit. No
    user decision chose the role in place of the plan facts.
- **Problem:** The switch shows the role, or "No role yet", where the
  approved mockup shows the tier and the organization count of each
  workspace. `WorkspaceView` supplies neither fact.
- **Impact if no action:** The product and its approved design stay apart.
  The person cannot see the plan or the size of another workspace in the
  app, because Plan and usage is still a coming-soon page.
- **Root question:** What does each row of the workspace switch say about
  its workspace?
- **Options:**
  - **A.** Keep the role and "No role yet", and update the mockup to show
    the role. The tier and the count then leave the design of the switch.
    Effort: Small.
  - **B.** Show the tier and the count on each row where the person has a
    role, as the mockup does. Keep "No role yet" on a row with no role.
    Add both facts to `WorkspaceView`, and regenerate the API types.
    Effort: Medium.
  - **C.** Show the tier and the count on every row, also on a row with no
    role. A member with no role then sees the plan and the size of that
    workspace. So the user must confirm that the rule of
    `docs/protocol/security.md:89-91` allows it. Effort: Medium.
  - **D.** Show the role, the tier, and the count together, such as
    "Viewer · Free · 1 organization". Update the mockup to match.
    Effort: Medium.
- **Left out:**
  - Change nothing: it breaks the rule that the mockups and the product
    agree.
  - Derive the facts from `organization_limit_exceeded`: it fails the goal,
    because that flag gives neither fact.
  - A separate request for each workspace of the switch: B is better on
    every point, with no new request and no new loading state.
- **Wider fix:** None in this item. The optional shared summary of
  [workspace-list-plan-facts.md](workspace-list-plan-facts.md) prevents the
  class: one Rust summary and one app formatter for both lists. It waits
  for this answer and the picker answer. No option here waits for it.
- **Previous examples:** The workspace picker of the same app omits the
  same facts. See
  [workspace-picker-plan-facts.md](workspace-picker-plan-facts.md).
- **Related:**
  - [workspace-picker-plan-facts.md](workspace-picker-plan-facts.md) records
    the same gap in the workspace picker. Neither answer waits for the
    other. Its option E also changes this switch, but E does not say what
    a row with no role shows. So after an answer E there, the rows with no
    role still need an answer here.
  - [workspace-list-plan-facts.md](workspace-list-plan-facts.md) holds the
    optional shared summary for both lists. It waits for both answers.
- **Recommendation:** B. It shows the approved facts, keeps the access rule
  for a member with no role, and adds no store read.
- **Action:** ask the user, because the implementation and the approved
  mockup give different information, and no recorded call chooses the role
  in place of the plan facts. Change neither the product nor the mockup
  before the answer.
