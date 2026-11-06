# Owner claim mockups keep the old rule

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-05, from the implementation handoff for PR #29.
- **Source:** Implementation.
- **Category:** Mockup.
- **Location:**
  `docs/mockups/src/components/shell/account-menu.tsx:25`,
  `docs/mockups/src/screens/auth/no-role.tsx:84`,
  `docs/mockups/src/entries/app/access/account-menu.mockup.tsx:11`,
  `docs/mockups/src/entries/app/access/index.mockup.tsx:28`,
  `docs/mockups/src/fixtures/members.ts:1`, and
  `docs/protocol/security.md:105`.
- **Reason:** This ownership mismatch predates the workspace rename and
  cache states completed in this branch.

- **Terms:**
  - **Workspace:** The ciproxy account of a customer, with one or more
    GitHub organizations.
  - **Admin:** The ciproxy role that controls the machines, Boat account,
    and runners of every organization in the workspace.
  - **Owner claim:** An action that gives a GitHub organization owner the
    Admin role without a grant from another Admin.
- **Background:** `docs/protocol/security.md` permits an owner claim when
  the workspace has no Admin, or the person owns every organization.
  A person with another ciproxy role claims from the account menu.
  The page for a person without a role follows the same eligibility rule.
  Ownership comes from the person's GitHub sign-in memberships.
- **Problem:** `AccountMenu` checks only `person.owner` and a non-Admin
  role. `NoRoleScreen` offers a claim to every person in its owner state.
  Their entries and the members fixture still say that any owner can
  always claim Admin. The fixture workspace has mokly-ai and mokly-labs,
  with Cal Moore and Ana Souza as Admins. Its ownership flag describes
  ownership of mokly-ai, without ownership of every organization.
- **Impact if no action:** The mockups teach an obsolete access rule.
  Dashboard implementation can copy a rule that lets one organization
  take control of another organization's runners and job secrets.
- **Root question:** How should the mockups show which GitHub owners can
  claim the workspace Admin role?
- **Options:**
  - **A.** Add the required ownership and Admin conditions separately in
    both screens. Align their entries and fixture comments. Show eligible
    and ineligible owners in both viewport variants. Effort: Small.
  - **B.** Define one claim eligibility model in the fixtures and let both
    screens read it. Show recovery with no Admin, ownership of every
    organization, and partial ownership while an Admin exists. Align the
    entries, comments, and account-menu person data. Effort: Medium.
- **Left out:** Keep any owner eligible, because it permits takeover of
  another organization's runners. Hide every claim, because it removes
  required recovery when no Admin exists. Only the two repairs above
  preserve the approved rule and the claim action.
- **Wider fix:** B. One eligibility model prevents the account menu and
  access page from showing different rules.
- **Previous examples:** Commit `05c7770` removed obsolete GitHub
  runner-right denial mockups after the workspace Admin rule changed.
- **Recommendation:** B. Both claim surfaces then show the same approved
  rule from one source.
- **Action:** fix after merge, apply B. This existing mockup mismatch is
  outside the completed workspace and cache TODO. Run
  `npm --prefix ts run mockups:check` in the later change. Inspect the
  affected screens at desktop widths of 1440 and 720 pixels, and mobile
  width of 390 pixels.
