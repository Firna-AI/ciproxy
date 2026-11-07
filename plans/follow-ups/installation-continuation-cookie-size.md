# Many claim IDs break successful authorization

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-07, from `conductor/mvp-m19-ui`, PR #36, extension
  review finding 8 of commit `d557d13`. An independent check confirmed it,
  with an exact size proof.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/access/pending.rs:20` and `:46`,
  `crates/ciproxy-api/src/access/sign_in.rs:116`, `:199`, and `:218`,
  `crates/ciproxy-api/src/access/flow.rs:55` and `:66`, and
  `crates/ciproxy-api/src/handlers/auth.rs:72`.
- **Reason:** The recommended fix needs a migration, a new table, and store
  methods, which the scope that the owner approved for the field does not
  include.

- **Terms:**
  - **Claim:** a row of `installation_claims` that connects one GitHub App
    install to the person who asked for it, and later to its workspace.
  - **Continuation:** `installation_continuation`, the exact claim IDs that
    the latest successful GitHub authorization of a browser proved or
    completed.
  - **Browser cookie:** `__Host-ciproxy`, the sealed `BrowserState` sign-in
    metadata of `crates/ciproxy-api/src/access/pending.rs`.
  - **OAuth binding:** `__Host-ciproxy-oauth`, the one-use sealed `Binding`
    of `crates/ciproxy-api/src/access/flow.rs`, which each start of a GitHub
    authorization sets.
  - **Browser-only sign-in:** a sign-in whose browser cookie opens but
    matches no stored session, so it shows no workspace data.
- **Background:**
  - Short paths in this item are under `crates/ciproxy-api/src/`.
  - The continuation names the claims that one GitHub authorization proved
    or completed. Late approvals need it most: an organization owner
    approves an install request on GitHub, and the member signs in later.
  - `GET /api/session` returns it in `SessionView`, at `wire/access.rs:24`.
    The field contract says: "Exact claim IDs proved or completed by the
    latest successful GitHub authorization. All IDs refer to
    organization_links. No order selects a continuation. Reloads keep this
    list. A successful authorization replaces it, even when empty."
  - The app contract, set with the field on 7 November 2026, says that the
    app must not infer the continuation from the setup outcome, claim
    history, timestamps, roles, or a remembered workspace.
  - The return page shows an ended claim only when the continuation names
    it, in `joinsReturnPage` of `ts/app/src/state/links.ts:47`. It shows
    which organizations joined, and which joined inactive and why.
  - On 7 November 2026, the owner approved one API route or field, with its
    HTTP test on the real PostgreSQL store, the generated types, and the app
    work. That approval covers no schema, migration, or store change.
  - `crates/ciproxy-api/README.md:101` promises: "Failed proof preserves
    business records and the existing sign-in."
  - A Direct claim comes from a normal install and needs no workspace
    choice. An Add claim comes from "Add an organization", and its person
    chooses the workspace.
  - `finish`, in `access/sign_in.rs:116-124`, keeps every ID that
    `prove_links` and `prove_completed_links` return. Nothing limits the
    count. `identity/http.rs:175-232` reads all memberships and installations.
  - `prove_links`, in `access/links.rs:120-178`, adds each unlinked claim of
    the person: each signed Direct claim, and each other claim whose install
    the GitHub sign-in still sees. So an open claim comes back at each sign-in.
  - `prove_completed_links`, in `access/link_continuation.rs:14`, adds each
    completed Direct claim at its first own-user proof. It marks the claim
    `user_verified`, so a later sign-in skips it.
  - `finish` writes the session row through `establish_dashboard_session`
    at `sign_in.rs:199`, before it seals the cookie at line 218. With a
    prior cookie, the row keeps its session ID and gets a new secret digest,
    in `crates/ciproxy-store-pg/src/dashboard/sessions.rs:45-89`.
  - `browser_cookie`, in `access/pending.rs:38-50`, seals the JSON with
    AES-256-GCM, encodes it as base64url, and returns `Error::Unavailable`
    above 3,800 characters, at line 46. Then `callback` redirects to
    `/sign-in?reason=github` with no new cookie, at `handlers/auth.rs:72`.
  - `start`, in `access/flow.rs:50-66`, embeds all metadata of the current
    browser cookie in the OAuth binding, with the same limit at line 66. The
    sign-in start `GET /api/auth/github`, at `handlers/auth.rs:36`, and the
    install return `GET /api/github/setup`, at `handlers/github_setup.rs:16`,
    call `start`. Both return its error as a raw HTTP 503, with the JSON
    message "The action could not finish. Try again."
  - Before the field, `BrowserState` held only fixed-size fields. Its
    largest cookie in the size proof below has 1,222 characters.
  - A size proof sealed copies of the serde shapes of `BrowserState`,
    `Binding`, and their parts as `crypto.rs` does: a 12-byte nonce, a
    16-byte tag, the `v1.` and `oauth.` prefixes, and the crate versions of
    `Cargo.lock`. It used copies, because the crate types are private.
  - Each additional ID adds 39 JSON bytes and 52 encoded characters.
    The first ID adds 38 JSON bytes.
    Seventy-five IDs alone take 2,926 JSON bytes and 3,902 base64
    characters. With 75 IDs, the full cookie has 4,628 to 5,120 characters.
  - The limits for three kinds of metadata:

    | Metadata | Base cookie | Callback fails at | Later sign-in start fails     | Later install return fails    |
    | -------- | ----------- | ----------------- | ----------------------------- | ----------------------------- |
    | Minimal  | 730         | 60 IDs            | Kept cookie with 55 to 59 IDs | Kept cookie with 50 to 59 IDs |
    | Typical  | 872         | 57 IDs            | Kept cookie with 53 to 56 IDs | Kept cookie with 47 to 56 IDs |
    | Large    | 1,222       | 50 IDs            | Kept cookie with 46 to 49 IDs | Kept cookie with 40 to 49 IDs |

  - Minimal metadata has a 1-character login, a 1-digit GitHub ID, a
    whole-second expiry, and no install context, setup outcome, or removed
    organization. Typical metadata has a 14-character login, 9-digit IDs, a
    nanosecond expiry as `Utc::now()` gives, an attempt install context, and
    the setup outcome `waiting_approval`. Large metadata has 39-character
    logins, an organization setup outcome, and a removed organization.
  - Real logins and IDs vary inside the three kinds. Normal use does not
    reach these counts: with typical metadata, one person needs 47 or more
    claims in one authorization.
  - A callback that fails this way has already consumed the one-use login
    state and saved every claim proof. `observe_existing` saved fresh
    credentials and memberships of a known person. `resolve_link`, in
    `crates/ciproxy-github/src/app/linking.rs:13-40`, linked each ready
    claim through `handle_installation`. Each completed Direct claim is
    `user_verified`, and a stored session row has a new secret digest.
  - The browser keeps its old cookie, which opens but no longer matches the
    session row. So `authenticate`, at `access/authentication.rs:123`,
    returns a browser-only sign-in. The person loses the workspace data
    until a sign-in succeeds.
  - The failure loses no stored claim, link, workspace, role, or
    organization. It loses the exact result of the failed authorization.
  - A retry with the old cookie keeps the fixed expiry and the session ID.
    It skips the claims that the failed attempt linked or completed, so it
    succeeds when the open claims fit. Its continuation no longer names the
    claims that ended, so the return page never shows which organizations
    joined, or which joined inactive and why.
  - Sign-out clears the cookies, also for a browser-only sign-in. A new
    sign-in then works under the same condition, with a new 30-day expiry.
  - Claims that stay open include an Add claim with two or more eligible
    workspaces, and a claim whose signed GitHub event never arrived. When
    these alone pass the limit, every sign-in fails.
  - A browser-only sign-in cannot choose a workspace, at
    `access/link_choice.rs:20-22`. So the person cannot shorten the list in
    the dashboard, and the failure repeats. That blocks the rule of
    `docs/protocol/workspaces.md`, "A request that an owner approves later":
    the person who asked signs in again, then chooses the workspace.
  - A sign-in can succeed with a cookie just under the limit. Each later
    sign-in start and install return then embeds that cookie in the OAuth
    binding, which passes the limit and returns HTTP 503. Only sign-out,
    deletion of the cookie, or the fixed expiry of the sign-in stops this.
  - Nobody ran a server for the repeated failure or the HTTP 503 case. Both
    follow from the code paths above.
  - Every option keeps these rules. Results stay exact for each batch size,
    with no claim limit, as the field contract requires. The fix adds no new
    sign-in rejection, as a design call of 7 November 2026 for the field
    says. The 3,800-character limits stay.
  - Under `docs/protocol/security.md:253-254`, ciproxy keeps the record of
    a person only while the person has a role or an open request for access.
- **Problem:** The browser cookie holds every continuation ID, and nothing
  limits their number. With typical metadata, a callback with 57 IDs passes
  the 3,800-character limit, and a later install return fails from 47 kept
  IDs. The failed callback has already saved the claim proofs and replaced
  the stored session secret.
- **Impact if no action:** A person with many claims cannot finish a valid
  GitHub sign-in, can lose the prior private workspace session, and never
  sees the exact ended outcomes. When the open claims alone pass the limit,
  every later sign-in fails too.
- **Root question:** How does the dashboard keep the exact result of each
  GitHub authorization across reloads, while the sign-in metadata size never
  depends on the claim count?
- **Options:**
  - **A.** Put the continuation in separate sealed cookie chunks, and keep
    only their reference in the browser cookie. It keeps the field and
    changes only `crates/ciproxy-api`, so it probably fits the approved
    scope. But every request then carries every chunk. Browsers cap the
    number of cookies for one site, and proxies cap the request header size.
    So A fails the goal, and it adds cookie cleanup. Effort: Medium.
    - It changes `access/pending.rs`, `access/sign_in.rs`, `access/flow.rs`,
      `handlers/auth.rs`, `router.rs`, the README, and the HTTP test.
    - The repository does not hold the header limits of the deployment edge
      and its proxies. The argument against A does not depend on them.
  - **B.** Store a token-free snapshot of the continuation in a new table,
    under a random opaque reference. Put only that reference in the browser
    cookie, so the cookie and the OAuth binding keep a fixed size. Keep the
    snapshot until the fixed expiry of each sign-in, also a browser-only
    sign-in. `GET /api/session`, the generated types, and the app stay as
    they are. Cost: a migration, a new table, store methods, and expiry
    cleanup. Effort: Medium.
  - **C.** Hand the exact continuation to browser storage through a typed
    same-origin step. The browser cookie keeps only a random authorization
    ID, which `GET /api/session` returns. The app keeps the ID and the list
    in localStorage, and uses a copy only when its ID matches the session.
    C needs no store work, but it loses the list after a reload when
    browser storage is off. Effort: Large.
    - The GET callback redirects to a new static app page with the same
      code and state. The page posts them to a new finish route, which sets
      the cookies and returns the exact list as typed JSON. Without a
      matching copy, the app warns and shows every open link. That notice
      needs a mockup variant first.
    - With storage off, `webStorage` falls back to memory for one document,
      in `ts/app/src/state/storage.ts:3-5`. A reload, or the reload after an
      API build change, then loses the list. The cookie of today and B keep
      it without storage. A new tab sees the shared localStorage copy, but a
      tab that opens before the handoff ends does not. sessionStorage would
      fail new tabs.
    - The typed response of the request that proved the claims carries the
      list, so no unsigned URL ID and no claim limit take part.
    - The router sets no Content Security Policy today, only
      `Cache-Control`, at `router.rs:137`. The fetch step needs no inline
      script, only one more static page. The GitHub callback URL stays.
    - The OAuth code passes through an app URL until `history.replaceState`
      removes it. PKCE and the OAuth binding cookie keep a leaked code
      useless in another browser. C still needs a security review.
    - C replaces the field and its app consumer, so it reopens the
      late-approval journeys. It goes past the approved scope: it changes
      the callback and the field, adds a route, and needs a mockup edit.
    - API paths: `handlers/auth.rs`, `handlers/sessions.rs`, `router.rs`,
      `openapi.rs`, `wire/access.rs`, `access/pending.rs`, `access/sign_in.rs`,
      `access/mod.rs`, `access/authentication.rs`, and the API README.
    - Test paths in `crates/ciproxy-api/tests/http/`: the `callback` helper
      at `fixture_io.rs:20`, the HTTP tests that sign in through it, and
      `late_approval_tests.rs`.
    - Other paths: `ts/api/generated/`, and in `ts/app/` a new sign-in finish
      page, `src/state/storage.ts`, `src/routing/claim-continuation.ts`,
      `src/routing/landing.ts`, `src/state/links.ts`, `scripts/harness-*.ts`,
      and `README.md`. The new notice gets a variant under `docs/mockups/`.
- **Left out:**
  - Compact IDs, packed IDs, or compression: they postpone the failure but
    keep a limit that depends on the claim count. Packed 16-byte IDs fit
    only about 90 to 107 IDs.
  - Drop the IDs past a limit: it loses exact results and adds a silent
    claim limit.
  - Raise the cookie limit: browsers keep about 4,096 bytes for one cookie.
  - Check the size before the session write: it prevents the partial
    changes, but it still refuses a valid sign-in, a new sign-in rejection.
  - An overflow marker that shows every open link: it warns and continues,
    but it breaks the exact results for large batches.
  - A separate HTML handoff option: it is a variant of C with the same
    disabled-storage limit. It also couples the API to the app storage
    format. C includes this browser-storage approach.
  - IDs in a URL query or fragment: unsigned URL IDs are not proof, and the
    API README promises outcomes "without frontend installation-ID
    forwarding". A sealed URL value still grows with the count.
  - A marker or an "outcome not shown" column on `installation_claims`: two
    browsers of one person overwrite one marker, and later proofs move it.
    It changes the "latest authorization" contract. B is better on every point.
  - `dashboard_login_states` as a handoff store: it misuses the one-use
    OAuth state contract, and still needs the browser storage and route of C.
  - Equal `verified_at` times as a batch tag, or a fix in the app alone: the
    app must not infer the continuation from order or time, and claim
    states name no authorization. PostgreSQL also keeps only microseconds.
  - Change nothing: the High harm stays.
- **Wider fix:** B. The cookie then holds one fixed-size reference, so no
  claim count changes its size. The OAuth binding embeds that cookie, so B
  also bounds each later sign-in start and install return.
- **Previous examples:** None found in the searched history or plans.
  `pending_browser_state_retains_no_person_and_promotes_without_extending_expiry`,
  in `crates/ciproxy-api/tests/http/membership_tests.rs:59`, checks the
  cookie size only with a small result.
- **Recommendation:** B. It alone keeps exact results across reloads in every
  browser with a fixed-size cookie, and changes no wire field or app code.
- **Action:** ask the user, because the owner must approve the scope of B
  first. B needs a migration, a new table, and store methods. The approval
  of 7 November 2026 covers only one API route or field, and no option
  inside that scope meets the goal.
  - Apply no option until the owner answers. This open item does not accept
    the failure, and it does not defer the fix.
  - While the item is open, it blocks work that depends on the sign-in
    finish. It also blocks the final merge of the MVP branch
    `calummoore/ci-proxy-load-balancer` into `main`.
  - When the owner approves B, apply it at once, because the severity is
    High and B adds a migration.
  - Add a migration with `up.sql` and `down.sql` under
    `crates/ciproxy-schema/migrations/`, and the table in
    `crates/ciproxy-schema/src/dashboard.rs`. The table holds only the
    opaque reference, the claim IDs, and the fixed expiry. It has no person
    column, token, secret, or GitHub credential, and it grants no access.
    Make no person row for it, so the retention rule above holds.
  - Add a save method and a read method to
    `crates/ciproxy-store-interface/src/dashboard/store.rs`, and a record
    type in `records.rs` only when needed. The read takes the reference and
    the current time, and ignores an expired snapshot. Implement both in
    `crates/ciproxy-store-pg/src/dashboard/mod.rs` and `sessions.rs`.
  - Delete expired snapshots in `expire_dashboard_auth`, which
    `maintenance.rs:83` and `access/authentication.rs:173` call.
  - Make a new random reference for each successful authorization. Give the
    snapshot the fixed expiry of the sign-in: the prior expiry for a
    matching prior cookie, else 30 days from now.
  - In `finish`, save the snapshot before `establish_dashboard_session`.
    A store failure then stops the sign-in before the secret changes.
  - Replace the field at `access/pending.rs:20` with the reference, and
    change `Principal` in `access/mod.rs` to match.
  - In `session` of `handlers/sessions.rs`, read the snapshot by its
    reference, also for a browser-only sign-in. Keep the filter at lines
    125-129, which returns only IDs of current requester-owned claims. That
    filter scopes the IDs to the person without a person column.
  - Keep the wire field, `ts/api/generated/`, and `ts/app/` unchanged. The
    `api-types` gate phase must show no difference.
  - An old cookie with the list field needs no migration. ciproxy is not in
    production, and serde ignores the unknown field.
  - Update `crates/ciproxy-api/README.md:95-102`, which says that the sealed
    browser metadata keeps the list. Add the snapshots to the dashboard
    sentences of the READMEs of `crates/ciproxy-schema`,
    `crates/ciproxy-store-interface`, and `crates/ciproxy-store-pg`.
  - Add one HTTP regression on the real PostgreSQL store to
    `crates/ciproxy-api/tests/http/late_approval_tests.rs`. Sign in with 75
    valid claims, and expect a successful callback and all 75 IDs from
    `GET /api/session`, also on a second read. Then run a later authorization
    with the kept cookie, and expect success and a replaced list. That step
    covers the OAuth binding path, so the test needs no install-return case.
  - Change no protocol doc, because the behaviour at the boundary stays.
