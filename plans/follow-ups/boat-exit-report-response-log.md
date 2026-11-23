# Guest logs acceptance for a non-2xx answer

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-21, from `conductor/boat-callback-diagnosis`, a
  problem that the check of review finding 9, round 2, found outside the
  scope of that docs-only diff.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-guest/src/boat.rs:55-60` and `:81-95`,
  `BoatChannel::send`, and `image/boat-smoke.md:230`.
- **Reason:** The branch that found it changes only docs, and guest code
  reaches a sandbox only in a new guest release and Boat snapshot.

- **Terms:**
  - **Guest:** the ciproxy program that runs in each Boat sandbox, a VM
    rented for one job, and sends its exit report (`crates/ciproxy-guest`).
  - **Exit report:** the one HTTPS POST of `RunnerExited` to
    `/hooks/boat/runner-exit` that the guest sends with its sandbox's own
    token when its runner exits.
  - **Claim:** the stored acceptance of one exit report, which sets
    `reported_at` on its delivery, the record of one VM and runner
    (`crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`).
- **Background:**
  - The recorded Boat runs used guest release 0.1.4. Its report client
    allows HTTPS only, follows no redirect, and stops after 30 seconds
    (`boat.rs:55-60`). The guest README disables redirects "to keep that
    token at its intended endpoint" (`crates/ciproxy-guest/README.md:60`).
  - The report URL and the token come from a private registration file.
    The guest keeps both as secrets (`boat.rs:20-25`).
  - After the answer, the guest calls `error_for_status`. Then it logs
    `runner exit report accepted` at the info level (`boat.rs:91-92`).
  - In the pinned `reqwest` 0.13.5 (`Cargo.lock:2716-2717`),
    `error_for_status` fails only for a client error or a server error
    (`src/async_impl/response.rs:378-386` of that crate).
  - In the pinned `http` 1.5.0 (`Cargo.lock:1564-1565`), a client error is
    400 to 499, and a server error is 500 to 599 (`src/status.rs:194-202`).
    A status code can be 100 to 999 (`src/status.rs:73-74`).
  - So the guest logs "accepted" for each answer that is not 4xx or 5xx. A
    3xx, or a nonstandard code from 600 to 999, also logs it.
  - The route answers only 204, 400, 401, or 503
    (`crates/ciproxy-control-plane/src/boat_callback.rs:22-46`). Its axum
    0.8.9 JSON extractor can also answer 400, 415, or 422 for a bad body.
    No route code answers 3xx. A 3xx alone therefore proves no claim. A
    different component can generate or change the response.
  - After an error, the guest logs only "guest observation could not be
    delivered", with no status (`crates/ciproxy-guest/src/service.rs:45-49`).
    The error text is dropped there. That text holds the report URL
    (`src/error.rs:301` of `reqwest` 0.13.5).
  - The image's Boat smoke guide says "A successful callback prints
    `runner exit report accepted`" (`image/boat-smoke.md:230`). It also
    asks for the callback server's receipt and the provider deletion
    (`:231-234`).
  - The receipt of a report is its claim, `reported_at` in persisted state.
    No live check reads the log line.
  - In the failed Boat `large` live check of 21 November 2026, the one POST
    to the route got 503. No 3xx answer was observed.
- **Problem:** The info line says "accepted" after a 3xx or a nonstandard
  answer, which can have no claim. After an error, the warning gives no
  status, so a reader cannot tell a 401 from a 503.
- **Impact if no action:** A reader of the sandbox journal can take a
  report that never reached the route for an accepted one. The harm is
  small, because the smoke guide also asks for the receipt and the
  provider deletion.
- **Root question:** How does the guest's log say truthfully which answer
  its exit report got?
- **Options:**
  - **A.** Log the numeric HTTP status of each answer, with wording that
    claims no more than the status shows.
    - An example is one info line `runner exit report answered` with the
      field `status` from `response.status().as_u16()`, before
      `error_for_status`.
    - Keep the current return value, the single attempt, and the redirect
      rule. Add no rejection and no stricter validation.
    - Log no URL, token, header, body, or `reqwest` error text.
    - Update `image/boat-smoke.md:230` to the new message.
    - Ship it in the next normal guest release. Effort: Small.
  - **B.** Make the wording conditional. Log "accepted" only for a 2xx
    answer. Log a warning with the status for each other answer that
    passes `error_for_status`. The behaviour stays the same. A 401 or a
    503 still gets the warning with no status. Effort: Small.
  - **C.** Treat each answer that is not 2xx as a failed report, so that
    the guest warns for a 3xx or a nonstandard code. This is a stricter
    validation, so it needs the user's approval first. It also changes what
    `send` returns for those answers, which needs a test and a guest
    release. Effort: Medium.
- **Left out:**
  - Change nothing, because the line keeps saying "accepted" with no
    claim, which fails the goal.
  - Log the URL, a header, the body, or the `reqwest` error text, because
    they hold secrets, and no log may hold a secret.
  - Follow redirects, because the guest README keeps the token at its
    intended endpoint.
  - A retry of a failed report, because it is a separate choice. It was
    option D of the [lock-timeout item][lock-timeout-item], and the user did
    not choose it on 22 November 2026. #116 closed that item with its
    option B.
- **Wider fix:** None. The guest makes one HTTP call, the exit report, so
  no other place repeats this log.
- **Previous examples:** None found of a log that claims more than its
  check shows. Two items record other diagnostic gaps:
  `plans/follow-ups/delivery-link-parse-warning.md` and
  `plans/follow-ups/gateway-error-close-logging.md`.
- **Related:**
  - `plans/follow-ups/boat-live-completion-acceptance.md`: its Background
    describes this line and says that no option reads it. No option there
    waits for this item.
- **Recommendation:** A. It makes the line true for each answer and keeps
  every current behaviour. B keeps the behaviour too, but its warnings
  still name no status for a 401 or a 503. C is the strictest, but it adds
  a stricter validation that needs the user.
- **Action:** fix after merge, apply A, because the guest code is outside
  this docs diff.

[lock-timeout-item]: https://github.com/Firna-AI/ciproxy/blob/2976db158049aefce6cfe96cf6337d12afc9be64/plans/follow-ups/boat-exit-report-lock-timeout.md
