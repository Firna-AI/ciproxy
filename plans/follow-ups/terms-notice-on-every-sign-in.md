# Some sign-ins skip the terms notice

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, review
  finding 2.
- **Location:** `ts/site/src/data/legal-terms.ts`, the acceptance sentence
  of "The service". `ts/app/src/screens/sign-in.tsx`, the footer.
  `crates/ciproxy-api/src/handlers/github_setup.rs`, near line 36.
  `ts/app/src/view/access.ts`, near lines 85 and 252.
  `ts/app/src/view/add-organization.ts`, near line 118.
- **Reason:** The fix adds a step to the GitHub setup path and to the
  sign-in links inside the dashboard, and only the user can choose that.

- **Terms:**
  - **Terms notice:** the footer of the dashboard sign-in page. It says "By
    signing in, you agree to the ciproxy Terms and Privacy policy".
  - **Setup return:** the dashboard address that GitHub opens after a person
    installs the ciproxy GitHub App on GitHub.
- **Background:**
  - The terms of service say "You accept their current version each time
    that you sign in with GitHub". The user chose this rule from the
    conductive terms on 22 November 2026.
  - The sign-in page shows the terms notice above its button.
  - The setup return sends the browser straight to GitHub authorization.
    The callback then gives a dashboard sign-in with no terms notice.
  - The install page, the add-organization page, and "Check again" also
    start a GitHub sign-in with no terms notice. They keep the current
    sign-in and its end time.
- **Problem:** A person who arrives through the setup return can get a
  dashboard sign-in without seeing the terms notice. The terms still say
  that this sign-in accepts the terms.
- **Impact if no action:** Firna Ltd can treat a sign-in as acceptance of
  the terms when the person never saw them.
- **Root question:** How does each person see the current terms before the
  step that accepts them, on every path that starts a dashboard sign-in?
- **Options:**
  - **A.** Limit the acceptance sentence to the sign-in page that shows the
    notice. A setup sign-in then accepts nothing. Effort: Small.
  - **B.** Send the setup return and the sign-in links inside the dashboard
    through the same notice and a Continue button. Keep the pending setup in
    the sealed server binding. Effort: Medium.
  - **C.** Record the terms version that each person accepts at one common
    step before the dashboard, and show the notice when the version is old.
    This also proves which version a person accepted. Effort: Large.
  - **D.** Add a sentence to the terms: a GitHub sign-in inside a running
    30-day sign-in starts no new sign-in and accepts no version. Use it with
    B. Effort: Small.
  - **E.** Put the notice in the footer of every dashboard page, and say that
    a person accepts by continued use. A passive link gives weak evidence of
    acceptance. Effort: Small.
  - **F.** Store the accepted terms version in the sealed sign-in cookie. The
    first dashboard page shows a one-time notice with Continue when the
    version is old. No database record proves the acceptance. Effort:
    Medium.
- **Left out:**
  - Acceptance once for each workspace breaks the rule that the user chose:
    acceptance at each sign-in.
  - A notice at checkout only fails the goal, because a Free workspace never
    reaches checkout.
  - A notice at the first install token or Boat connection fails the goal,
    because a Viewer never sees it.
- **Wider fix:** C. It checks the accepted version at one common step, so a
  new sign-in path cannot skip the notice. F is a lighter form.
- **Previous examples:** None found.
- **Related:**
  - `terms-changes-unattended-workspaces.md`: the same acceptance rule. The
    record of C also gives that item proof of the version that binds a
    workspace.
  - `terms-acceptance-parties.md`: the same sentence. That item decides
    whose acceptance binds the customer, which no option here decides.
    Answer both together.
  - `site-and-app-publication.md` makes the App public. The fix should land
    before that step.
- **Recommendation:** B, with the sign-in links inside the dashboard, or with
  D. It keeps the acceptance rule that the user chose and changes only the
  paths that skip the notice. C best protects the codebase.
- **Action:** ask the user, because B adds a required step to the GitHub
  setup path and the sign-in links, and no requirement covers that step.
