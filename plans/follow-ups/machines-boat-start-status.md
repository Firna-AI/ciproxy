# Machines lacks the real Boat start status

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from pull request #50, branch
  `conductor/m20-machines-and-settings-ui`, review finding 6 of round 4b,
  deferred with option D. Recorded on 2026-11-16 from
  `conductor/m20-closure`. On 2026-11-17, the final review of the branch
  `calummoore/ci-proxy-load-balancer` found the accounting hold again as
  finding 603. Its check confirmed option D, found the missing billing and
  retry holds again, and added option F.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/view/machines.ts:134`, `boatTakesJobs`,
  `ts/app/src/view/machines-overflow.ts:47`, `caption`,
  `crates/ciproxy-api/src/wire/overflow.rs:77`, `OverflowSettings`,
  `crates/ciproxy-api/src/handlers/overflow_views.rs:52`, `read`,
  `crates/ciproxy-overflow/src/view.rs:108`, `scaler_account`, and
  `crates/ciproxy-scaler/src/overflow.rs:45`, `available`.
- **Reason:** Backend admission already stops these starts, and the fix
  reaches the overflow, scaler, and API crates after the branch that found
  it had used its permitted fix rounds.

- **Terms:**
  - **Boat:** the first overflow provider, and today the only one. ciproxy
    starts sandboxes in the customer's own Boat account while the machines
    are busy.
  - **Overflow summary:** `GET /api/workspaces/<id>/overflow/summary`,
    which every role reads, with the same `OverflowSettings` body as the
    Admin read `GET /api/workspaces/<id>/overflow`.
  - **Spend bound:** a saved maximum cost of a Boat start, which the spend
    limit counts until Boat confirms the cost.
  - **Accounting hold:** the wait of new Boat starts while a start of the
    month has no spend bound, which `Spend::counted_usd` reports as
    `AccountingIncomplete`.
  - **Scheduler rule:** `available` at
    `crates/ciproxy-scaler/src/overflow.rs:45-54`, which decides whether
    the scaler may start a new Boat runner.
- **Background:**
  - Two Machines views say whether Boat takes new jobs now. Both read the
    overflow summary.
  - The first view says where new jobs go while a machine takes none, in
    `elsewhereText` at `ts/app/src/view/machines.ts:150-164`. The list
    shows it in the notice of an offline machine. A machine page shows it
    for a machine with no slots.
  - `elsewhereText` names Boat when `boatTakesJobs` at
    `machines.ts:134-144` is true. That function checks the deployment, the
    switch, the account, the key, and `spend.limit_reached !== true`.
  - The second view is the caption of the Overflow panel of the list, in
    `caption` at `ts/app/src/view/machines-overflow.ts:47-57`. It promises
    that jobs start on Boat after the saved wait, unless overflow is off or
    the limit is reached.
  - `read` at `crates/ciproxy-api/src/handlers/overflow_views.rs:52-113`
    builds the summary from the overflow view of `ciproxy-overflow`. The
    control plane starts a Boat runner only while the scheduler rule holds.
    `scaler_account` at `crates/ciproxy-overflow/src/view.rs:108-140` gives
    the rule its input.
- **Problem:** Neither view reads `spend.bounded`, so both ignore a known
  accounting hold. The summary carries no billing or retry fact, so neither
  view can see those holds. Both views can then promise a Boat start while
  the scheduler rule holds new starts.
- **Impact if no action:** When no other machine can take a job, the page
  promises a Boat start, but the job waits, and the page gives no reason.
  Routing and spend stay correct, because backend admission holds the
  start.
- **Root question:** How should Machines learn and show whether Boat takes
  new jobs now, so that it never promises a start that the control plane
  holds?
- **Options:**
  - **A.** Handle the missing bound in both views. Show the accounting
    hold, and add its cases to the existing view tests. The views keep two
    rules, and they still miss the billing and retry holds. Effort: Small.
  - **B.** Give both views one shared client status projection. Keep an
    explicit accounting hold, and keep a missing observation apart from a
    known hold. Add the cases to the existing view tests. The summary still
    lacks the billing and retry facts. Effort: Small.
  - **C.** Add start availability to the Rust overflow summary. Generate
    its TypeScript type, and use it in both views. It sees every hold, but
    it can copy the scheduler rule in the API. Effort: Medium.
  - **D.** Give the summary one start status from the scheduler rule:
    - The status names its reason: the limit, the key, billing, a retry
      time, or the accounting hold.
    - Compute it in `ciproxy-overflow` from `scaler_account` and the
      scheduler rule, made public.
    - Give no status when the API reads stored facts after a failed or busy
      refresh. Do not show a hold then.
    - Add the Machines mockup state first.
    - Render it in both Machines views.
    - Extend the real-store gap case of
      `crates/ciproxy-api/tests/http/m18_fit_tests.rs` to read the summary,
      and extend the existing view tests.
    - Settings under Overflow reads the same body, so its `overflowNotice`
      shows the status too.
    - Jobs keeps its own rule, `boatHold`, over the `JobOverflow` facts of
      its own route.
    - Cost: a change across the overflow, scaler, API, and app code, with
      a wider contract. The retry check must use one clock origin for the
      retry time and the current time, as the control plane does.
      Effort: Medium.
  - **E.** Treat the missing bound like the other states where Boat takes
    no jobs, with existing copy only. `boatTakesJobs` returns false when
    `bounded` is false, so the notices name the machines or the wait. The
    caption reads "Jobs run on your machines only." Add both view cases. It
    names no cause, and it still misses the billing and retry holds. Its
    copy choice also works with B or D. Effort: Small.
  - **F.** Add the missing hold facts to the summary, and share one client
    rule:
    - Add `billing_required` and `retry_at` to `OverflowSettings`.
      `JobOverflow` already has both facts.
    - Move `boatHold` at `ts/app/src/view/jobs-places.ts:86` into one
      client projection that Jobs, Machines, and Settings share.
    - Add the Machines mockup state and the view cases.
    - Cost: one client copy of the scheduler rule stays. A new server hold
      then needs a client change too.
    - Effort: Medium.
- **Left out:**
  - No change, because it keeps the wrong start promise.
  - Hiding the panel, because it removes the approved account and spend
    facts.
  - A new backend guard, because backend admission already stops these
    starts.
  - A default bound or a serde alias for an old record, because it
    overrides the hold of `docs/protocol/overflow-spend.md:62-63`, under
    "What the limit counts", which commit `6450c3e` applied on 5 November
    2026. It also leaves the billing and retry holds.
  - Absent spend shown as a hold, because absent spend often comes from
    the busy try-lock of `read_view`, while the control plane can still
    start Boat jobs. "What the fix keeps" below gives the facts.
  - A read of the overflow facts of the Jobs live page from Machines,
    because it adds a poll of another route. On 8 November 2026, the user
    chose to extend the existing page responses of Machines and Settings
    with missing facts, not to read them from other routes.
  - Copy that names only the settings and claims no current start, because
    the page then explains no hold, and it changes the approved caption.
  - Tests for each possible hold still keep the copied client rule.
    D removes that copy and needs only the regression cases.
- **Wider fix:** D. One server status from the scheduler rule replaces the
  client rules, so a missing client fact cannot become a start promise. B
  is a direct fix: it joins the two client copies, but it must still read
  each hold field. F carries the missing facts, but one client copy of the
  rule stays.
- **Previous examples:**
  - The app took the add permission `can_add_organization` for the
    admission of a new organization. Commit `3d3f599` of 8 November 2026
    added the server fact `new_organization_requires_upgrade`, at
    `crates/ciproxy-api/src/wire/settings.rs:40`, from `requires_upgrade`
    in `crates/ciproxy-api/src/handlers/organization_admission.rs`.
  - `crates/ciproxy-api/src/handlers/jobs_live.rs:151` keeps a missing
    counted spend unknown, through `counted_usd().ok()`. Commit `1d209cd`
    added it.
  - The server computes the label placement, at
    `crates/ciproxy-api/src/handlers/label_boat.rs:90-96`, so no client
    rule decides it.
  - Commit `0ade974` of 8 November 2026, "fix(jobs): use current holds and
    truthful fit explanations", made Jobs keep an uncounted spend apart
    from a start promise, at `ts/app/src/view/jobs-places.ts:102`.
- **Related:**
  - `plans/follow-ups/label-boat-fit-boundary.md`, a shared server hold.
    Its option A gives the label responses the current Boat facts of
    `JobOverflow`, so that one hold rule serves the live queue and the
    label screens. `LabelPlacement` at
    `crates/ciproxy-api/src/wire/facts.rs:39-53` is physical, and it
    ignores the current hold. The start status of D is that hold, computed
    once on the server. The branch that lands second reuses the server
    source of the first, so that Machines and the label screens cannot
    disagree. Neither item waits for the other.
  - `plans/follow-ups/dashboard-duplicate-controls.md`, the same code. The
    Overflow panel at `ts/app/src/screens/machines/overflow-panel.tsx`
    uses the feature parts `Panel` and `TextLink`, which that item replaces
    with the standalone components. Only the order of the work changes.
    The options do not conflict.
- **Recommendation:** D. Both Machines views then show the status of the
  scheduler rule, so a hold that the page cannot see never becomes a start
  promise. C sees every hold too, but its copy of the rule in the API can
  drift. The first-pass review recommended B, which still misses the
  billing and retry holds. The final review of 17 November 2026 also
  recommended B first, and its check confirmed D for the same reason.
- **Action:** fix after merge, apply D, because backend admission already
  stops these starts, and the fix reaches the overflow, scaler, and API
  crates after the branch that found it had used its permitted fix rounds.

## Requirements

`docs/protocol/overflow.md`, `docs/protocol/overflow-spend.md`,
`docs/protocol/job-routing.md`, and `docs/protocol/job-routing-failures.md`
already say when a provider, such as Boat, takes jobs. D shows that status,
and it adds no requirement. On 23 November 2026, the user chose several
providers for a workspace, one spend limit for each provider, and an order
of places for each label. A job then goes to the next place of its order
when the current place cannot take it now. Keep these rules:

- `overflow-spend.md:62-63`, under "What the limit counts": when a start
  has no safe bound, the limit holds new starts on that provider until that
  cost resolves. At `:79`, under "Boat start fees", the limit also holds
  new Boat starts when no source supplies a start fee.
- `overflow-spend.md:46-47`: the limit counts confirmed costs and the
  pending bound of each start whose cost the provider has not confirmed.
- `overflow-spend.md:91-95`, under "At the limit": at the limit of a
  provider, ciproxy starts no new runner there. While the limit holds new
  starts, jobs skip that provider and go to the next place of their order.
- `overflow.md:91-92`, under "A rejected key", and `:231-232`: after a
  provider rejects the key, jobs skip that provider until the customer
  connects a new key.
- `overflow.md:220-228`, under "Refused sandboxes": after a refusal of new
  sandboxes, the provider takes no job until the retry time that it gives.
  A refused size stops only that size. Jobs that the provider cannot take
  go to the next place of their order, and ciproxy tries the provider
  again. At `:233-234`, a billing refusal holds the provider until the
  account can start sandboxes again.
- `overflow-spend.md:64-66`: Settings under Overflow and the Machines page
  show the confirmed spend and the pending amount of each provider apart,
  against its monthly limit.
- `job-routing.md:149-160`, under "When a provider cannot take a job": a
  provider cannot take a job while overflow is off, while it is not
  connected or not available, after it rejected the key, while its spend
  limit holds new starts, before the retry time of a refusal, while its
  account needs billing, or when no size of its account fits. At
  `:89-92`, under "The label order", such a job goes to the next place of
  its order.
- `job-routing-failures.md:21-22`, under "Own machine offline": new jobs
  run on the other own machines and at the other places of their order.
- `job-routing-failures.md:31-32`, under "A provider fails": while a
  provider refuses new sandboxes, new jobs skip it and go to the next place
  of their order.

## Evidence

- **The state.** A valid summary can have an available deployment,
  overflow on, and an accepted key, with `spend.bounded: false` and
  `spend.limit_reached: null`. `read` sets both at
  `crates/ciproxy-api/src/handlers/overflow_views.rs:81-94`. The parser
  keeps both, at `ts/app/src/api/machine-parse.ts:40` and `:42`.
- **What the page says.** With one offline machine and no other healthy
  machine, the list notice says "New jobs run on Boat." A machine with no
  slots says "No slots. Jobs run on Boat." The panel says "Connected to
  the account mokly · jobs start there after 30 seconds". Its note stays
  empty, because `spend` exists. The final review of 17 November 2026
  reproduced the same text with an offline machine and unbounded spend.
- **The accounting hold.** `Spend::counted_usd` at
  `crates/ciproxy-overflow/src/view.rs:36-41` fails with
  `AccountingIncomplete` while a start of the month has no bound. Then the
  control plane keeps the account of `retained_scaler_account`, with
  `enabled: false`, and logs "Boat accounting prevents new starts", at
  `crates/ciproxy-control-plane/src/boat_runtime.rs:88-102`. The start path
  also stops, at `crates/ciproxy-overflow/src/service/start.rs:80`.
- **The billing and retry holds.** `scaler_account` maps
  `billing_required` and `retry_at` of the stored account state to
  refusals, at `view.rs:119-125`. The scheduler rule holds new starts for
  both. `preparation_wait` at
  `crates/ciproxy-overflow/src/service/provider_preparation.rs:21-28` also
  makes a start wait for them. `OverflowSettings` at
  `crates/ciproxy-api/src/wire/overflow.rs:77-92` carries neither fact.
  `JobOverflow` of the Jobs live page carries both, at
  `crates/ciproxy-api/src/wire/job_overflow.rs:16-19`.
- **The likelihood.** The missing bound is rare. Each normal start saves a
  fee bound before dispatch, at `start.rs:95-97` and `:142`. Commit
  `309b6ba` of 5 November 2026 renamed the saved field `start_quote_usd`
  to `start_fee_bound_usd`, with no serde alias. So a start record from
  before it reads the bound as absent. The hold ends at the next UTC month
  at the latest, because the fee counts only in its start month, at
  `crates/ciproxy-overflow/src/spend.rs:45` and `:64-71`. Billing and retry
  holds come from ordinary Boat refusals, so the whole gap is Common.
- **The clock of the rule.** The rule compares `snapshot.now` with a retry
  time that `scaler_account` gives as a duration from its clock origin.
  The control plane passes `DateTime::UNIX_EPOCH`, at
  `boat_runtime.rs:99`, and builds `now` from the same origin, at
  `crates/ciproxy-control-plane/src/snapshot.rs:199`.
- **Other client copies of the rule.** Settings under Overflow words its
  notice in `overflowNotice` at `ts/app/src/view/boat.ts:143-163`. It
  gives the accounting hold for `bounded: false`, but no billing or retry
  hold. The Jobs page decides in `boatHold` at
  `ts/app/src/view/jobs-places.ts:86-106`, from `JobOverflow`. It holds
  Boat for billing and for a retry time after the read. It keeps an
  uncounted spend apart from a missing observation.
- **The tests.** Every spend object of the Machines tests has
  `bounded: true`: `ts/app/_tests_/machines-fixtures.ts:53`,
  `ts/app/_tests_/machines-view.test.ts:115`, and
  `ts/app/_tests_/machines-overflow.test.ts:22` and `:63`. The Machines
  browser harness does the same, in the `overflow/summary` reply of
  `ts/app/scripts/m20-harness-server.ts`. No Machines case covers a missing
  bound, billing, or a retry time. Only the Settings tests cover
  `bounded: false`, at `ts/app/_tests_/boat-view.test.ts:58` and `:103`.
- **The real-store gap.** The waiting-label fit test at
  `crates/ciproxy-api/tests/http/m18_fit_tests.rs:64-77` seeds the gap on
  PostgreSQL with `m18_gap_support::seed`. It proves that `scaler_account`
  fails with `AccountingIncomplete`, but it reads only the Jobs live page.
  No HTTP test reads `bounded: false` from the summary.

## What the fix keeps

- Keep a missing observation apart from a known hold. `read_view` takes
  the overflow lock with a try-lock, at
  `crates/ciproxy-overflow/src/service/refresh.rs:86-88` and
  `crates/ciproxy-overflow/src/session.rs:26-31`. When the lock is busy,
  the summary falls back to stored facts with no `spend`, at
  `crates/ciproxy-api/src/handlers/overflow_views.rs:53-60`. The control
  plane can still start Boat jobs then, so do not show that state as a
  hold. Keep its current text, which
  `ts/app/_tests_/machines-overflow.test.ts:74-78` expects: the start
  caption, with the note "Boat spend shows here when Boat reports it."
- Do not weaken admission, and do not guess a missing cost.
- Use the generated API types. Run `npm --prefix ts run api:generate`, and
  do not edit `ts/api/generated/` by hand.
- Keep the assertions of the existing view tests.
- Use existing hold copy where it fits:
  - The Machines copy in `ts/app/src/view/machines-copy.ts`, such as "Jobs
    run on your machines only." and "Jobs wait until a machine can take
    them."
  - The accounting hold of Settings under Overflow: "Boat costs are not
    known yet", with "Jobs run on your machines only until Boat reports
    the cost of each start." Its mockup state is `unknown-costs`, in
    `docs/mockups/src/entries/app/settings/overflow/spend/states.mockup.tsx`.
  - The billing and retry notices of the Jobs page, at
    `ts/app/src/view/jobs-boat-notices.ts:66-83`.
- The Machines mockups show only the start promise, at
  `docs/mockups/src/screens/machines/overflow-panel.tsx:56-61` and in the
  `offline` state at
  `docs/mockups/src/entries/app/machines/list/states.mockup.tsx:21-31`.
  When the fix needs hold copy that no Machines mockup state shows, add
  that state before the implementation. This holds for every option, also
  for A, B, and C.
- The app also shows the off, limit, rejected-key, not-connected, and
  machines-only captions, from `ts/app/src/view/machines-copy.ts:215-221`.
  No Machines mockup shows these states.
- The summary store in `ts/app/src/state/machines-overflow.ts` is built on
  `createPageRead` of `ts/app/src/state/page-store.ts` since 17 November
  2026, the read rule of `createPageStore` with no interval. Read the status
  from its snapshot. A failed refresh then shows beside a kept status that
  can be old.
- Reuse the setup of the real-store gap case for the HTTP regression of the
  summary.
