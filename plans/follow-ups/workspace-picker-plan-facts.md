# Workspace choices omit approved plan information

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-07, from `conductor/mvp-m19-ui`, pull request #36, the
  second owner review at `d304418`, its finding 2, which the branch review
  numbers 12. A check of that finding added options D and E. Item 91 of
  `plans/ciproxy-mvp.md` records the same choice.
- **Source:** Review.
- **Category:** Mockup.
- **Location:** `ts/app/src/view/add-organization.ts:56`,
  `docs/mockups/src/screens/settings/choose-workspace.tsx:51`,
  `crates/ciproxy-api/src/wire/links.rs:99`, and
  `crates/ciproxy-api/src/handlers/link_views.rs:84`.
- **Reason:** Only the user can say which side of the mismatch is right,
  and the owner sends the answer later.

- **Terms:**
  - **Workspace picker:** the list on the Add an organization return page.
    A person with several Admin workspaces picks the workspace of the new
    organization there.
  - **Tier:** the Free, Team, or Business plan of a workspace. `PlanTier` in
    `crates/ciproxy-api/src/wire/billing.rs:13` names it on the wire.
  - **Organization count:** the linked organizations of a workspace, also
    the inactive ones, but not the removed ones.
  - **`EligibleWorkspace`:** the API summary of one offered workspace, at
    `crates/ciproxy-api/src/wire/links.rs:97-106`.
  - **Hold note:** the line that says that the new organization stays
    inactive in that workspace until the workspace moves to Team.
- **Background:**
  - On 6 November 2026, the user chose this flow: after the install,
    ciproxy lists the workspaces where the person is an Admin, and the
    person picks one.
  - The approved mockup is the "Add an organization" entry at
    `docs/mockups/src/entries/app/settings/organizations/add/index.mockup.tsx`.
    Each row shows the name and a body from `choiceText`, at
    `choose-workspace.tsx:50-57`, such as "Team · 1 organization". On a full
    Free plan, the body adds the hold note.
  - The app maps each offer to an ID, a name, a note, and a checked flag,
    at `ts/app/src/view/add-organization.ts:56-63`. The note exists only
    when `requires_upgrade` is true.
  - `ts/app/src/screens/add-organization/link-section.tsx:54-60` gives each
    `RadioCard` the name as its title and the note as its body. A row
    without a hold shows the name only.
  - `EligibleWorkspace` holds only `id`, `name`, and `requires_upgrade`. The
    generated type in `ts/api/generated/api.d.ts` is the same.
  - The offer handler already reads both facts and keeps only the boolean,
    at `crates/ciproxy-api/src/handlers/link_views.rs:84-94`. It counts the
    organizations that are not removed, and it reads the billing tier.
  - `docs/protocol/workspaces.md:120-121` asks only for the hold: "The list
    says when the tier of a workspace keeps the organization inactive
    there." The app meets it. The tier and the count come only from the
    mockup.
  - Every option keeps the hold note. Every option also keeps the count
    that decides a hold, as `docs/protocol/workspaces.md:157-158` and
    `:167-168` say. C reads its count from a third source, so C must give
    the same number.
  - Plan and usage is still a coming-soon page in the app, at
    `ts/app/src/app/billing.tsx`. So the app shows both facts nowhere yet.
  - No test checks a tier or a count. `ts/app/_tests_/links.test.ts:30-62`
    checks the unpicked start and the hold note. The browser case
    `links-picker` checks the heading and one PUT, at
    `ts/app/scripts/browser-cases.ts:111-116` and
    `ts/app/scripts/browser-actions.ts:64-73`.
    `crates/ciproxy-api/tests/http/link_flow_tests.rs:133-153` checks that
    two offers come back.
  - The browser case `links-long-names-visible` builds this picker with a
    long workspace name, at
    `ts/app/scripts/harness-review-scenarios.ts:22-32`. It checks the full
    Add label at each width, at `ts/app/scripts/browser-long-actions.ts:43-62`.
    Options B to E add a line to each row, so each must keep that case
    passing.
  - A and C change no API. B, D, and E change the Rust API and the
    generated types.
  - The task that built the picker had no grant to change Rust, the
    generated API, or the mockups. It reported the gap as an API limit. No
    user decision chose the reduced picker.
- **Problem:** The picker shows only the name and an optional hold note.
  `EligibleWorkspace` supplies neither the tier nor the count. The review
  saw both facts absent at all three widths that it checked.
- **Impact if no action:** The person cannot compare the offered plans and
  organization counts before confirming the destination.
- **Root question:** What context should the workspace picker give before
  confirmation?
- **Options:**
  - **A.** Approve the reduced information and update the mockup. This
    removes the tier and the count from the design. Effort: Small.
  - **B.** Add the tier and the count to the Rust summary. Regenerate the
    API types and render the approved information. Effort: Medium.
  - **C.** Read authorized plan and organization summaries separately. Add
    loading and error states for those extra requests. Effort: Medium.
  - **D.** Add the tier and the count to `WorkspaceView`, the summary of
    each workspace in the session. The picker finds each offer there by
    ID, as `workspaceName` already does for linked names. The session
    handler already reads both facts, at
    `crates/ciproxy-api/src/handlers/sessions.rs:95-99` and `:121-122`. So
    D adds no store read. But every page then carries the two facts, and
    the picker depends on a second list with another source. The session
    lists workspaces through memberships of organizations that are not
    removed, at `crates/ciproxy-store-pg/src/dashboard/identity.rs:136-140`.
    The offers are stored, at
    `crates/ciproxy-store-pg/src/installation_links/offers.rs:30-34`. The
    handler keeps only the offers where the person is still an Admin, at
    `link_views.rs:80-83`. So a rare offer can lack its facts.
    Effort: Medium.
  - **E.** Build one Rust workspace summary, the tier and the count, with
    one helper. Carry it in `EligibleWorkspace` and in `WorkspaceView`.
    Show it with one app formatter in the picker and in the workspace
    switch, as both mockups do. A new workspace list then gets the facts
    from the same type. E also changes the workspace switch, which needs
    its own answer, also for the rows with no role. Effort: Medium.
- **Left out:**
  - Deriving the plan or the count from `requires_upgrade`: it fails the
    goal, because it gives neither fact.
  - Change nothing: it breaks the rule that the mockups and the product
    agree, and that rule stays. `docs/mockups/README.md:4` says that the
    mockups are part of the spec.
  - A tier and a count saved with each stored offer: it fails the goal,
    because both facts can change before the person picks.
  - A ready display line from the API: B is better on every point. B gives
    the same facts as typed values, and product copy stays in the app.
  - A separate route for the offer facts: B is better on every point, with
    one read and no new route.
- **Wider fix:** E. The rules that the mockups match the product and that
  Rust owns the boundary data are in force. But two workspace lists of the
  app still omit the facts. E removes the case with one Rust summary and
  one app formatter. Like every option here, E waits for the user. After
  an answer B or D, the shared summary stays open as optional work in
  [workspace-list-plan-facts.md](workspace-list-plan-facts.md).
- **Previous examples:** The workspace switch of the same app omits the
  same facts. See
  [workspace-switch-plan-facts.md](workspace-switch-plan-facts.md).
- **Related:**
  - [workspace-switch-plan-facts.md](workspace-switch-plan-facts.md) records
    the same gap in the workspace switch. Only option E reaches the switch.
    Each other option here can be applied without the switch answer.
  - [workspace-list-plan-facts.md](workspace-list-plan-facts.md) holds the
    optional shared summary for both lists. It waits for this answer and
    the switch answer. No option here waits for it.
  - Item 91 of `plans/ciproxy-mvp.md`, "The workspace picker lacks the plan
    and organization count", is a repeat of this question. One answer
    settles both records.
- **Recommendation:** B. It keeps the approved design through one generated
  contract, and the handler already reads both facts. B does not wait for
  the switch answer. E best protects the codebase when the switch item gets
  an answer that shows the same facts.
- **Action:** ask the user, because the implementation and the approved
  mockup give different information, and no recorded call chooses the
  reduced design. Change neither the product nor the mockup before the
  answer.
