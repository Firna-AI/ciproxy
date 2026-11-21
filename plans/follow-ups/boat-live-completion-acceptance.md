# Live Boat acceptance requires an unpromised report claim

- **Severity:** High. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `conductor/boat-callback-diagnosis`, item 2
  of the read-only code and contract trace of the failed Boat `large` live
  check. A fresh options check on the same day added options E, F, and G,
  and changed the recommendation from B to F.
- **Source:** Review.
- **Category:** Test.
- **Location:** `plans/ciproxy-mvp.md:2827-2830`, `:2888-2890`, and
  `:2910-2913`.
- **Reason:** The user asked to see the options before any acceptance
  change, and the last two paid sandbox slots wait for the answer.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Exit report:** the HTTPS POST of `RunnerExited` to
    `/hooks/boat/runner-exit` that the ciproxy guest in the sandbox sends
    once when its runner exits.
  - **Claim:** the store step that accepts an exit report once and saves
    `reported_at`, in
    `crates/ciproxy-store-pg/src/overflow/completion.rs:16-44`.
  - **Job-end cleanup:** the cleanup pass that deletes a sandbox after
    GitHub reports its job as completed. It starts at
    `crates/ciproxy-control-plane/src/boat_jobs.rs:45-49`.
  - **Live check:** the paid Boat checks of the Boat runner-start task: the
    `large` job, the overflow smoke, and the spend-limit smoke. Its scripts
    are outside the repository.
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
  - The guest logs `runner exit report accepted` only after a 2xx answer
    (`crates/ciproxy-guest/src/boat.rs:91-92`). Its journal goes with the
    deleted sandbox (`image/boat-smoke.md:227`).
  - Each overflow operation of a workspace holds one PostgreSQL advisory
    lock, the workspace lock, for its whole run. The exit report and the
    job-end cleanup both need it
    (`crates/ciproxy-store-pg/src/overflow/queries.rs:24-76`).
  - Observed `large` run, 21 November 2026: job 136802119867 on sandbox
    `bx_tn79cdzz` succeeded. Its locale, PostgreSQL, and Docker steps
    passed, with no setlocale warning.
  - The guest posted its exit report at 09:27:59.621438Z. It got 503 after
    5.351988 seconds, because a 5-second database lock limit ended its
    wait for the workspace lock.
  - The delivery ended at 09:28:10.142605Z with `reported_at` NULL, and the
    provider now returns 404 for the sandbox. The job-end cleanup is the
    code path that fits this saved state. The process ID of the lock
    holder was not retained, so the job-end cleanup as holder is an
    inference.
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
    - Both cases failed their own checks for other causes. Those FAILs
      stay.
  - Four of the six paid sandbox slots that the orchestrator allowed are
    used, and each of the four sandboxes now returns 404. The last two
    slots are for the overflow smoke and the spend-limit smoke. They stay
    unused until this item has an answer.
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
- **Options:**
  - **A.** Pass a successful job on the job-end path. Require GitHub
    success, the matching runner, an ended delivery, and a provider GET
    404. Keep `reported_at` as evidence only. A needs no product change. A
    broken exit report route then passes unseen for successful jobs.
    Effort: Small.
  - **B.** Do A, and also require that the load balancer log shows the
    guest's POST to `/hooks/boat/runner-exit` in the delivery's time
    window. Record its status. B proves that the guest sent its report
    after a normal runner exit. It does not prove that the control plane
    accepted the report. It needs load balancer request logs with a
    sample rate of 1, as in the observed run. Effort: Small.
  - **C.** Keep the current rule. Run the held cases only after a product
    change that records the claim in the observed order, and after its
    deploy. That change is option B, C, F, or G of
    `plans/follow-ups/boat-exit-report-lock-timeout.md`. C can still fail
    in an order that those options do not change. When the job-end cleanup
    ends the delivery before the guest posts, no claim records. In the
    observed run, the guest posted 0.75 seconds after the completed job.
    A new `large` run also needs a paid sandbox past the six-sandbox limit,
    so it needs a new call of the orchestrator. Effort: Medium, plus the
    product change.
  - **D.** Prove the exit report where it is the only deletion trigger.
    That is a launched sandbox whose runner exits with no job, while no
    other Boat work runs. Make that exit with a cause outside ciproxy. An
    example is a stop of the runner process through the Boat command API
    while no job is assigned. The `idle-delete` scenario does not fit,
    because ciproxy's own cleanup causes that exit and races the report.
    Use A or B for the successful jobs. D needs one more paid sandbox past
    the current limit, so it needs a new call of the orchestrator.
    Effort: Medium.
  - **E.** Keep the claim rule only for the exit callback, as one check of
    its own.
    - Pass the two held smokes now on the rule of B.
    - The callback check stays FAIL and open until a run records a claim.
      That run comes after option B, C, F, or G of
      `plans/follow-ups/boat-exit-report-lock-timeout.md` is deployed.
    - E relaxes no requirement of the plan, and the milestone stays open.
      The callback run needs one paid sandbox past the six-sandbox limit,
      so it needs a new call of the orchestrator.
    - Effort: Medium, plus the product change.
  - **F.** Do B for the two held smokes. Prove the exit callback with the
    two accepted reports of `bx_748nk3ay` and `bx_gbsxpayp` above.
    - Record the choice and a new run record. The original FAILs stay.
    - F proves what B cannot: the deployed control plane accepts a real
      guest's report and then deletes the sandbox.
    - F does not prove a claim after a successful job.
      `plans/follow-ups/boat-exit-report-lock-timeout.md` shows that the
      product does not give that claim in the observed order.
    - Its evidence comes from cases that failed for other causes, on an
      older snapshot with the same report path.
    - Effort: Small.
  - **G.** Do B, and accept only an explained status.
    - Accept 204. Accept 503 only with a database lock-timeout line in the
      same seconds. Accept 401 only after the end of the delivery. Each
      other status fails the case.
    - G catches a broken route in each run, which A and B miss.
    - G reads Cloud SQL logs in the live check, as this diagnosis did.
    - Effort: Medium.
- **Left out:**
  - Change nothing and run the held cases, because successful jobs can
    meet the same race and use the last paid slots.
  - Mark a recorded FAIL as a PASS, because each run failed the rule that
    it had. A new rule needs the orchestrator's recorded choice and a new
    run record. F does not mark a FAIL as a PASS. It cites facts of those
    runs in a new record.
- **Wider fix:** None at the check boundary. Option B of
  `plans/follow-ups/boat-exit-report-lock-timeout.md` removes the race from
  the claim itself.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/boat-exit-report-lock-timeout.md`. Options
  C and E of this item wait for its option B, C, F, or G and their deploy.
  Options A, B, F, and G of this item relax the claim for each case that C
  keeps, so they conflict with C. Option F of this item uses earlier
  evidence of the report path, which that item does not change.
- **Recommendation:** F. The first recommendation was B. F has the cost of
  B and needs no paid sandbox beyond the last two held slots. It also
  proves that the deployed control plane accepts a real guest's report
  where that report alone ends the sandbox. B is the fallback if the user
  rejects evidence from those earlier cases. E is the option that relaxes
  nothing.
- **Action:** ask the user, because F changes the acceptance rule that the
  plan records, and the user asked to see the options before any
  acceptance change.

Evidence of the observed run, outside Git: the folder `callback-resume/`
in `.context/ciproxy-mvp/live-2026-11-20/boat-runner-start-2026-11-21/`.
That folder also holds the orchestrator's check of the two earlier
accepted reports.
