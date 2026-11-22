# Two GitHub sign-in live probes remain

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the dashboard API milestone of
  `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's request.
- **Location:** `crates/ciproxy-api/scripts/README.md`, the sections
  "Repeated GitHub authorization" and "GitHub installation binding".
- **Reason:** Both probes need the browser and the GitHub accounts of the
  user.

- **Terms:**
  - **Repeated authorization:** a second GitHub sign-in of a user whose
    session already exists, for example after an App install.
  - **Install binding:** the flow that links a new App installation to the
    workspace of the person who started it.
- **Background:**
  - The live API smoke passed on the main preview on 8 and 16 November
    2026.
  - The one-tab repeated authorization passed on 17 November 2026. GitHub
    showed no consent page. The session moved from access ended to ready
    with both workspaces, and the original expiry stayed.
  - The two-tab variant did not run. In it, one callback tab closes before
    it receives its response.
  - Since 22 November 2026, the `reauthorization_url` of the session is the
    terms step at `/continue`. So Check again on a page whose sign-in has
    no workspace opens that page first. Its "Continue with GitHub" button
    starts the GitHub sign-in. The one-tab run above came before this step.
  - The install-binding probes did not run. They need a GitHub App user
    token of the person who starts the install and a fresh App JWT. The
    private credential file names them `github_user_token`,
    `github_app_jwt`, `github_app_slug`, `target_organization_id`, and
    `installation_proof_file`.
- **Task:**
  1. Run the two-tab variant as the "Repeated GitHub authorization"
     section says. Where the probe follows the `reauthorization_url` or
     selects Check again, select Continue with GitHub on the terms step.
     Close one callback tab before it receives its response. Check again
     from the other tab. Confirm that the original 30-day expiry stays in
     effect.
  2. Run the install-binding probe once as an organization owner. Then
     run it as a member whose install needs the approval of an owner.
  3. Record the results, the build identity, and pass or fail under
     `.context`. Record no cookie, OAuth code, state, grant, or install
     command.
- **Impact if no action:** A sign-in race or a wrong install binding stays
  unproven against the real GitHub flow.
- **Needs:** The user's browser with the test App authorized. A member
  account of a test organization for the approval case.
- **Done when:** Both probes pass on the main preview or on production.
