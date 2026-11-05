# Organizations lacks the removed App state

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-05, from the implementation handoff for PR #29.
- **Source:** Implementation.
- **Category:** Mockup.
- **Location:**
  `docs/mockups/src/screens/settings/organizations.tsx:30`,
  `docs/mockups/src/screens/settings/organization-list.tsx:55`,
  `docs/mockups/src/entries/app/settings/organizations/states/states.mockup.tsx`,
  `docs/mockups/src/screens/auth/app-removed.tsx:18`,
  `docs/mockups/src/fixtures/access.ts:13`, and
  `docs/protocol/app-removal.md:24`.
- **Reason:** This existing external App-removal gap is separate from the
  completed Admin take-out flow.

- **Terms:**
  - **Workspace:** The ciproxy account of a customer, with one or more
    GitHub organizations.
  - **External removal:** An owner removes the ciproxy GitHub App on
    GitHub, without using the workspace's Take out action.
  - **Take-out:** An Admin takes an organization out through ciproxy,
    which removes its own App installation.
- **Background:** `docs/protocol/app-removal.md` requires Organizations to
  show an externally removed organization for 30 days. The row must give
  the last day on which a new installation keeps its job history.
  ciproxy deletes the organization's data after 30 days. An installation
  within the window returns it to its workspace with its history, unless
  another workspace's Admin starts it with Add an organization.
- **Problem:** `OrganizationsScreen` has Team, Free, over-limit, single
  organization, take-out, and rename states, but no removed state.
  `OrganizationList` renders active repository access and actions only.
  The separate `AppRemovedScreen` covers removal of the last organization,
  rather than a removed organization beside an active one in Settings.
- **Impact if no action:** The mockups omit the recovery deadline and
  retained history for a removed organization. Dashboard implementation
  can miss this required organization state.
- **Root question:** How should Organizations show a removed organization
  during its 30-day recovery window?
- **Options:**
  - **A.** Add a removed state with a branch that renders a removed row.
    Show the removal date and recovery deadline beside active
    organizations. Keep active organization actions separate.
    Effort: Small.
  - **B.** Give active and externally removed organizations a shared
    fixture model with distinct states. Render the retained row and
    deadline from that model in both variants. Derive actions and the
    active organization count from its state. Effort: Medium.
- **Left out:** Show only the removal page for the whole workspace,
  because it cannot show an active organization beside a removed one.
  Reuse the take-out state, because its retention rules differ. These
  leave two repairs that show the required state within Organizations.
- **Wider fix:** B. One organization state model keeps retention text,
  available actions, and active organization counts consistent.
- **Previous examples:** `AppRemovedScreen` reads the existing
  `appRemoval` fixture for its removal date and recovery deadline.
  That fixture uses 20 November 2026 and adds 30 days for the deadline.
- **Recommendation:** B. The row can show retained history without
  treating a removed organization as active.
- **Action:** fix after merge, apply B. The original TODO covered Admin
  take-out, which `docs/protocol/workspaces.md` exempts from external
  removal retention. Preserve the other organization's jobs, machines,
  Boat account, and subscription. Count only active organizations for the
  rule that keeps the last organization. Run
  `npm --prefix ts run mockups:check` in the later change. Inspect the
  changed screen at desktop widths of 1440 and 720 pixels, and mobile
  width of 390 pixels.
