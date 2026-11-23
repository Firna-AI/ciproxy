# Take the group App ID from GitHub

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from pull request #142
  (`conductor/ciproxy-runner-group-per-environment`), the wider fix of
  review finding 7.
- **Location:** `crates/ciproxy-github/src/installation_state.rs:44`,
  `crates/ciproxy-github/src/app/reconcile.rs:85`,
  `crates/ciproxy-control-plane/src/composition.rs:176`.
- **Reason:** The fix parses a field of the GitHub response that ciproxy
  does not read today, and a response without it then fails. That is
  stricter validation, which the user decides.

- **Terms:**
  - **Managed runner group:** the private GitHub runner group that one
    environment keeps in each organization. Its name is
    `ciproxy-<app id>-<organization id>`.
  - **Installation record:** the response of GitHub's App-JWT
    installation endpoints, which `AppInstallationClient` in
    `crates/ciproxy-github/src/installation_state.rs` reads into
    `InstallationResponse`.
- **Background:**
  - The App ID in the group name comes from the control-plane setting
    `GITHUB_APP_ID`. The composition root in
    `crates/ciproxy-control-plane/src/composition.rs` passes it to
    `AppService::app_id`. The same setting is the issuer of the App JWT.
  - Pull request #142 gave the App ID its own string type, so another
    `String` setting, such as the OAuth client ID, no longer compiles in
    its place. A literal, or a deliberate wrap of another value, still
    compiles. No test reaches the composition root with a group request.
  - GitHub defines the JWT `iss` as "The client ID or application ID of
    your GitHub App" and recommends the client ID. So working App
    authentication does not prove that `GITHUB_APP_ID` holds the App ID.
  - GitHub's REST reference marks `app_id` of an installation as a
    required integer. Every `prepare` call already follows a read of the
    installation record under the organization lease.
- **Problem:** The group name depends on a setting that nothing checks
  against the App that GitHub authenticated. If the setting moves to the
  client ID, every managed name changes. Groups with a saved ID get the
  new name, but a lookup by name after a purge misses the old group.
- **Impact if no action:** A wrong or changed `GITHUB_APP_ID` gives an
  environment a different group name. Without a saved group ID, the
  environment then creates a second group and leaves the first one.
- **Root question:** How does the group name stay tied to the App that
  GitHub authenticated?
- **Options:**
  - **A.** Keep the typed setting of #142. Effort: Small.
  - **B.** Read `app_id` from the installation record, pass it to
    `AppService::prepare`, and remove `AppService::app_id` and its
    wiring. `InstalledApp` gets a new field, with eight builders in
    three crates, two of them in `ciproxy-api` production code, and the
    test servers' installation JSON. A response without `app_id` then
    fails. Update `crates/ciproxy-github/scale-sets.md`, which names
    `GITHUB_APP_ID` as the source. Effort: Medium.
  - **C.** Give authentication and group naming one typed App identity.
    This changes the signer. If that identity is the JWT issuer, a later
    move of the issuer to the client ID changes every managed name.
    Effort: Medium.
- **Left out:** A startup check of `GITHUB_APP_ID` against GitHub's
  `GET /app`, and a numeric parse of the setting. Each checks the setting,
  not the value that reaches the App service, and each adds a startup
  failure. A test of the composition root needs a new GitHub base-URL
  input, because `composition.rs` fixes `https://api.github.com/`.
- **Wider fix:** B. No change of the wiring, the deploy mapping, the
  setting, or the JWT issuer can then change the name.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/shared-saved-runner-group.md` option D
  would use the managed name as an owner marker, which makes a correct App
  ID in the name more important.
- **Recommendation:** B. It ties the name to the authenticated App at
  Medium effort. A is enough while the setting holds the App ID.
- **Action:** ask the user, because B rejects an installation record
  without `app_id`, which is stricter validation.
