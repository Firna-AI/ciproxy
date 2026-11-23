# Continue can resume the wrong install

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  1, finding 1.
- **Location:** `crates/ciproxy-api/src/handlers/github_setup.rs:32`, where
  the setup return keeps its pending setup.
  `crates/ciproxy-api/src/handlers/auth.rs:51`, where Continue reads it.
  `crates/ciproxy-api/src/access/flow.rs:47` and `:60`, which seal and open
  it. `ts/app/src/view/access.ts:50`, the terms step page.
- **Reason:** Only two installs in one browser at about the same time meet
  the bug, and its fix costs the same after the merge.

- **Terms:**
  - **Setup return:** the route `/api/github/setup` that GitHub opens after
    a person installs the ciproxy GitHub App, or asks an owner for the
    install.
  - **Terms step:** the dashboard page at `/continue`, with the terms
    notice and a "Continue with GitHub" button.
  - **OAuth cookie:** the sealed cookie `__Host-ciproxy-oauth`. It lives for
    at most 10 minutes. It holds a pending setup or the one-use binding of a
    running GitHub sign-in.
  - **Pending setup:** the setup details that a setup return keeps in the
    OAuth cookie until the person selects Continue with GitHub.
  - **Claim:** the record of one App install that waits to link to a
    workspace. It names the person who started the install.
- **Background:**
  - ciproxy supports two adds at once. An add is an install that an Admin
    starts with "Add an organization". A person can run two adds in two
    tabs of one browser.
  - Each setup return checks its input, keeps a pending setup in the OAuth
    cookie, and opens the terms step. `begin` in
    `crates/ciproxy-api/src/handlers/github_setup.rs` does this.
  - The terms step is a static page. It reads nothing, so it names no
    install. `termsStepPage` in `ts/app/src/view/access.ts` builds it.
  - Continue with GitHub opens `/api/auth/github?resume=setup`. With a
    valid pending setup in the OAuth cookie, that route starts the GitHub
    sign-in for that setup. Otherwise it starts a plain sign-in. `begin` in
    `crates/ciproxy-api/src/handlers/auth.rs` and `begin_login` in
    `crates/ciproxy-api/src/access/authentication.rs` do this.
  - A pending setup is valid for 10 minutes.
  - The browser holds one OAuth cookie. So a second setup return writes
    over the pending setup of the first.
  - Check again and the other GitHub sign-in links inside the dashboard
    also open the terms step. Their Continue reads the same cookie.
  - A plain sign-in from `/api/auth/github` also replaces a pending setup.
  - Each successful sign-in also proves the other open claims of the
    person, at `crates/ciproxy-api/src/access/links.rs:122`. The test at
    `crates/ciproxy-api/tests/http/link_recovery_tests.rs:80` shows this
    for an add whose return was lost.
  - `binding_cookie` in `crates/ciproxy-api/src/access/flow.rs` refuses a
    cookie value longer than 3800 characters.
  - The privacy policy names three dashboard cookies. The third "protects
    the GitHub sign-in step and keeps the install details that GitHub
    returns, for up to 10 minutes each time that it is set".
    `ts/site/src/data/legal-privacy.ts` holds this text.
  - On 22 November 2026, the user delegated five legal decisions to the
    orchestrator agent. The agent then made these calls. They are agent
    calls, not user decisions.
    - Every GitHub sign-in that does not start on the sign-in page passes
      the terms notice first. The pending setup stays in the sealed binding
      that the server writes.
    - That binding is the one-use OAuth cookie. The setup return keeps the
      pending setup there for at most 10 minutes, so the cookie list of the
      privacy policy stays true.
    - The terms step is a static page. The export renders it in full
      without its query, so nothing changes after it loads.
    - Only the Continue of the terms step resumes a setup. So a later plain
      sign-in in the same browser never takes over the setup of another
      person.
- **Problem:** Two setup returns in one browser share the one pending setup
  of the OAuth cookie. When the second return comes before the first tab
  selects Continue with GitHub, that Continue resumes the second install.
- **Impact if no action:** The first tab shows the person's own verified
  result for the second install, which can be another organization. The
  first install loses its pending setup, but a later sign-in still links
  it.
- **Root question:** How does each Continue resume the install that opened
  its page?
- **Options:**
  - **A.** Match each page to its pending setup before Continue resumes it.
    When another return replaced the setup, use the existing fallback to a
    plain sign-in. This stops the swap, but the displaced setup is still
    lost. Each page must read the name of its setup, which reverses the
    call that the terms step is a static page. Effort: Medium.
  - **B.** Give each setup return a separate, browser-bound record. Make
    Continue use only the record of its page. Keep the 10-minute expiry,
    clear the records on a plain sign-in, and add a regression test with
    two returns. As in A, each page reads the name of its record. The
    records need a place. A fourth cookie changes the cookie sentence of the
    privacy policy. A server record of each setup return needs a migration.
    Records inside the OAuth cookie must also travel through the binding of
    a running sign-in. So B is Medium only with the records inside the OAuth
    cookie. Effort: Medium.
  - **C.** Give every sign-in its own browser-bound record, from the terms
    notice through the return from GitHub. This also keeps apart sign-ins
    that overlap after Continue, but it reaches more code. As in A, each
    page reads the name of its record, and a plain sign-in clears the
    records. Effort: Large.
  - **D.** Keep a short list of pending setups in the one sealed cookie.
    The first Continue proves all of them, and the session keeps a list of
    their outcomes. No page needs the name of its setup, so the terms step
    stays static. Effort: Large.
- **Left out:**
  - Keep the newest setup as the default: it lets a person continue an
    install that did not open the page.
  - A server record for each install attempt: an install that starts on
    GitHub has no ciproxy state.
  - A terms notice before the install: an install can start on GitHub, with
    no ciproxy page before it.
  - A terms step that the API renders: the API renders no HTML page, and
    the terms notice would get a second copy outside the dashboard.
  - The `Referer` header as the carrier of the setup of a page: a browser
    setting or an extension can leave the header out.
- **Wider fix:** B. Separate records remove the shared slot that causes the
  swap.
- **Previous examples:**
  - The test at `crates/ciproxy-api/tests/http/link_flow_tests.rs:128`
    gives two adds distinct identities.
  - The test at `crates/ciproxy-api/tests/http/oauth_tests.rs:22` checks
    that a sign-in uses its matching browser record.
- **Related:** `plans/follow-ups/failed-setup-return-outcome.md`, the same
  code: the pending setup in the OAuth cookie and the sign-in that resumes
  it. A fix of this item can change the steps of its reproduction.
- **Recommendation:** B. It keeps both pending setups, and the fix stays
  within the new step.
- **Action:** fix after merge, apply B, because a Medium product bug that
  is Rare waits when its fix does not get harder after the merge, and B
  first needs a design choice of where the separate records live.
