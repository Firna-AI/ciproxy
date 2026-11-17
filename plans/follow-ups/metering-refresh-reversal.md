# Older refresh leaves duplicate starts

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 401.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-metering/src/service.rs:100`,
  `crates/ciproxy-metering/src/report/runtime.rs:72`,
  `crates/ciproxy-metering/src/report/runtime.rs:106`,
  `crates/ciproxy-metering/src/report/runtime.rs:133`, and
  `crates/ciproxy-metering/src/report/boat.rs:52`.
- **Reason:** The race needs two refreshes on either side of UTC midnight,
  and no production data holds metering counts yet.

- **Terms:**
  - **Start:** one job that a ciproxy runner takes. Usage counts the starts
    of each day and of each billing period.
  - **Alias:** a second metering record of the same job. A scale-set
    attempt is known before GitHub gives its numeric job, so one job can
    have two records.
  - **Cursor:** the saved accounting record of one job. It keeps the facts
    and the observation time of its last applied contribution, so that a
    refresh can reverse that contribution.
  - **Refresh:** `refresh` at `crates/ciproxy-metering/src/service.rs:89`.
    It applies signed corrections to the daily totals of a workspace.
  - **Retention start:** the first of the 90 UTC dates that metering keeps.
    Refresh drops the older dates.
- **Background:**
  - Metering keeps daily totals of starts and runtime for each workspace.
    The Usage and Savings pages show them.
  - For an ordinary job, the new observation time is the later of the
    refresh time and the time of the cursor, at `service.rs:119-123`. The
    correction range ends the day after that time, at `:124`. So an older
    refresh still corrects every date through the later time of the cursor.
  - `crates/ciproxy-metering/README.md:65-66` states this rule: "Each new
    observation preserves the stored cursor's later observation time.
    Corrections include every affected UTC date through that time."
  - The Boat cost loop uses the same rule, at `service.rs:147-156`.
  - Alias retirement does not. When GitHub gives the numeric job of a
    scale-set attempt, refresh retires the alias cursor, at
    `service.rs:100-116`. Its range ends the day after the refresh date,
    not after the time of the cursor.
  - `removal_deltas`, at `crates/ciproxy-metering/src/report/runtime.rs:106`,
    reverses only the dates and the start inside that range. Then
    `retire_accounting` deletes the cursor.
  - A refresh can run with a time that is older than a saved observation.
    The Savings handler takes its time before its reads, at
    `crates/ciproxy-api/src/handlers/page_context.rs:36`. It then refreshes
    at `crates/ciproxy-api/src/handlers/savings.rs:42`, outside the
    `pass_lock` of the accounting worker at
    `crates/ciproxy-control-plane/src/accounting.rs:114`.
  - The attempt and its numeric job are two metering identities with two
    cursors, since migration
    `crates/ciproxy-schema/migrations/202611061300_scale_job_metering`.
    Each runner takes one job, as `docs/protocol/agent.md:157` and
    `docs/protocol/overflow.md:90` say. So one identity for each runner is
    possible.
  - Each reversal is computed again from the saved facts with the current
    calculation code, in `runtime.rs` and `boat.rs`.
  - Both callers take the refresh time from the current clock. So the race
    needs two refreshes on either side of UTC midnight.
- **Problem:** An older refresh can retire an alias whose cursor was
  observed on the next UTC date. The retirement range stops before that
  date, so it does not reverse the later start or runtime. The store then
  deletes the cursor, and the numeric job record counts the same
  contribution again.
- **Impact if no action:** Usage and savings totals stay too high until
  retention removes the dates. A later refresh cannot repair them, because
  the alias cursor is gone. Each race leaves at most one extra start for
  each job that GitHub mapped in that window, and the runtime between
  midnight and the newer refresh.
- **Root question:** How must metering remove every duplicate
  contribution?
- **Options:**
  - **A.** Derive each retirement range from the later of the refresh time
    and the observation time of the cursor. Add a regression case.
    Effort: Small.
  - **B.** Make `removal_deltas` take its complete upper bound from the
    cursor. Callers give only the retention start. Add the regression case.
    Effort: Small.
  - **C.** Move the alias accounting to the surviving record in one store
    transaction, when their mapping becomes known. This needs broader store
    changes. Effort: Medium.
  - **D.** Change all four delta functions to take only the retention
    start. Each function finds its upper bound from the later of the saved
    and the new observation times.
    - The four functions are `accounting_deltas` at `runtime.rs:72`,
      `removal_deltas` at `:106`, `start_deltas` at `:133`, and
      `boat_cost_deltas` at `crates/ciproxy-metering/src/report/boat.rs:52`.
    - Remove the three range calculations in `service.rs`, at `:100`,
      `:124`, and `:153`.
    - Add the alias case to
      `crates/ciproxy-metering/src/_tests_/service_tests.rs`. Use a cursor
      observed on the next UTC date and an older refresh time.
    - Job and Boat results do not change.
    - Four test files need the new call signatures:
      `crates/ciproxy-metering/src/_tests_/runtime_tests.rs`,
      `crates/ciproxy-metering/src/_tests_/boat_tests.rs`,
      `crates/ciproxy-store-pg/tests/store/metering/cursor_tests.rs`, and
      `crates/ciproxy-store-pg/tests/store/metering/concurrent/boat_tests.rs`.
    - Effort: Small.
  - **E.** Key the metering facts and the cursors by the runner, because
    each runner takes one job. Merge the attempt and the numeric job into
    one fact for each runner. Then no duplicate record and no retirement
    exist.
    - It needs a migration of the cursor key and of the cleanup triggers.
    - It also needs a rule for which source wins, a one-time retirement of
      the existing alias cursors, and store tests.
    - Effort: Large.
  - **F.** Save the daily contribution and the start contribution that each
    cursor applied. Subtract the saved contribution for each correction and
    each retirement.
    - This also keeps reversals exact after the calculation code changes.
    - It needs a migration with a backfill, larger cursor rows, and changes
      to both cursor writes.
    - Effort: Medium.
- **Left out:**
  - Skipping older refreshes: B and D reverse the full contribution at once
    for the same effort. Skipping keeps the duplicate until the next pass,
    and it reports a stale refresh.
  - Rebuilding the totals: it fails the goal, because job records expire
    before the 90-day totals.
  - One refresh at a time for each workspace, or a later time in the
    Savings handler: it fails the goal. A step of the wall clock, or two
    pods during a rollout, can still give an older refresh time.
- **Wider fix:** D. B stops the short reversal only for alias retirement.
  Job, start, and Boat corrections still take their upper bound from each
  caller.
- **Previous examples:** The test
  `older_refresh_preserves_next_day_cursor_and_corrects_every_affected_date`,
  at `crates/ciproxy-store-pg/tests/store/metering/observation_tests.rs:35`,
  covers older refreshes for ordinary jobs and Boat costs. It does not
  cover alias retirement.
- **Recommendation:** D, changed from B. D costs the same as B. It applies
  one rule to every cursor reversal: the saved contribution sets its
  complete reversal. So no caller can cut a reversal short.
- **Action:** fix after merge, apply D, because the race is rare and no
  production data holds its counts yet. The race needs two refreshes on
  either side of UTC midnight.
