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
- **Reason:** The user asked to see the options before any product code
  change, so this branch records the decision and changes no code.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Guest:** the ciproxy program in the sandbox that starts one GitHub
    runner and reports its exit, in `crates/ciproxy-guest`.
  - **Exit report:** the guest's one HTTPS POST of `RunnerExited` to
    `/hooks/boat/runner-exit`, with a token for its own sandbox only.
  - **Claim:** the acceptance of one exit report for a delivery, the record
    of one VM and runner. It saves `reported_at` and the `Deleting` phase in
    `crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`.
  - **Workspace lock:** the PostgreSQL advisory lock that each overflow
    operation of one workspace holds for its whole run, in
    `crates/ciproxy-store-pg/src/overflow/queries.rs:24-76`.
- **Background:**
  - Each overflow operation of a workspace holds the workspace lock for
    its whole run, also during Boat, GitHub, and key calls. The operations
    are start, resume, view refresh, connect, disconnect, cleanup, and the
    exit report. A session frees the lock only when it closes, at the end
    of its whole pass (`store.rs:42`).
  - Only the exit report waits for the lock (`cleanup.rs:28`). The store
    interface says why: "so a guest callback is never lost as busy"
    (`store.rs:44`). All other operations use a try-lock and fail at once
    with `Busy` (`crates/ciproxy-overflow/src/session.rs:22-31`). A failed
    pass runs again only at a later trigger, such as the 10-second tick
    (`crates/ciproxy-control-plane/src/runtime_tasks.rs:225-229`). So no
    later pass has a deadline to get the lock.
  - The wait runs `SELECT pg_advisory_lock($1)` on a new store connection
    (`queries.rs:39-49`), which sets `lock_timeout = 5000`
    (`connector.rs:79-83`). PostgreSQL applies this limit to advisory lock
    waits too. The organization lease uses the same pattern, and a store
    test expects its lock timeout
    (`crates/ciproxy-store-pg/src/github/lease.rs:16-27` and
    `crates/ciproxy-store-pg/tests/store/connection_tests.rs:102-122`).
  - The claim comes only after the lock (`cleanup.rs:28-33`). So a
    cancelled wait saves nothing, and the route returns 503 with no log
    line (`crates/ciproxy-control-plane/src/boat_callback.rs:41-45`).
  - The claim exists only on the session interface, whose connection holds
    the lock (`store.rs:50-52` and `:95-101`). It accepts a report only for
    a delivery that has not ended when the claim runs (`completion.rs:29`).
    The crate README states that rule, not a protocol doc
    (`crates/ciproxy-overflow/README.md:120-121`).
  - Each delivery save writes `record_json`, `reported_at`, and `ends_at`
    from the caller's own copy, through one function
    (`crates/ciproxy-store-pg/src/overflow/deliveries.rs:55-66`). The
    snapshot reads the phase and `reported_at` from `record_json`
    (`crates/ciproxy-store-pg/src/overflow/rows.rs:88-96`). Only `Ended`
    makes the runner `stopped` (`overflow/placements.rs:24-40`).
  - A view refresh and the cleanup save running deliveries from older
    snapshot copies. The code is in `crates/ciproxy-overflow/src/service/`:
    `refresh.rs:86-99`, `usage.rs:17-44` and `:121`, `costs.rs:13-28` and
    `:41-50`, and `inventory.rs:19` and `:195-199`. A stale save that
    restores `Running` makes the next pass ask GitHub again, and a waiting
    job then keeps the sandbox (`inventory.rs:182-189` and
    `crates/ciproxy-control-plane/src/boat_snapshot.rs:58-60`).
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
    After a failure, it logs one warning and ends
    (`crates/ciproxy-guest/src/boat.rs:55-60` and `:81-95`,
    `crates/ciproxy-guest/src/service.rs:45-49` and `:156-159`). Its unit
    has `Restart=no` (`image/ciproxy-boat-guest.service:11`), and a test
    protects the single attempt
    (`crates/ciproxy-guest/src/_tests_/service_tests.rs:140`).
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
    requires a recorded claim for each successful job.
  - For a runner that exits with no job, the protocol requires deletion at
    once on the exit report (`docs/protocol/overflow.md:115-118`). Without
    a report, the runner counts as ready, and a waiting job of its label
    keeps the sandbox (`boat_snapshot.rs:58-60` and `inventory.rs:189`).
  - After an App removal, a started sandbox ends only at its exit report or
    at its lifetime. That is 6 hours, or 2 hours on a Boat free trial
    (`docs/protocol/overflow.md:158-164` and
    `docs/protocol/app-removal.md:24-29`). The GitHub retirement step then
    refuses, so the job-end cleanup keeps the sandbox
    (`boat_authority.rs:85-90`).
  - The control-plane guide's status table says for 503: "The runtime
    retries saved cleanup" (`crates/ciproxy-control-plane/overflow.md:137`).
    Before the claim, nothing is saved. Its 401 row at `:136` omits a
    report after the end, which the claim also rejects (`cleanup.rs:30-33`).
  - Observed run, 21 November 2026: Boat `large` job 136802119867 on
    sandbox `bx_tn79cdzz`. The job and its locale, PostgreSQL, and Docker
    steps succeeded.
    - 09:27:58.869556Z: the control plane saved the completed job.
    - 09:27:59.621438Z: the guest's POST reached the load balancer.
    - 09:28:01.793Z: the check turned overflow off. This came after the
      POST, so it did not start the wait.
    - 09:28:04.921207Z: Cloud SQL cancelled a `pg_advisory_lock` wait of
      process 2341176 with a lock timeout.
    - The load balancer logged 503 after 5.351988 seconds.
    - 09:28:10.142605Z: the delivery ended with `reported_at` NULL. The
      provider now returns 404 for the sandbox.
  - The times above are log facts. The holder's process ID and the release
    time were not retained. That the job-end cleanup held the lock is an
    inference from the times and the code. Under it, the release came after
    the end at 09:28:10.14Z, when the cleanup finished its pass
    (`inventory.rs:200-209`). Each observed-run outcome below rests on it.
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
  - The store interface and the control-plane guide promise that the
    report is not lost. Neither promise holds before the claim.
- **Impact if no action:** A dropped report can keep a sandbox billing for
  up to 6 hours when its runner exits with no job, or after an App
  removal. In the no-job case, a waiting job of the same label can also
  stay stranded.
- **Root question:** How does an exit report take effect when other Boat
  work of the same workspace runs at the same moment?
- **Options:**
  - **A.** Let the report wait longer. Give only the report's lock session
    its own limit below the guest's 30-second client limit, for example
    25 seconds. A report behind a start, a refresh, or another report then
    takes effect. A report behind the job-end cleanup of its own delivery
    gets 401, not 503, because the claim needs `ends_at` NULL
    (`completion.rs:24-36`). So in the observed order, the claim is still
    lost. Each waiting report holds one more database connection, and a
    holder past the limit still drops the report. A also corrects
    `store.rs:44` and the 401 and 503 rows. Effort: Small.
  - **B.** Save the report as a durable claim first, without the lock.
    Then wait for the lock, and delete in the request when it comes.
    - Move the claim off the session interface, and update its doc line.
      The claim still accepts one report per token, only for a delivery
      that has not ended. So a first POST after the end still gets 401.
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
      already saved. A longer limit only makes a deferral rarer. After the
      wait ends, a later pass deletes, at no set time.
    - B also keeps a report behind a cleanup pass that ends several
      deliveries in turn, longer than any wait that fits the guest's limit.
    - In the observed run, B would have saved the claim at about
      09:27:59.9Z, before the end. That matters for options C and E of
      `plans/follow-ups/boat-live-completion-acceptance.md`: C keeps a claim
      for each case, and E for one new `large` run. B's main value is the
      report of a no-job exit and after an App removal.
    - Protocol: `docs/protocol/overflow.md:114`, `:117`, and `:160` must
      say that a saved report deletes the sandbox in its request when the
      lock comes within the wait. Otherwise a later pass deletes it.
    - B updates the 204, 401, and 503 rows and `store.rs:44`. It can also
      log the error class of a failed report, to support its validation.
    - Effort: Medium.
  - **C.** Delay the job-end deletion by a short grace time, for example
    30 seconds, while no report exists. A report that gets the lock within
    the grace can then claim before the job-end deletion. A first POST
    after the grace can still lose its claim. C does not stop drops behind
    other holders, such as a start or another report. It adds the grace
    time to the bill of each sandbox whose report never comes. It changes
    the timing of `docs/protocol/overflow.md:114`, so the protocol doc and
    the grace value need the user. Effort: Medium.
  - **D.** Retry the report in the guest until it gets 204, 400, or 401,
    or until a time limit ends. D also covers network errors. In the
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
      nothing. A first POST after the end still gets 401.
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
    - F changes the 204, 401, and 503 rows, `store.rs:44`, and the sentence
      "The store accepts it once for its one live runner"
      (`crates/ciproxy-overflow/README.md:121`). Effort: Medium.
  - **G.** Save the report as its own durable fact that no delivery save
    writes. An example is a received-at column that only the route sets,
    without the lock.
    - Only a lock holder turns that fact into the claim: the waiting report
      route, or a later cleanup pass. A report that arrived before the end
      also records on an ended delivery. A first POST after the end still
      gets 401.
    - So G defers the delete too. After the route's wait ends, a later pass
      applies the fact and deletes, at no set time.
    - The delivery row keeps one writer under the lock. So no save can drop
      the report, and no save needs the rule of B.
    - G needs a migration. The snapshot must read the new fact, so that the
      scaler stops counting the runner as ready. The cleanup needs a new
      step that applies the fact.
    - G needs the protocol sentence of B, and it updates the same rows and
      text. Effort: Large.
  - **H.** Do B, but try the lock once after the claim, and never wait.
    - The request deletes only when the lock is free. At each busy lock, a
      later pass deletes, at no set time.
    - H holds no database connection while the lock is busy, and it needs
      no lock limit. The save rule and the guard window are those of B.
    - In the observed run, H would also have saved the claim at about
      09:27:59.9Z, and the delete would have waited for a pass.
    - Protocol: `docs/protocol/overflow.md:114`, `:117`, and `:160` must
      say that a saved report deletes the sandbox at once only when no
      other Boat work holds the workspace. Otherwise a later pass deletes.
    - H updates the same rows and text as B. Effort: Medium.
- **Left out:**
  - Change nothing, because reports keep dropping behind each holder that
    runs longer than 5 seconds.
  - An image-only reporter, because D is better on every point. The guest
    already holds the token. Both need a new snapshot, and both end in 401
    in the observed order. D also covers network errors.
  - A load balancer retry of 503, because A is better on every point. The
    retry gives the longer wait of A, but it needs infrastructure and
    repeats a one-use POST.
  - A longer lock limit for every store connection, because it changes
    each waiting lock of the product and the tested 5-second policy
    (`connection_tests.rs:28-81` and `:102-122`).
  - Give the report priority over a running holder, because a holder
    cannot stop in the middle of a provider call.
- **Wider fix:** B. The report becomes a durable claim that no lock holder
  can drop, once each save keeps the claim and the first end. H is the
  same fix in its try-lock form. G is an equally structural form at Large
  effort, because no save writes its fact.
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
- **Recommendation:** B. It keeps each report whatever holds the lock,
  needs no migration, and deletes in the request more often than H. G best
  protects the delivery row against later writers, at Large effort.
- **Action:** ask the user, because the user asked to see the options
  before any product code change, and B changes the documented callback
  outcome.

Evidence of the observed run, outside Git: the folder `callback-resume/`
in `.context/ciproxy-mvp/live-2026-11-20/boat-runner-start-2026-11-21/`.
