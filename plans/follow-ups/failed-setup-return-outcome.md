# A failed setup sign-in may lose its outcome

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-07, from `conductor/mvp-m19-ui`, pull request #36. The
  check of finding 1 of the second owner review, which the branch review
  numbers 11, noted it at `d304418` from a source trace, outside the scope.
- **Location:** `crates/ciproxy-api/src/access/sign_in.rs:172`, the
  statement `let setup = setup.or_else(...)` in `finish`,
  `crates/ciproxy-api/src/handlers/auth.rs:56-73`, in `callback`, and
  `crates/ciproxy-api/src/access/authentication.rs:78-80`, in
  `begin_login`.
- **Reason:** Only a read of the code suggests the loss, nobody reproduced
  it, and the base branch `calummoore/ci-proxy-load-balancer` has the same
  API behaviour.

- **Terms:**
  - **Setup return:** GitHub sends the browser to `/api/github/setup` after
    an install or an install request. Since 22 November 2026, ciproxy then
    keeps the pending setup and opens the terms step.
  - **Terms step:** the dashboard page at `/continue`, with the terms
    notice and a "Continue with GitHub" button.
  - **Pending setup:** the setup that a setup return keeps, sealed in the
    one-use OAuth cookie `__Host-ciproxy-oauth` for 10 minutes.
  - **Setup sign-in:** the GitHub sign-in with the setup purpose. It proves
    the install with the person's own GitHub account. Until 22 November
    2026, the setup return started it. Now Continue with GitHub starts it
    from a valid pending setup.
  - **Setup outcome:** the `setup` field of `GET /session`: `organization`,
    `organizations_only` for a personal account, or `waiting_approval`.
    `SetupOutcome` in `crates/ciproxy-api/src/wire/access.rs` defines it.
  - **Browser cookie:** the sealed `__Host-ciproxy` cookie. Its state,
    `BrowserState` in `crates/ciproxy-api/src/access/pending.rs`, holds the
    setup outcome and the install context.
- **Background:**
  - Line numbers are from `d304418`.
  - At `d304418`, `begin` in `crates/ciproxy-api/src/handlers/github_setup.rs`
    started a sign-in with a setup purpose. Its sealed binding kept the
    earlier browser cookie, at `crates/ciproxy-api/src/access/flow.rs:50-60`.
  - Since 22 November 2026, the setup return starts no sign-in. `begin` in
    `github_setup.rs` checks its input as before. Then it keeps the pending
    setup in the OAuth cookie, through `hold_setup`, and opens `/continue`.
    Continue with GitHub on that page opens
    `/api/auth/github?resume=setup`. With a valid pending setup, `begin` in
    `crates/ciproxy-api/src/handlers/auth.rs` and `begin_login` in
    `crates/ciproxy-api/src/access/authentication.rs` start the setup
    sign-in. Its sealed binding still keeps the earlier browser cookie.
    Without a valid pending setup, Continue starts a plain sign-in.
  - The binding of the setup sign-in takes the place of the pending setup
    in the same cookie. A plain `/api/auth/github` also replaces a pending
    setup. So a read of the code shows that no pending setup is left after a
    failed or cancelled setup sign-in. Each retry then starts a plain
    sign-in: from the sign-in page, from a second Continue with GitHub, and
    from a link inside the dashboard, which now opens the terms step first.
  - `finish` proves the outcome with `prove_setup`, at
    `crates/ciproxy-api/src/access/sign_in.rs:55-78`. `prove_setup`, at
    `crates/ciproxy-api/src/access/setup.rs:46-84`, only reads GitHub. It
    writes nothing.
  - The outcome then goes only into the new browser cookie, at
    `sign_in.rs:204-224`. No store write keeps it.
  - Each later step of `finish` can fail: store reads and writes, GitHub
    reads, the credential and cookie seals, and the session write. The
    callback then sends the person to `/sign-in?reason=github` and sets no
    cookie, at `crates/ciproxy-api/src/handlers/auth.rs:66-73`.
  - A cancelled GitHub authorization and a missing or expired binding take
    the same redirect, at `auth.rs:56-65`.
  - The sign-in page links to `/api/auth/github`, through
    `githubSignInHref` in `ts/app/src/api/endpoints.ts:15`. That route
    starts a plain sign-in, at
    `crates/ciproxy-api/src/access/authentication.rs:78-80`.
  - A plain sign-in proves no setup outcome. At `sign_in.rs:172`, it takes
    the outcome of the earlier browser cookie, when that cookie belongs to
    the same GitHub person and has not expired.
  - `GET /session` returns the outcome of the cookie, at
    `crates/ciproxy-api/src/handlers/sessions.rs:134`.
  - The app reads it. `organizations_only` opens `/personal-account`, at
    `ts/app/src/routing/landing.ts:105-110`. `waiting_approval` opens the
    return page when this tab started the install and requests wait, at
    `landing.ts:131-137`. `organization` names the new installation, at
    `ts/app/src/routing/continuation.ts:43-47`.
  - `docs/protocol/workspaces.md:79-84` says that the page after a personal
    account install says that ciproxy works with organizations only.
  - `crates/ciproxy-api/README.md` says: "Failed proof preserves business
    records and the existing sign-in." The code keeps the existing
    sign-in.
  - On 7 November 2026, the owner chose to keep the claim results of a
    failed sign-in in the store until a successful sign-in consumes them.
    The `installation_continuation` paragraph of
    `crates/ciproxy-api/README.md` gives that rule. Check whether that
    change also reaches `setup`.
  - The base branch has the same plain retry, the same cookie field, and
    the same fallback. The app pages that read the outcome are new in the
    branch that noted this.
  - Limits: this note comes from a read of the code. Nobody ran it. The
    trace did not check how often a retry follows a setup return. It did
    not check whether the `installStarted` mark of the tab survives the
    retry, or how far the claim list covers an `organization` outcome.
  - The install context at `sign_in.rs:167-171` uses the same fallback.
    Nobody traced its effect.
- **Task:** Reproduce the path with the real PostgreSQL store, and record
  what the retry shows.
  - Start from the personal setup return of
    `fresh_personal_setup_records_only_verified_token_free_metadata` in
    `crates/ciproxy-api/tests/http/setup_personal_tests.rs`. Its helper
    `complete_setup`, in `crates/ciproxy-api/tests/http/setup_support.rs`,
    runs the setup return, keeps the pending setup cookie, and starts the
    setup sign-in through `/auth/github?resume=setup`, as Continue with
    GitHub does. The helpers `begin`, `pending_cookie`,
    `start_authorization`, and `finish` of the same file run these steps
    one at a time.
  - Fail the cookie seal with `FailingBrowserCipher` of
    `crates/ciproxy-api/tests/http/session_failure_tests.rs`. Turn the
    failure on after the setup sign-in starts, so that only its callback
    fails. Sign in again through `/auth/github`, and read `setup` of
    `GET /session`.
  - Also retry through `/auth/github?resume=setup` with the OAuth cookie
    that the failed sign-in left, as a second Continue with GitHub does.
  - Repeat with an earlier outcome in the browser cookie, and with a
    cancelled GitHub authorization of the setup sign-in.
  - When the retry shows the outcome of the return, delete this item.
  - When it does not, replace this item with a finding item that holds the
    result. The decision test of the `decisions-and-findings` skill then
    says whether it waits for the user.
- **Impact if no action:** If the loss is real, a person who installed on a
  personal account can miss the page that explains it after a rare failure
  or a cancelled authorization. An older outcome of the same 30-day sign-in
  can also open its old page.
- **Needs:** A local PostgreSQL 15 database with `DATABASE_URL` set, as
  `AGENTS.md` describes.
- **Done when:** A reproduction shows what `setup` of `GET /session` holds
  after a failed or cancelled setup sign-in and a retry. This item is then
  deleted, or a finding item with that result replaces it.
