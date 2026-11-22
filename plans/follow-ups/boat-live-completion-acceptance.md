# Live Boat acceptance requires an unpromised report claim

- **Severity:** High. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-21, from `conductor/boat-callback-diagnosis`, item 2
  of the read-only code and contract trace of the failed Boat `large` live
  check. A fresh options check on the same day added options E, F, and G,
  and changed the recommendation from B to F. Two reviews then corrected
  options B to G.
- **Source:** Review.
- **Category:** Test.
- **Location:** `plans/ciproxy-mvp.md:2827-2830`, `:2888-2890`, and
  `:2910-2913`.
- **Reason:** Main has merged. F's live runs are approved and remain pending.
  They do not wait for this PR to merge. No result is recorded yet.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Exit report:** the HTTPS POST of `RunnerExited` to
    `/hooks/boat/runner-exit` that the guest, the ciproxy program in the
    sandbox, sends once when its runner exits.
  - **Claim:** the stored acceptance of one exit report for a delivery,
    the record of one VM and runner. It saves `reported_at` in
    `crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`.
  - **Live check:** the paid Boat checks of the Boat runner-start task: the
    `large` job, the overflow smoke, and the spend-limit smoke. Its scripts
    are outside the repository.
  - **Orchestrator:** the agent session that runs this Boat task and owns
    its run order and its budget of six paid sandboxes. It is not the user.
- **Background:**
  - The live check passes a case only when its delivery ended in the phase
    `ended`, with `reported_at` and `ends_at` set. The same rule applies to
    the `large` case and to the two smoke cases. This rule comes from the
    live check scripts, not from the plan.
  - The plan ties the callback to the large job. It asks to "Pass the Boat
    large job, its exit callback, its PostgreSQL and Docker steps, the
    overflow smoke, and the complete spend-limit smoke"
    (`plans/ciproxy-mvp.md:2827-2830`). Its newest Boat milestone asks to
    "Pass the actual large job, callback, PostgreSQL and Docker steps,
    job-log locale check, overflow smoke and complete spend-limit smoke"
    (`:2888-2890`).
  - The user's answer of 21 November 2026 says to merge into `main` now
    and to run the remaining Boat smokes in parallel. It lists "the Boat
    exit callback" among the checks that remain (`:2910-2913`).
  - The protocol requires that "ciproxy stops and deletes each sandbox when
    its job ends" (`docs/protocol/overflow.md:114`). It also requires that
    the guest report its runner exit, and that the control plane then
    delete the sandbox at once (`:115-118`). The exit report is the fast
    trigger, and the sandbox lifetime is the backstop (`:124-129`).
  - The job-end cleanup is the cleanup pass that deletes a sandbox after
    GitHub reports its job as completed
    (`crates/ciproxy-control-plane/src/boat_jobs.rs:45-49`). It and the exit
    report both need the workspace lock. That is the PostgreSQL advisory
    lock that each overflow operation of a workspace holds for its whole
    run (`crates/ciproxy-store-pg/src/overflow/queries.rs:24-76`).
  - No protocol line says which trigger comes first, or that a claim is
    recorded for each successful job. So the design permits the job-end
    cleanup to end a successful job's sandbox before any claim.
  - The repository's overflow smoke already passes on that job-end path.
    It needs a successful job on the matching runner, an ended delivery,
    and a provider deletion. It reads no claim
    (`crates/ciproxy-control-plane/scripts/overflow-smoke.py:53-69`).
  - In its `idle-delete` scenario, ciproxy's own cleanup removes the
    registration and deletes the sandbox while it holds the workspace lock
    (`overflow-smoke.py:48-51` and
    `crates/ciproxy-overflow/src/service/inventory.rs:182-202`). So that
    runner exits because of the cleanup, and its report races the cleanup.
  - The image's Boat smoke guide asks for the callback receipt and the
    provider deletion as evidence (`image/boat-smoke.md:230-234`). That
    guide covers a manual smoke with a hand-made registration.
  - The route returns 401 for a missing or unknown token, and for a claim
    that finds no live delivery. It returns 503 for each other error
    (`crates/ciproxy-control-plane/src/boat_callback.rs:27-45` and
    `crates/ciproxy-overflow/src/service/cleanup.rs:19-43`). A 503 can come
    before the claim, with nothing saved, or after a saved claim. After a
    saved claim, `reported_at` is set, and the next pass retries the
    deletion (`inventory.rs:87-97`).
  - Guest 0.1.4 logs `runner exit report accepted` after `error_for_status`
    passes (`crates/ciproxy-guest/src/boat.rs:91-92`). In the pinned
    `reqwest` 0.13.5, that check rejects only 4xx and 5xx answers. So a
    3xx, or a nonstandard code from 600 to 999, also logs the line. The
    guest follows no redirect (`boat.rs:57`).
  - A saved claim can also exist without that line. The claim commits in
    its own transaction (`completion.rs:23`). A later step of the same
    request can then fail with 503 (`cleanup.rs:34-42` and `:71`). The
    guest then only warns (`crates/ciproxy-guest/src/service.rs:45-49`).
    The route's delete can also stop the sandbox before the guest logs the
    line. The journal goes with the sandbox (`image/boat-smoke.md:227`).
  - So the line is no receipt, and no option reads it. The receipt is the
    saved claim, `reported_at`, read from persisted state.
  - Observed `large` run, 21 November 2026: job 136802119867 on sandbox
    `bx_tn79cdzz` succeeded. Its locale, PostgreSQL, and Docker steps
    passed, with no setlocale warning.
  - One POST to the exit route arrived at 09:27:59.621438Z. It was the
    only POST to the route in a four-minute query. It got 503 after
    5.351988 seconds, and a database lock-timeout line came at 5.30 seconds.
    That fits a 5-second limit on the report's wait for the workspace lock.
    Its timing strongly fits this case's guest. The retained entry itself
    does not name the sender.
  - The delivery ended at 09:28:10.142605Z with `reported_at` NULL, and the
    provider now returns 404 for the sandbox. The job-end cleanup is the
    code path that fits this saved state. The process ID of the lock
    holder and the release time were not retained, so the job-end cleanup
    as holder is an inference.
  - So the live check recorded FAIL with "The paid delivery did not record
    the runner exit callback and end". That FAIL stays recorded.
  - Two earlier exit reports of the same day were accepted:
    - Sandbox `bx_748nk3ay`: `reported_at` 03:09:56.341416Z, end
      03:10:03.403811Z, provider GET 404.
    - Sandbox `bx_gbsxpayp`: `reported_at` 03:18:24.255918Z, end
      03:18:30.849299Z, provider GET 404.
    - Their runners exited before a job, so each accepted report ended its
      sandbox. `plans/follow-ups/boat-launched-runner-readiness.md:91-95`
      also records these reports.
    - Their control plane was deploy `bf2da45e`. Its crates match the
      crates of `main` at `3bdc4777`. They ran the same guest, release
      0.1.4 at `b751ef5f`, on an older snapshot with the same report path.
    - Both cases failed their checks for other causes. Those FAILs stay.
  - Four of the six paid sandbox slots that the orchestrator allowed are
    used, and each of the four sandboxes now returns 404. The last two are
    for the overflow smoke and the complete spend-limit smoke, once each,
    with no retry. At the end, every task sandbox must be deleted, and
    overflow must be off in both workspaces. The main machine in Firna-AI
    must be Ready again on agent 0.1.4.
- **Problem:**
  - For a successful job, the job-end cleanup and the exit report compete
    for the workspace lock, and the code gives neither side priority. When
    the cleanup takes the lock first and ends the delivery, the claim is
    lost.
  - The one observed run ended with no claim, so the check failed while
    the product met the protocol. Its lock holder is an inference, because
    the holder's process ID was not retained.
  - One claim rule covers all three cases, but the plan ties the callback
    only to the large job.
- **Impact if no action:** The two held smoke cases can fail the same way
  and use the last two paid sandbox slots. The Boat live checks then stay
  blocked.
- **Root question:** What must the remaining live Boat checks prove about
  the exit report and the end of each sandbox?
- **Options:** "Evidence levels" below defines the report evidence that
  options B, D, E, F, and G use.
  - **A.** Pass a successful job on the job-end path. Require GitHub
    success, the matching runner, an ended delivery, and a provider GET
    404. Keep `reported_at` as evidence only. A needs no product change. A
    broken exit report route then passes unseen for successful jobs.
    Effort: Small.
  - **B.** Do A, and also require route activity: a POST to
    `/hooks/boat/runner-exit` in the delivery's time window, in the load
    balancer log. Record its status. When the case runs alone, the POST
    reaches the isolated-window level. B shows no valid token, no matched
    runner, and no saved claim. It needs load balancer request logs with a
    sample rate of 1, as in the observed run. Effort: Small.
  - **C.** Keep the current rule, which requires a saved claim for every
    case. Run the held cases only after a product change that records the
    claim in the observed order, and after its deploy. That change is
    option B, C, F, G, or H of
    `plans/follow-ups/boat-exit-report-lock-timeout.md`.
    - C can still fail. Under B, F, G, or H there, a first POST after the
      job-end cleanup ended the delivery records no claim. Under C there, a
      POST after the grace or behind another long holder can record none.
      Under F there, the claim also needs the lock within the wait limit.
    - The observed POST came 0.75 seconds after the completed job.
    - A new `large` run needs a paid sandbox past the six-sandbox limit,
      and each failed run needs one more. Each needs a new call of the
      orchestrator.
    - Effort: Medium, plus the product change.
  - **D.** Prove the exit report where it is the only deletion trigger.
    That is a launched sandbox whose runner exits with no job, while no
    other Boat work runs. Make that exit with a cause outside ciproxy. An
    example is a stop of the runner process through the Boat command API
    while no job is assigned. The `idle-delete` scenario does not fit,
    because ciproxy's own cleanup causes that exit and races the report.
    That case passes at the saved-claim level. Use A or B for the
    successful jobs. D needs one more paid sandbox past the current limit,
    so it needs a new call of the orchestrator. Effort: Medium.
  - **E.** Keep the claim rule only for the exit callback of the large
    job, as one check of its own.
    - Pass the two held smokes now on the rule of B, while the large
      callback check stays open and FAIL.
    - Close that check only when a new successful `large` job and the
      saved claim of its own delivery are recorded together. That run
      comes after option B, C, F, G, or H of
      `plans/follow-ups/boat-exit-report-lock-timeout.md` is deployed.
    - That run can still lose its claim, as C above says. Under F there,
      it also loses it when the lock does not come free within the wait
      limit.
    - E keeps the plan's requirement of a large job with the claim of its
      own delivery. It weakens the claim rule of the live check for the two
      smokes, which then need only the route activity of B. C keeps every
      claim rule of the live check.
    - The milestone stays open. The callback run needs one paid sandbox
      past the six-sandbox limit, and each failed run needs one more. Each
      needs a new call of the orchestrator.
    - Effort: Medium, plus the product change.
  - **F.** Do B for the two held smokes. Prove the exit callback with the
    two accepted reports of `bx_748nk3ay` and `bx_gbsxpayp` above.
    - Record the choice and a new run record. The original FAILs stay.
    - F proves what B cannot: the deployed control plane accepts a real
      guest's report and then deletes the sandbox. Those reports have saved
      claims. The held smokes reach only the level of B.
    - F does not prove a claim after a successful job.
      `plans/follow-ups/boat-exit-report-lock-timeout.md` shows that the
      product does not give that claim in the observed order.
    - Its evidence comes from cases that failed for other causes, on an
      older snapshot with the same report path.
    - Effort: Small.
  - **G.** Do B, and accept only a status that the race can explain. G
    compares the POST's arrival and response, the delivery end, the
    lock-timeout line, and its statement form, where each exists.
    - Accept 204.
    - Accept a 503 after a saved claim only when `reported_at` is set.
    - Accept a 503 before a claim only with its own lock-timeout line on
      `SELECT pg_advisory_lock($1)`. The line must fall between the arrival
      plus 5 seconds and the response, and the latency must be 5 seconds
      or more. In product code, only the waiting exit report sends that
      statement (`crates/ciproxy-overflow/src/session.rs:23-24` and
      `crates/ciproxy-store-pg/src/overflow/queries.rs:39-43`). The
      organization lease sends a form with two keys
      (`crates/ciproxy-store-pg/src/github/lease.rs:21`). The migration
      lock sends a literal key (`crates/ciproxy-store-pg/src/factory.rs:24`).
    - Accept 401 only when the POST arrived before the end and the response
      came after it. Each other status or order fails the case.
    - The observed run meets the time and latency parts. Its line came 5.30
      seconds after the arrival, and its response at 5.35 seconds. The safe
      files do not keep the statement text, so its form is unknown.
    - False positives remain. No request ID ties the load balancer line,
      the route, and the database line. The line keeps no lock key, so
      another exit report can time out in the same seconds. Timing alone
      proves no valid token and no matched runner. Another delay before the
      token check can also push a 401 past the end. In a run that loses the
      race, a wrong runner ID also passes. Option F, or a product option,
      must show the claim.
    - G fails a status that the race cannot explain, which A and B accept.
      So it narrows the false positive of the isolated window. It needs
      Cloud SQL logs in the live check and a small clock tolerance between
      the two logs. G puts no check in force.
    - Effort: Medium.
- **Left out:**
  - Change nothing and run the held cases, because successful jobs can
    meet the same race and use the last paid slots.
  - Mark a recorded FAIL as a PASS, because each run failed the rule that
    it had. The new rule applies only to new runs, in a new run record.
    F cites facts of the old runs and marks no FAIL as a PASS.
- **Wider fix:** None at the check boundary. The chosen B of the lock record
  in Related removes the race from the claim itself.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/boat-exit-report-lock-timeout.md`: the user chose B
    there, and a separate workspace builds it. F uses earlier evidence of
    the report path, so it waits for no B deploy. The unchosen C and E
    would wait for B, C, F, G, or H there and their deploy.
  - `plans/follow-ups/boat-exit-report-response-log.md`: it corrects the
    guest's log line that the Background describes. No option here waits
    for it.
- **Recommendation:** F. The first recommendation was B. F has the cost of
  B and needs no paid sandbox beyond the last two held slots. It also
  proves that the deployed control plane accepts a real guest's report
  where that report alone ends the sandbox. Before the answer, B was the
  fallback if the user rejected that evidence. E keeps the plan's
  large-job and own-claim requirement, but it weakens the live check's
  claim rule for the two smokes. C best protects the checks, because it
  keeps that rule for every case. A sandbox past the limit still needs
  the orchestrator's authorization.
- **User answer, 2026-11-22:** F. Not chosen: A, B alone, C, D, E, and G.
  The two held smokes no longer need their own `reported_at`. Each needs a
  POST to the exit route in its delivery window, with its status, in the
  load balancer log. It also needs GitHub job success, the matching
  runner, an ended delivery, and a Boat GET 404. The two earlier saved
  claims prove the report path. No recorded FAIL becomes a PASS.
- **Action:** fix now, apply F to the two one-shot live smokes, as the user
  chose on 22 November 2026. Keep both records as the user requested.

## Evidence levels

Each level names what the evidence of one case shows. A time match is no
sender match.

| Level | Evidence | Supports | Does not prove |
| --- | --- | --- | --- |
| Route activity | A POST to `/hooks/boat/runner-exit` in the case's delivery window, in the load balancer log | That some client posted in that window | Which sandbox sent it, a valid token, a matched runner, or a claim |
| Isolated window | Route activity while no other delivery of this control plane is live. A delivery is live until its `ends_at` is set | Attribution to this case's guest, by inference | That this guest sent it. The route is public, so a POST that no guest sent can match |
| Trusted delivery link | A server observation after token and runner verification that ties the POST to one delivery | That the request matches this delivery | Which process sent it, or a saved claim. The observed run retained no such field |
| Saved claim | `reported_at` on the case's delivery, in persisted state | The receipt: the control plane accepted the report | That the report ended the sandbox. `ends_at` and the provider's 404 show the end |

- No timing match proves a valid token or a matched runner.
- The retained entry of the observed run keeps no source address, trace,
  delivery, or runner identity.
- In a later run, a match of a logged source address to the sandbox's
  provider address is only a candidate. It holds only if Boat gives each
  sandbox its own egress address. Nothing shows that, so this record claims
  no such match.
- The task allowed the checks beside the merge into main. The orchestrator
  owns their run order and may run the two held smokes one after the other.
  That supports the isolated-window inference and creates no new gate.

Evidence outside Git: the observed run and the orchestrator's check of the
two earlier accepted reports, in the folder `callback-resume/` in
`.context/ciproxy-mvp/live-2026-11-20/boat-runner-start-2026-11-21/`.
