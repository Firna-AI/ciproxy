# Boat report logs hide the answer status

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-21, from `conductor/boat-callback-diagnosis`, a
  problem that the check of review finding 9, round 2, found outside the
  scope of that docs-only diff.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-guest/src/boat.rs:41-51`
  (`BoatCallback::post`), `:82-87` (client), and `:115-119`
  (`BoatChannel::send`), `crates/ciproxy-guest/src/usage/mod.rs:212-214`,
  `crates/ciproxy-control-plane/src/boat_callback.rs:58-68`, and
  `image/boat-smoke.md:230`.
- **Reason:** The branch that found it changes only docs, and guest code
  reaches a sandbox only in a new guest release and Boat snapshot.

- **Terms:**
  - **Guest:** the ciproxy program that runs in each Boat sandbox, a VM
    rented for one job, and sends its exit report (`crates/ciproxy-guest`).
  - **Exit report:** the one HTTPS POST of `RunnerExited` to
    `/hooks/boat/runner-exit` that the guest sends with its sandbox's own
    token when its runner exits.
  - **Sample report:** the HTTPS POST of `UsageSamples` to the same route
    that the guest sends every 30 seconds while its job runs, with the CPU
    and memory figures so far.
  - **Claim:** the stored acceptance of one exit report, which sets
    `reported_at` on its delivery, the record of one VM and runner
    (`crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`).
- **Background:**
  - The recorded Boat runs used guest release 0.1.4. The report client
    allows HTTPS only, follows no redirect, and stops after 30 seconds
    (`boat.rs:82-87`). The guest README disables redirects "to keep that
    token at its intended endpoint" (`crates/ciproxy-guest/README.md:84`).
  - The report URL and the token come from a private registration file.
    The guest keeps both as secrets (`boat.rs:23-27` and `:34-38`).
  - After the answer, the guest calls `error_for_status`. Then it logs
    `runner exit report accepted` at the info level (`boat.rs:115-119`).
  - In the pinned `reqwest` 0.13.5 (`Cargo.lock:2833-2834`),
    `error_for_status` fails only for a client error or a server error
    (`src/async_impl/response.rs:378-386` of that crate).
  - In the pinned `http` 1.5.0 (`Cargo.lock:1616-1617`), a client error is
    400 to 499, and a server error is 500 to 599 (`src/status.rs:194-202`).
    A status code can be 100 to 999 (`src/status.rs:73-74`).
  - So the guest logs "accepted" for each answer that is not 4xx or 5xx. A
    3xx, or a nonstandard code from 600 to 999, also logs it.
  - For an exit report, the route answers only 204, 400, 401, or 503
    (`crates/ciproxy-control-plane/src/boat_callback.rs:38-50`). Its axum
    0.8.9 JSON extractor can also answer 400, 415, or 422 for a bad body.
    No route code answers 3xx. A 3xx alone therefore proves no claim. A
    different component can generate or change the response.
  - After an error, the guest logs only "guest observation could not be
    delivered", with no status (`crates/ciproxy-guest/src/service.rs:48-51`).
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
  - Both reports send through `BoatCallback::post`. Only the exit path,
    `BoatChannel::send`, logs `runner exit report accepted`. The sample
    path warns "job usage sample report could not be delivered" with no
    status (`crates/ciproxy-guest/src/usage/mod.rs:212-214`), and a
    successful sample report writes no line.
  - The route answers a refused sample report with 400, 401, 410, 422, or
    429, and it logs no line (`crates/ciproxy-control-plane/src/boat_callback.rs:58-68`).
    The plan of the CPU and memory change gave these answers so that a
    guest can tell a closed sandbox from a fault.
- **Problem:** The info line says "accepted" after a 3xx or a nonstandard
  answer, which can have no claim. After an error, the warning gives no
  status, so a reader cannot tell a 401 from a 503. For a sample report,
  no log on either side says why the control plane refused it.
- **Impact if no action:** A reader of the sandbox journal can take a
  report that never reached the route for an accepted one. The harm is
  small, because the smoke guide also asks for the receipt and the
  provider deletion.
- **Root question:** How do the logs say truthfully which answer each
  Boat report got, the exit report and the sample report, now that both
  send through one sender?
- **Options:**
  - **A.** Let `BoatCallback::post` give the numeric HTTP status to its
    caller, and log at each caller, with wording that claims no more than
    the status shows.
    - The exit path logs one info line, for example
      `runner exit report answered`, with the field `status`.
    - The sample path stays quiet on a success. For each other answer, it
      warns with the status.
    - The route logs one warning line for each refused sample report, with
      the answer code and the name of the reason, and no value from the
      body.
    - Keep the current return value, the single attempt, and the redirect
      rule. Add no rejection and no stricter validation.
    - Log no URL, token, header, body, or `reqwest` error text.
    - Update `image/boat-smoke.md:230` to the new message. Ship it in the
      next normal guest release. Effort: Small.
  - **B.** Let `BoatCallback::post` take the kind of report, and log in the
    sender. It logs the exit answer at the info level and a sample answer
    only when it is not a success. The route logs as in A. Effort: Small.
  - **C.** Treat each answer that is not 2xx as a failed report, so that
    the guest warns for a 3xx or a nonstandard code. It changes
    `BoatCallback::post`, so it changes the failure handling of both
    report kinds. This is a stricter validation, so it needs the user's
    approval first. Effort: Medium.
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
- **Wider fix:** None. The `documentation` skill and the follow-up rules
  already ask that each log claims only what its check shows.
- **Previous examples:** None found of a log that claims more than its
  check shows. Two items record other diagnostic gaps:
  `plans/follow-ups/delivery-link-parse-warning.md` and
  `plans/follow-ups/gateway-error-close-logging.md`.
- **Related:**
  - `plans/follow-ups/boat-live-completion-acceptance.md`: its Background
    describes this line and says that no option reads it. No option there
    waits for this item.
  - `plans/follow-ups/job-usage-report-size-limit.md`: its option D adds a
    step drop before the shared `BoatCallback::post`. The drop then applies
    to both report kinds, and the options here must say so.
- **Recommendation:** A. It makes each line true for each answer, keeps
  every current behaviour, and keeps the sender free of report kinds. B
  puts the log in one place, but the sender must then know each kind. C is
  the strictest, but it adds a stricter validation that needs the user.
- **Action:** fix after merge, apply A, because the fix is a Low log
  change that reaches a sandbox only in a new guest release.

[lock-timeout-item]: https://github.com/Firna-AI/ciproxy/blob/2976db158049aefce6cfe96cf6337d12afc9be64/plans/follow-ups/boat-exit-report-lock-timeout.md
