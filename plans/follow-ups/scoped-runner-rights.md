# Let a role add runners for chosen organizations

- **Severity:** Low. **Effort:** Medium. **Confidence:** Medium. **Harder
  later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from the branch
  `calummoore/ci-proxy-load-balancer`, the user's answer on who may add a
  machine to a workspace with several organizations.
- **Location:** `docs/protocol/security.md`, "Roles".
- **Reason:** On 4 November 2026 the user chose the Admin role alone for
  now, and named this narrower right as "later, not now".

- **Terms:**
  - **Workspace:** the ciproxy account of one customer. It holds one or more
    GitHub organizations, and owns the machines and the Boat account.
  - **Admin:** the ciproxy role that manages everything in a workspace,
    including the machines and the Boat account.
- **Background:**
  - A machine and the Boat account of a workspace run the jobs of every
    organization of the workspace, with their secrets.
  - On 4 November 2026 the user decided that the Admin role alone adds a
    machine and connects Boat. ciproxy checks no GitHub right for it.
  - The grant of the Admin role is the one step of trust, and its dialog
    says that it covers the runners of every organization.
  - An organization that wants other people to control its runners uses a
    workspace of its own.
  - The user said: "later, not now, we could even allow them to specify
    which orgs or repo they can add runners for".
- **Problem:** The Admin role gives control of the runners of every
  organization, also to a person who should control only some of them.
- **Impact if no action:** A customer with several organizations must
  trust each Admin with all of them, or split them into workspaces.
- **Root question:** How does a workspace let a person control the runners
  of some of its organizations or repositories only?
- **Options:**
  - **A.** Keep the Admin role alone. Effort: Small.
  - **B.** A narrower right on a role: the organizations whose jobs the
    machines of that person may run. Effort: Medium.
  - **C.** B, down to repositories. Effort: Large.
- **Left out:** The GitHub runner right as the check, because the user found
  that rule too complex to understand and manage.
- **Wider fix:** None, because this is one new feature.
- **Previous examples:** The rule of the GitHub runner right, which the user
  reversed on 4 November 2026.
- **Recommendation:** B, when a customer asks for it. Machines then need a
  record of the organizations that they serve, which the scaler reads.
- **Action:** ask the user, because the user named this as later work, and
  it adds a rule.
