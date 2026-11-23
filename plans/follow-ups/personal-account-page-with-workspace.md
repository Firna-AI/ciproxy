# Personal-account page skips people with a workspace

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review fix
  round 1, a gap that the fix of finding 6 found.
- **Location:** `docs/protocol/workspaces.md`, "Add an organization", the
  bullets from line 90 to line 100. `ts/app/src/routing/landing.ts:80`,
  `freshOutcome`, and `:107`, the step of `firstLanding` that opens the
  page. `ts/app/src/screens/state-pages.tsx:60`, `PersonalAccountScreen`.
  `ts/app/src/view/access.ts:139`, `personalAccountPage`.
  `docs/mockups/src/entries/app/github-app/states.mockup.tsx:20`, the
  mockup entry of the page.
- **Reason:** The fix decides which side of a requirement and code
  difference is right, and the gap is older than the branch that found it.

- **Terms:**
  - **Setup sign-in:** the GitHub sign-in that Continue with GitHub starts
    on the terms step at `/continue`, at most 10 minutes after GitHub opens
    the setup return. It proves the install with the person's own GitHub
    account.
  - **Setup outcome:** the `setup` field of `GET /api/session`. The value
    `organizations_only` means an install on the person's own personal
    account. The session keeps the outcome for the whole 30-day sign-in.
  - **Landing:** the dashboard page at `/` that opens the next page from
    the session. `ts/app/src/routing/landing.ts` holds its rules.
  - **Install mark:** a value in the storage of one browser tab. The tab
    sets it when it follows an install link of the dashboard to GitHub, in
    `ts/app/src/services/services.ts`.
- **Background:**
  - "Add an organization" in `docs/protocol/workspaces.md` says: "When a
    person installs the app on a personal account, ciproxy makes no
    workspace for it."
  - The same section then says: "After that sign-in, the dashboard shows a
    page that says that ciproxy works with organizations only." "That
    sign-in" is the setup sign-in.
  - Its decision line says: "a personal account gets this page and no
    workspace."
  - Before 22 November 2026, the section said: "The page that GitHub then
    returns to says that ciproxy works with organizations only." That
    sentence had the same gap, so the gap is older than the branch that
    found it.
  - The API gives the outcome `organizations_only` after the setup sign-in
    of any install on the person's own personal account. A role in a
    workspace does not change it. `prove_setup` in
    `crates/ciproxy-api/src/access/setup.rs` sets it.
  - The test `fresh_personal_setup_records_only_verified_token_free_metadata`
    in `crates/ciproxy-api/tests/http/setup_personal_tests.rs` shows this
    outcome for a person with the Admin role.
  - The landing opens `/personal-account` for this outcome only when
    `freshOutcome` holds. The session state must not be `ready`, or the tab
    must carry the install mark.
  - The comment of `freshOutcome` says: "A person without a workspace has
    nothing else to see, so it always counts. A person with a workspace
    sees it only on the return of an install that this tab started."
  - A person with a role in a workspace has the session state `ready`. So
    when the install did not start in the same tab, the landing opens the
    workspace. An example is an install from the page of the App on GitHub.
  - On 22 November 2026, a run of `sessionLanding` with the fixtures of
    `ts/app/scripts/fixtures-core.ts` gave these pages for the outcome
    `organizations_only`:
    - No workspace and no install mark: `/personal-account`.
    - A workspace and no install mark: the workspace.
    - A workspace and the install mark: `/personal-account`.
  - The personal-account page has no link to a workspace. The comment of
    `ts/app/src/screens/state-pages.tsx` says that its pages show "for a
    session without a workspace".
  - The mockup entry `personalAccount` describes the page for one person
    with no word about a workspace.
  - The landing already keeps the last handled installation in the
    browser, so that an organization install opens its next page once.
    `ts/app/src/routing/continuation.ts` holds that rule.
- **Problem:** The requirement says that the page follows the setup sign-in
  of each install on a personal account. The dashboard skips the page for a
  person with a workspace, unless the same tab started the install.
- **Impact if no action:** A reviewer cannot tell from the requirement which
  page such a person gets. A person with a workspace who installs on a
  personal account straight from GitHub gets no word that the install made
  no workspace, and the App stays on the personal account.
- **Root question:** What does a person with a workspace see after they
  install the App on a personal account?
- **Options:**
  - **A.** Change nothing. The requirement keeps a case that the dashboard
    does not meet. Effort: Small.
  - **B.** Change the requirement to match the dashboard. "Add an
    organization" then says that the page follows the setup sign-in for a
    person without a workspace, and for an install that started in the same
    tab. In each other case, the dashboard opens the workspace. Add a
    decision line: the session keeps the outcome for the whole sign-in, so
    the page must not take the place of the workspace at each visit.
    Effort: Small.
  - **C.** Change the dashboard to match the requirement. Show the page
    once after each such setup sign-in, also for a person with a workspace.
    Remember it in the browser, as the landing does for an organization
    install. The page then needs a way to the workspace, so its mockup needs
    a new state first. Add a landing test for each case. Effort: Medium.
  - **D.** Open the workspace, and show a notice there once. The notice says
    that the install on the personal account made no workspace, and it
    links to the GitHub settings that remove the App. This changes the
    requirement and the dashboard, and it needs a new mockup. Effort:
    Medium.
- **Left out:** Show the page at each visit of a person with a workspace:
  the session keeps the outcome for the whole 30-day sign-in, so the page
  would keep the person away from the workspace.
- **Wider fix:** None, because the gap is one rule of one page.
- **Previous examples:** `plans/follow-ups/mixed-install-access-recovery.md`
  records another landing rule that hides what a requirement asks the
  dashboard to show.
- **Related:** `plans/follow-ups/failed-setup-return-outcome.md`. It also
  concerns which outcome page a setup return shows: it asks whether a
  failed or cancelled setup sign-in loses its outcome.
- **Recommendation:** B. The dashboard rule has a stated reason, and B makes
  the requirement say what each person sees, at Small effort. C protects the
  person best, because each install on a personal account then gets its
  explanation, but it needs a new mockup state first.
- **Action:** ask the user, because the fix decides which side of a
  requirement and code difference is right.
