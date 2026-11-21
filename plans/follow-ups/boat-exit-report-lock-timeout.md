# A busy workspace lock drops Boat exit reports

- **Severity:** High. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `conductor/boat-callback-diagnosis`, item 1
  of the read-only code and contract trace of the failed Boat `large` live
  check. A fresh options check on the same day corrected B and added F
  and G.
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
    `/hooks/boat/runner-exit`, with a token that works only for its own
    sandbox.
  - **Claim:** the store step that accepts an exit report once. It saves
    `reported_at` and the `Deleting` phase, in
    `crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`.
  - **Workspace lock:** the PostgreSQL advisory lock that each overflow
    operation of one workspace holds for its whole run, in
    `crates/ciproxy-store-pg/src/overflow/queries.rs:24-76`.
- **Background:**
  - Each overflow operation of a workspace holds the workspace lock for
    its whole run, also during Boat, GitHub, and key calls. The operations
    are start, resume, view refresh, connect, disconnect, cleanup, and the
    exit report.
  - All operations except the exit report use a try-lock. They fail at
    once with `Busy` (`crates/ciproxy-overflow/src/session.rs:22-31`).
  - The exit report waits for the lock
    (`crates/ciproxy-overflow/src/service/cleanup.rs:28`). The store
    interface gives the reason: "Wait for the workspace session so a guest
    callback is never lost as busy"
    (`crates/ciproxy-store-interface/src/overflow/store.rs:44`).
  - The wait runs `SELECT pg_advisory_lock($1)` on a new store connection
    (`queries.rs:39-49`). That connection sets `lock_timeout = 5000`
    (`crates/ciproxy-store-pg/src/connector.rs:79-83`). PostgreSQL applies
    this limit to advisory lock waits too.
  - The waiting organization lease uses the same pattern, and a store test
    expects its lock timeout
    (`crates/ciproxy-store-pg/src/github/lease.rs:16-27` and
    `crates/ciproxy-store-pg/tests/store/connection_tests.rs:102-122`).
  - The claim comes only after the lock (`cleanup.rs:28-33`). So a
    cancelled wait saves nothing. The route maps that error to 503 and
    logs nothing
    (`crates/ciproxy-control-plane/src/boat_callback.rs:41-45`).
  - The claim exists only on the session interface, whose connection holds
    the lock (`store.rs:50-52` and `:95-101`). It accepts a report only for
    a delivery that has not ended when the claim runs (`completion.rs:29`).
    The crate README states that rule, not a protocol doc
    (`crates/ciproxy-overflow/README.md:120-121`).
  - Each delivery save writes `record_json`, `reported_at`, and `ends_at`
    from the caller's own copy of the record. Each write of a delivery row
    goes through that one function
    (`crates/ciproxy-store-pg/src/overflow/deliveries.rs:55-66`). The
    snapshot reads the phase and `reported_at` from `record_json`
    (`crates/ciproxy-store-pg/src/overflow/rows.rs:88-96`).
  - A view refresh saves each running delivery from its own snapshot copy
    (`crates/ciproxy-overflow/src/service/refresh.rs:86-99`,
    `crates/ciproxy-overflow/src/service/usage.rs:17-44` and `:121`, and
    `crates/ciproxy-overflow/src/service/costs.rs:13-28`). The cleanup
    saves its copy too
    (`crates/ciproxy-overflow/src/service/inventory.rs:195-199` and
    `costs.rs:41-50`).
  - A stale save that restores `Running` makes the next pass ask GitHub
    again, and a waiting job then keeps the sandbox (`inventory.rs:182-189`
    and `crates/ciproxy-control-plane/src/boat_snapshot.rs:58-60`).
  - The guest makes one POST with a 30-second client limit and does not
    try again. A failure gives one warning, and then the guest ends
    (`crates/ciproxy-guest/src/boat.rs:55-60` and `:81-95`,
    `crates/ciproxy-guest/src/service.rs:45-49` and `:156-159`). The guest
    unit has `Restart=no` (`image/ciproxy-boat-guest.service:11`). A test
    protects the single attempt
    (`crates/ciproxy-guest/src/_tests_/service_tests.rs:140`).
  - A completed GitHub job starts the job-end cleanup at once
    (`crates/ciproxy-control-plane/src/boat_jobs.rs:45-49`). The 10-second
    pass starts the same cleanup
    (`crates/ciproxy-control-plane/src/listeners.rs:132-144` and
    `crates/ciproxy-control-plane/src/boat_runtime.rs:35-38`).
  - The job-end cleanup takes the lock with the try-lock. It keeps the
    lock while it asks GitHub to remove the runner registration, measures
    runtime, and deletes the sandbox (`inventory.rs:16-17` and `:182-202`,
    `cleanup.rs:45-80`). It does not read or wait for the exit report.
  - The protocol permits that order. It requires deletion when the job
    ends. It makes the exit report a fast trigger and the sandbox lifetime
    the backstop (`docs/protocol/overflow.md:114-129`). No protocol line
    requires a recorded claim for each successful job.
  - For a runner that exits with no job, the protocol requires deletion at
    once on the exit report (`docs/protocol/overflow.md:115-118`). Without
    a report, the runner counts as ready, and a waiting job of its label
    keeps the sandbox (`boat_snapshot.rs:58-60` and `inventory.rs:189`).
  - After an App removal, a started sandbox ends only at its exit report or
    at its lifetime. The lifetime is 6 hours, or 2 hours on a Boat free
    trial (`docs/protocol/overflow.md:158-164` and
    `docs/protocol/app-removal.md:24-29`). The GitHub retirement step then
    refuses, so the job-end cleanup keeps the sandbox
    (`crates/ciproxy-control-plane/src/boat_authority.rs:85-90`).
  - Two docs promise more than the code gives. `store.rs:44` says that a
    waiting report is never lost as busy. The status table of the
    control-plane guide says for 503: "The runtime retries saved cleanup"
    (`crates/ciproxy-control-plane/overflow.md:132-137`). Before the
    claim, nothing is saved.
  - The 401 row of that table does not name a report that arrives after
    its delivery ended, which the claim also rejects
    (`crates/ciproxy-control-plane/overflow.md:136` and
    `cleanup.rs:30-33`).
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
  - The process ID of the lock holder was not retained. That the job-end
    cleanup held the lock is an inference from the times and the code. It
    is not a log fact.
  - Without the 5-second limit, the report in this order would still
    lose. It would get the lock after the delivery ended. The claim
    accepts only a delivery with `ends_at` NULL, so the route would return
    401 (`completion.rs:24-36`).
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
    gets 401 instead of 503, so the claim is still lost. Each waiting
    report holds one more database connection. A holder that runs longer
    than the limit still drops the report. A also corrects the text at
    `store.rs:44`, and the 401 and 503 rows of the guide's status table.
    Effort: Small.
  - **B.** Save the report as a durable claim first, without the workspace
    lock.
    - Move the claim off the session interface, and update its doc line.
      The claim still accepts one report per token, and only for a
      delivery that has not ended.
    - Each save must keep both claim facts, `reported_at` and the
      `Deleting` phase, against every stale copy. The one save function
      can hold that rule.
    - The retirement guard compares `reported_at`
      (`crates/ciproxy-store-interface/src/boat_delivery.rs:47-48` and
      `crates/ciproxy-store-interface/src/runtime_results.rs:74-75`). The
      retirement reads that guard before its GitHub call, and it writes its
      result only on an equal guard (`boat_authority.rs:151-158` and
      `:195-212`, `crates/ciproxy-store-pg/src/boat_results.rs:44-47`).
      So a claim during a cleanup gives `RunnerNeeded`. The next pass then
      deletes the `Deleting` delivery (`inventory.rs:87-97` and
      `:190-193`). This needs a test.
    - After the claim, the route can wait for the lock and delete at once.
      It can also use a try-lock and leave the delete to the next 10-second
      pass. Either form is an implementation choice, not a requirement. The
      try-lock form needs no new lock limit.
    - A holder that read its snapshot before the claim keeps a stale
      `Running` copy. Only the job-end cleanup of the same delivery deletes
      the sandbox in that pass. Otherwise the next pass deletes it.
    - B also keeps a report behind a cleanup pass that ends several
      deliveries in turn. Such a pass can hold the lock longer than any wait
      that fits the guest's 30-second limit.
    - In the observed run, B would have saved the claim at about
      09:27:59.9Z, before the end. That matters only if the live acceptance
      keeps a claim for each run, as options C and E of
      `plans/follow-ups/boat-live-completion-acceptance.md` do. B's main
      value is the report of a no-job exit and after an App removal.
    - B updates the 204, 401, and 503 rows of the guide's status table and
      the text at `store.rs:44`. The change can also log the error class of
      a failed report, to support its validation.
    - Effort: Medium.
  - **C.** Delay the job-end deletion by a short grace time, for example
    30 seconds, while no report exists. The report then has time to claim
    before the job-end deletion. C does not stop drops behind other
    holders, such as a start or another report. It adds the grace time to
    the bill of each sandbox whose report never comes. It changes the
    timing of `docs/protocol/overflow.md:114`, so the protocol doc and the
    grace value need the user. Effort: Medium.
  - **D.** Retry the report in the guest until it gets 204, 400, or 401,
    or until a time limit ends. D also covers network errors. In the
    observed run, the retries end with 401 or with the deleted sandbox, so
    the claim is still lost. D needs a new guest, an agent release, and a
    new Boat snapshot build. It also changes the tested single attempt and
    `crates/ciproxy-guest/README.md:58-62`. Effort: Large.
  - **E.** Hold the workspace lock only around store writes, not during
    Boat, GitHub, and key calls. This applies to start, cleanup, refresh,
    and the exit report. Long holds then end for each operation.
    Per-delivery compare-and-swap writes are one form of E. E redesigns
    the one-lock model of `crates/ciproxy-overflow/README.md:40` and adds
    races between provider calls and saves. Effort: Large.
  - **F.** Judge the report at its arrival.
    - Read the trusted time before the lock wait. Give only the report's
      session the longer limit of A.
    - Accept the claim also for a delivery that ended after that time.
      Such a late claim saves only `reported_at`. It keeps `Ended` and
      deletes nothing.
    - In the observed order, the claim runs after the holder ends at
      09:28:10.14Z. It records `reported_at` of about 09:27:59.9Z, and the
      route returns 204 within the guest's 30-second limit.
    - No delivery save changes, so F adds no concurrent write.
    - A holder that runs past the limit, or a restart during the wait,
      still drops the report, as in A. So F protects the no-job and
      App-removal cases no better than A.
    - F changes the 204, 401, and 503 rows of the guide's status table and
      the text at `store.rs:44`. It also changes the sentence "The store
      accepts it once for its one live runner"
      (`crates/ciproxy-overflow/README.md:121`).
    - Effort: Medium.
  - **G.** Save the report as its own durable fact that no delivery save
    writes.
    - An example is a received-at column that only the route sets, without
      the workspace lock.
    - Only a lock holder turns that fact into the claim: the waiting report
      route, or the next cleanup pass. A report that arrived before the end
      also records on an ended delivery.
    - The delivery row keeps one writer under the lock. So no save can drop
      the report, and no save needs the rule of B.
    - G needs a migration. The snapshot must read the new fact, so that the
      scaler stops counting the runner as ready. The cleanup needs a new
      step that applies the fact.
    - G also updates the 204, 401, and 503 rows of the guide's status table
      and the text at `store.rs:44`.
    - Effort: Large.
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
    cannot stop in the middle of a provider call. The report still waits
    behind it.
- **Wider fix:** B. The report becomes a durable claim that no lock holder
  can drop, once each save keeps both claim facts. G is an equally
  structural form at Large effort, because no save writes its fact.
- **Previous examples:** None found of this cause.
  `plans/follow-ups/boat-launched-runner-readiness.md` lists "a failed
  report" as one cause of a stranded job, with no cause named.
- **Related:**
  - `plans/follow-ups/boat-live-completion-acceptance.md`: its options C
    and E keep a claim rule, so they wait for B, C, F, or G of this item
    and their deploy. Only B, C, F, or G records the claim in the observed
    order. Its option F uses earlier accepted reports, which this item
    does not change.
  - `plans/follow-ups/boat-launched-runner-readiness.md`: the same class,
    where a lost report strands a waiting job. Its ready deadline is a
    backstop for each lost report, not a fix of this cause.
- **Recommendation:** B. It is the only Medium option that keeps each
  report whatever holds the lock, and it needs no migration. G best
  protects the delivery row against later writers, at Large effort.
- **Action:** ask the user, because the user asked to see the options
  before any product code change, and B changes the documented callback
  outcome.

Evidence of the observed run, outside Git: the folder `callback-resume/`
in `.context/ciproxy-mvp/live-2026-11-20/boat-runner-start-2026-11-21/`.
