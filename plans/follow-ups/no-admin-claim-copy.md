# No-role page misstates a workspace with no Admin

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 1203 found.
- **Source:** Review.
- **Category:** UX wording.
- **Location:** `ts/app/src/view/access.ts`, `noRolePage`, its `lead`,
  `ts/app/src/screens/no-role.tsx`, and
  `docs/mockups/src/screens/auth/no-role.tsx`.
- **Reason:** No doc or mockup covers this page in a workspace with no
  Admin, so the fix needs a new page state first.

- **Terms:**
  - **Page for no role:** the page that a member of an organization of the
    workspace sees while the person has no role. `noRolePage` in
    `ts/app/src/view/access.ts` builds it.
  - **Admin claim:** the action that gives an eligible organization owner
    the Admin role with no approval of another Admin. The API offers it
    through `can_claim_admin`.
- **Background:**
  - `noRolePage` gives the title "Waiting for an Admin of" and the name of
    the workspace. Its lead says: "ciproxy sent your request for access to
    the Admins. When one of them gives you a role," and then that the
    workspace opens here.
  - It uses this lead in every case, also when `workspace.admins` is empty.
    Then the page shows no list of Admins.
  - A workspace can lose its last Admin. "Who gets a role" in
    `docs/protocol/security.md` covers this case. A person who leaves every
    organization of the workspace loses the role at the next membership
    check.
  - The same section says that an owner of an organization of the
    workspace can claim the Admin role when the workspace has no Admin. The
    server still requires the person to own an organization.
  - So in such a workspace, an owner sees the claim notice. A member who is
    not an owner can do nothing on the page.
  - The same section says that a member with no role sees only a page that
    asks an Admin for access. That page names the Admins. The section says
    nothing about a workspace with no Admin.
  - No mockup shows the page for no role in a workspace with no Admin. The
    `mockups` skill asks for the mockup state before the screen changes.
  - The dashboard reads only `can_claim_admin`. The server alone decides
    who can claim.
  - On 17 November 2026 the final review gave the claim notice and the
    account menu line the words of the corrected mockups. That fix does not
    change the lead.
- **Problem:** In a workspace with no Admin, the page for no role says that
  ciproxy sent the request to the Admins. It also says that the workspace
  opens when one of them gives a role. No Admin exists to do that.
- **Impact if no action:** A member with no role waits for an answer that
  cannot come. The page does not say what can help, such as an owner who
  claims the Admin role.
- **Root question:** What should the page for no role tell a person in a
  workspace that has no Admin?
- **Options:**
  - **A.** Drop only the false sentence for a workspace with no Admin, and
    keep the rest of the page. Cost: the page then gives the person no next
    step. Effort: Small.
  - **B.** Add a mockup state for a workspace with no Admin first. Then give
    the page that state. Say that the workspace has no Admin yet, and that
    an owner of one of its organizations on GitHub can claim the Admin
    role. Keep the claim notice for an owner. Effort: Small.
  - **C.** Make the server keep an Admin in each workspace, for example with
    a role for an owner when the last Admin goes. Cost: it changes the role
    rules of `docs/protocol/security.md`. Effort: Medium.
- **Left out:** A dashboard rule that works out who can claim from the
  organizations: it copies the server rule, and the dashboard reads only
  `can_claim_admin`.
- **Wider fix:** None. The `mockups` skill already requires a mockup state
  for each state of a screen.
- **Previous examples:** Round 1 and round 2 of the same review found older
  claim words in the mockups and in the dashboard. Commit `3428fd3` of 17
  November 2026 corrected the mockups.
- **Recommendation:** B. It removes the false sentence, and it gives the
  person the one next step that the role rules allow.
- **Action:** ask the user, because B adds a page state with new copy for a
  case that no protocol doc and no mockup covers.
