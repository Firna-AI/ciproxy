# A busy workspace lock drops Boat exit reports

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `conductor/boat-callback-diagnosis`, item 1
  of the read-only code and contract trace of the failed Boat `large` live
  check. Later checks added F, G, and H, and corrected B, F, and G.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-store-pg/src/connector.rs:79-83`,
  `crates/ciproxy-store-pg/src/overflow/queries.rs:39-49`,
  `crates/ciproxy-overflow/src/service/cleanup.rs:28-33`,
  `crates/ciproxy-store-interface/src/overflow/store.rs:44`, and
  `crates/ciproxy-control-plane/overflow.md:136-137`.
- **Reason:** The recommended option changes protocol requirements, so the
  human user must decide. This branch records the decision and no code.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Exit report:** the one HTTPS POST of `RunnerExited` to
    `/hooks/boat/runner-exit` that the guest, the ciproxy program in the
    sandbox (`crates/ciproxy-guest`), sends with its sandbox's own token.
  - **Claim:** the acceptance of one exit report for a delivery, the record
    of one VM and runner. It saves `reported_at` and the `Deleting` phase in
    `crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`.
  - **Workspace lock:** the PostgreSQL advisory lock of one workspace, in
    `crates/ciproxy-store-pg/src/overflow/queries.rs:24-76`.
  - **Orchestrator:** the agent session that runs this Boat task and owns
    its run order and its budget of six paid sandboxes. It is not the user.
- **Background:**
  - Start, resume, view refresh, connect, disconnect, cleanup, and the exit
    report hold the workspace lock, also during Boat, GitHub, and key calls.
    A session frees it only at its close, after its whole pass (`store.rs:42`).
  - Only the exit report waits for the lock (`cleanup.rs:28`). The store
    interface says why: "so a guest callback is never lost as busy"
    (`store.rs:44`). All other operations use a try-lock and fail at once
    with `Busy` (`crates/ciproxy-overflow/src/session.rs:22-31`). A failed
    pass runs again only at a later trigger, such as the 10-second tick
    (`crates/ciproxy-control-plane/src/runtime_tasks.rs:225-229`). So no
    later pass has a deadline to get the lock.
  - The wait runs `SELECT pg_advisory_lock($1)` on a new connection with
    `lock_timeout = 5000` (`queries.rs:39-49`, `connector.rs:79-83`).
    PostgreSQL applies it to advisory locks too, as a store test of the
    organization lease shows
    (`crates/ciproxy-store-pg/src/github/lease.rs:16-27` and
    `crates/ciproxy-store-pg/tests/store/connection_tests.rs:102-122`).
  - The claim comes only after the lock (`cleanup.rs:28-33`), so a
    cancelled wait saves nothing and returns 503 with no log line
    (`crates/ciproxy-control-plane/src/boat_callback.rs:41-45`). The claim
    exists only on the session interface, whose connection holds the lock
    (`store.rs:50-52` and `:95-101`). It accepts a report only for a
    delivery that has not ended when it runs (`completion.rs:29`). The
    crate README states that rule, not a protocol doc
    (`crates/ciproxy-overflow/README.md:120-121`).
  - Each delivery save writes `record_json`, `reported_at`, and `ends_at`
    from the caller's own copy, through one function
    (`crates/ciproxy-store-pg/src/overflow/deliveries.rs:55-66`). The
    snapshot reads the phase and `reported_at` from `record_json`
    (`overflow/rows.rs:88-96`). Only `Ended` makes the runner `stopped`
    (`overflow/placements.rs:24-40`). A view refresh and the cleanup save
    running deliveries from older copies. See `refresh.rs:86-99`,
    `usage.rs:17-44` and `:121`, `costs.rs:13-28` and `:41-50`, and
    `inventory.rs:19` and `:195-199` in `crates/ciproxy-overflow/src/service/`.
    A stale save that restores `Running` makes the next pass ask GitHub
    again, and a waiting job then keeps the sandbox (`inventory.rs:182-189`
    and `crates/ciproxy-control-plane/src/boat_snapshot.rs:58-60`).
  - The cleanup's GitHub retirement reads a guard of the delivery before
    its GitHub call. It writes its result only on an equal guard
    (`crates/ciproxy-control-plane/src/boat_authority.rs:151-158` and
    `:195-212`, `crates/ciproxy-store-pg/src/boat_results.rs:44-47`). The
    guard holds the phase, `ends_at`, and `reported_at`
    (`crates/ciproxy-store-interface/src/boat_delivery.rs:41-48`).
  - The pinned Boat adapter treats a 404 on delete as success
    (`crates/sandbox-boat/src/control/deletion.rs:28-29` of the
    `futex-ai/sandbox` dependency at `ac412fc`). So a second delete ends
    the delivery again, with a later `ends_at` (`cleanup.rs:60-79`).
  - The guest makes one POST with a 30-second client limit and no retry.
    After a failure, it logs one warning and ends. Its unit has `Restart=no`
    (`image/ciproxy-boat-guest.service:11`), and a test protects the single
    attempt. The code is in `crates/ciproxy-guest/src/`: `boat.rs:55-60` and
    `:81-95`, `service.rs:45-49` and `:156-159`, and
    `_tests_/service_tests.rs:140`.
  - A completed GitHub job starts the job-end cleanup at once, and the
    10-second tick starts the same cleanup, in
    `crates/ciproxy-control-plane/src/`: `boat_jobs.rs:45-49`,
    `listeners.rs:132-144`, and `boat_runtime.rs:35-38`. It takes the lock
    with the try-lock. It keeps the lock while it asks GitHub to remove the
    runner registration, measures runtime, and deletes the sandbox
    (`inventory.rs:16-17` and `:182-209`, `cleanup.rs:45-80`). It does not
    read or wait for the exit report.
  - The protocol permits that order. It requires deletion when the job
    ends. It makes the exit report a fast trigger and the sandbox lifetime
    the backstop (`docs/protocol/overflow.md:114-129`). No protocol line
    requires a recorded claim for each successful job. For a runner that
    exits with no job, it requires deletion at once on the report
    (`:115-118`). Without a report, that runner counts as ready, and a
    waiting job of its label keeps the sandbox (`boat_snapshot.rs:58-60`).
  - After an App removal, a started sandbox ends only at its exit report or
    at its lifetime. That is 6 hours, or 2 hours on a Boat free trial
    (`docs/protocol/overflow.md:158-164` and
    `docs/protocol/app-removal.md:24-29`). The GitHub retirement step then
    refuses, so the job-end cleanup keeps the sandbox
    (`boat_authority.rs:85-90`).
  - Observed run, 21 November 2026: Boat `large` job 136802119867 on
    `bx_tn79cdzz` succeeded, with its locale, PostgreSQL, and Docker steps.
    - 09:27:58.869556Z: the control plane saved the completed job.
    - 09:27:59.621438Z: the only POST to the exit route in a four-minute
      query reached the load balancer. It got 503 after 5.351988 seconds.
      Its timing strongly fits this guest. The entry names no sender.
    - 09:28:01.793Z: the check turned overflow off. This came after the
      POST, so it did not start the wait.
    - 09:28:04.921207Z: Cloud SQL cancelled a `pg_advisory_lock` wait of
      process 2341176 with a lock timeout.
    - 09:28:10.142605Z: the delivery ended with `reported_at` NULL. The
      provider now returns 404 for the sandbox.
  - The times above are log facts. The holder's process ID and release time
    were not retained. That the job-end cleanup held the lock is an inference
    from the times and the code. Under it, the release came after the end at
    09:28:10.14Z, when the cleanup finished its pass (`inventory.rs:200-209`).
    The outcomes below rest on it and on the POST's fit to this guest.
  - The orchestrator's grade call, 21 November 2026: the likelihood is
    Common. Routine cleanup, start, view refresh, and provider calls of
    the same workspace can hold the lock for more than 5 seconds. Parallel
    exit reports of normal CI jobs also compete for it. A normal job showed
    the drop, with no unusual setup or input. One observed run gives no
    measured rate.
- **Problem:**
  - An exit report that waits more than 5 seconds for the workspace lock
    fails with 503 and saves nothing. The guest does not send it again.
  - A successful job can meet this race, because its job-end cleanup
    starts at about the same moment as its exit report. One observed run
    showed it. There, the job-end cleanup still deleted the sandbox.
  - `store.rs:44` and the guide's 503 row, "The runtime retries saved
    cleanup" (`crates/ciproxy-control-plane/overflow.md:137`), promise that
    the report is not lost. Neither holds before the claim. The 401 row at
    `:136` omits a report after the end (`cleanup.rs:30-33`).
- **Impact if no action:** A dropped report can keep a sandbox billing for
  up to 6 hours when its runner exits with no job, or after an App
  removal. In the no-job case, a waiting job of the same label can also
  stay stranded.
- **Root question:** How does an exit report take effect when other Boat
  work of the same workspace runs at the same moment?
- **Options:** "Proposed responses" below gives the answers of each.
  - **A.** Let the report wait longer. Give only the report's lock session
    its own limit below the guest's 30-second client limit, for example
    25 seconds. A report behind a start, a refresh, or another report then
    takes effect. Behind the job-end cleanup of its own delivery, it gets
    401 only if the lock comes within the wait. The claim needs `ends_at`
    NULL (`completion.rs:24-36`). Otherwise it gets 503. So in the observed
    order, the claim is still lost. Each waiting report holds one more
    database connection. Effort: Small.
  - **B.** Save the report as a durable claim first, without the lock.
    Then wait for the lock, and delete in the request when it comes.
    - Move the claim off the session interface, and update its doc line.
      The claim still accepts one report per token, only for a delivery
      that has not ended.
    - One rule covers every save from any copy, also the route's own older
      claim copy. `reported_at` and the first `ends_at` stay once saved.
      The phase never moves back from `Deleting` or `Ended`. A save that
      ends the delivery moves `Deleting` to `Ended` and keeps `reported_at`.
      No save clears an end or moves it later.
    - Only a claim between the guard read and its write gives
      `RunnerNeeded`. A later pass then deletes the `Deleting` delivery
      (`inventory.rs:87-97` and `:190-193`). So such a claim moves the
      delete of a successful job from its usual pass to a later one. A
      claim outside that window passes, and the cleanup ends the delivery
      from its older copy under the rule above. This needs a test.
    - The wait can keep today's 5-second limit, because the claim is
      already saved. A longer limit only makes a deferral rarer.
    - B also keeps a report behind a cleanup pass that ends several
      deliveries in turn, longer than any wait that fits the guest's limit.
    - In the observed run, B would have saved the claim at about
      09:27:59.9Z, before the end. That matters for options C and E of
      `plans/follow-ups/boat-live-completion-acceptance.md`: C keeps a claim
      for each case, and E for one new `large` run. B's main value is the
      report of a no-job exit and after an App removal.
    - An error-class log of a failed report can support its validation.
      Effort: Medium.
  - **C.** Delay the job-end deletion by a short grace time, for example
    30 seconds, while no report exists. A report that gets the lock within
    the grace can then claim before the job-end deletion. C does not stop
    drops behind other holders, such as a start or another report. It adds
    the grace time to the bill of each sandbox whose report never comes.
    It changes the timing of `docs/protocol/overflow.md:114`, so the
    protocol doc and the grace value need the user. Effort: Medium.
  - **D.** Retry until 204, a 4xx answer, or a time limit. Retry each
    other status and network error without following redirects. In the
    observed run, the retries end with 401 or with the deleted sandbox, so
    the claim is still lost. D needs a new guest, an agent release, and a
    new Boat snapshot build. It also changes the tested single attempt and
    `crates/ciproxy-guest/README.md:58-62`. Effort: Large.
  - **E.** Hold the workspace lock only around store writes, not during
    Boat, GitHub, and key calls, in start, cleanup, refresh, and the exit
    report. Long holds then end, so only E deletes without waiting for a
    long holder. Per-delivery compare-and-swap writes are one form of E. E
    redesigns the one-lock model of `crates/ciproxy-overflow/README.md:40`
    and adds races between provider calls and saves. Effort: Large.
  - **F.** Judge the report at its arrival.
    - Read the trusted time before the lock wait, and give only the
      report's session the longer limit of A.
    - Accept the claim also for a delivery that ended after that time. Such
      a late claim saves only `reported_at`, keeps `Ended`, and deletes
      nothing.
    - A delivery end is not a lock release. F records the claim only if
      the request gets the lock within its wait limit.
    - In the observed run, the wait began at about 09:27:59.9Z. With the
      example limit of 25 seconds, it would end at about 09:28:24.9Z, about
      15 seconds after the recorded end. Only if the lock came free before
      then would F record `reported_at` and return 204. The window moves
      with the chosen limit.
    - No delivery save changes, so F adds no concurrent write. A holder
      that runs past the limit, or a restart during the wait, still drops
      the report, as in A. So F protects the no-job and App-removal cases
      no better than A.
    - F also changes "The store accepts it once for its one live runner"
      (`crates/ciproxy-overflow/README.md:121`). Effort: Medium.
  - **G.** Save the report as its own durable fact that no delivery save
    writes. An example is a received-at column that only the route sets,
    without the lock.
    - Only a lock holder turns that fact into the claim: the waiting report
      route, or a later cleanup pass. A report that arrived before the end
      also records on an ended delivery. So G defers the delete too.
    - The delivery row keeps one writer under the lock. So no save can drop
      the report, and no save needs the rule of B.
    - G needs a migration. The snapshot must read the new fact, so that the
      scaler stops counting the runner as ready. The cleanup needs a new
      step that applies the fact. Effort: Large.
  - **H.** Do B, but try the lock once after the claim, and never wait.
    - H holds no database connection while the lock is busy, and it needs
      no lock limit. The save rule and the guard window are those of B.
    - In the observed run, H would also have saved the claim at about
      09:27:59.9Z, and the delete would have waited for a pass. Effort: Medium.
- **Left out:**
  - Change nothing, because reports keep dropping behind each holder that
    runs longer than 5 seconds.
  - An image-only reporter, because D is better on every point. The guest
    already holds the token. Both need a new snapshot, and both end in 401
    in the observed order. D also covers network errors.
  - A load balancer retry of 503, because A is better on every point. The
    retry gives the longer wait of A, but it needs infrastructure and
    repeats a one-use POST.
  - A longer lock limit for every connection, because it changes each waiting
    lock and the tested 5-second policy (`connection_tests.rs:28-122`).
  - Give the report priority over a running holder, because a holder
    cannot stop in the middle of a provider call.
- **Wider fix:** B, or H in its try-lock form. The report becomes a durable
  claim that no holder can drop, once each save keeps the claim and the
  first end. G is equally structural, at Large effort.
- **Previous examples:** None found of this cause.
  `plans/follow-ups/boat-launched-runner-readiness.md` lists "a failed
  report" as one cause of a stranded job, with no cause named.
- **Related:**
  - `plans/follow-ups/boat-live-completion-acceptance.md`: its options C
    and E keep a claim rule and wait for B, C, F, G, or H of this item and
    their deploy. In the observed order, B, G, and H record the claim, and
    C and F only under the conditions in their texts. Its option F uses
    earlier accepted reports, which this item does not change.
  - `plans/follow-ups/boat-launched-runner-readiness.md`: the same class,
    where a lost report strands a waiting job. Its ready deadline is a
    backstop for each lost report, not a fix of this cause.
  - `plans/follow-ups/boat-exit-report-response-log.md`: it corrects the
    guest's acceptance log line in `BoatChannel::send`, which D also
    changes. Neither item waits for the other.
- **Recommendation:** B. It keeps each report whatever holds the lock,
  needs no migration, and deletes in the request more often than H. G best
  protects the delivery row against later writers, at Large effort.
- **Action:** ask the user, because B changes the requirements of
  `docs/protocol/overflow.md:114`, `:117`, and `:160`, and no recorded user
  decision covers that change. A recommendation applies nothing.

## Proposed responses

The table covers replies after JSON extraction. The user chooses a letter
with its proposed responses, or names the alternative below.

| Letter | Parsed message is not `RunnerExited`, or the runner ID is not a UUID | Bad or used token, or ended | Fails before the save | Saved, then a step fails or the delete waits | Handler completes without another error | A success means | Delete time | Protocol text |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | 400 | 401. if ended, only when the lock comes within the wait, else 503 | 503, also when the wait ends | 503, as today. maintenance retries remaining cleanup | 204 | Saved and deleted | In the request, after the lock | No change |
| B | 400 | 401, also for a first POST after the end | 503 on a store error | 204 | 204 | Report saved | In the request within the wait, else a later pass | Delete time and success sentence |
| C | 400 | 401. a POST after the grace can meet an ended delivery | 503, also when the wait ends | 503, as today | 204 | Saved and deleted | The job-end delete waits for the grace | Grace time at `:114` |
| D | 400. the guest stops | 401. the guest stops | 503. the guest retries | 503. the retry then gets 401 | 204 | Saved and deleted | As today | No change |
| E | 400 | 401 | 503 | 503, as today | 204 | Saved and deleted | In the request, with no long holder | No change |
| F | 400 | 401 for a POST after the end | 503 when the wait ends first | 204 | 204, also for a late claim | Report saved | As today. a late claim deletes nothing | Success sentence |
| G | 400 | 401 for a POST after the end | 503 on a store error | 204 | 204 | Fact saved. a lock holder sets `reported_at` later | In the request within the wait, else a later pass | As B |
| H | 400 | 401, also for a first POST after the end | 503 on a store error | 204 | 204 | Report saved | In the request only if the lock is free, else a later pass | As B |

- Alternative: B, F, G, and H keep today's 503 for each failure after the
  save, also for a deferred delete. The user can name it instead.
- The protocol lines are `docs/protocol/overflow.md:114`, `:117`, and
  `:160`. The success sentence is "A success answer means only that the
  control plane saved the report." Each changed letter updates `store.rs:44`
  and the guide's table (`crates/ciproxy-control-plane/overflow.md:132-137`).
- These answers reach the guide, the load balancer log, the live checks,
  and the retry rule of D. Guest 0.1.4 acts on no status. It logs
  acceptance for each answer that is not 4xx or 5xx (`boat.rs:91-92`).

Evidence of the observed run, outside Git: the folder `callback-resume/`
in `.context/ciproxy-mvp/live-2026-11-20/boat-runner-start-2026-11-21/`.
