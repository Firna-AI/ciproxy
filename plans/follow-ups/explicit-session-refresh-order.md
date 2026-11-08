# Dashboard reads can apply older snapshots

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from `conductor/mvp-m19-ui`, PR #36, review
  finding 4 on the session store. Review finding 6 on the Setup store
  joined it on 2026-11-07, because both findings share one wider fix.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/state/session.ts:87`, `load`, `:137`, `mutate`,
  `:149`, `ensure`, `:157`, `refresh`, `:158`, `visit`, and `:179`,
  `claimAdmin`, `ts/app/src/state/setup.ts:64`, `load`, and `:90`,
  `ensure`, and `ts/app/src/state/poller.ts:52`, `run`.
- **Reason:** Each wrong result needs one reply that arrives after a later
  read or action, and one more check, a reload, or a new Setup visit
  recovers.

- **Terms:**
  - **Session store:** `createSessionStore` in `ts/app/src/state/session.ts`.
    It keeps the last reply of `GET /api/session`: the sign-in, the
    workspaces and their roles, and the CSRF token.
  - **Setup store:** `createSetupStore` in `ts/app/src/state/setup.ts`. It
    reads Setup and the machines of one workspace as one pair, and it polls
    while Setup waits.
  - **Explicit read:** a read that a person or an action asks for, so it
    must start after that request. Examples are Check again, Try again, the
    read after an Admin claim, the CSRF recovery that `mutate` runs after a
    refused change, and the Setup page load.
  - **Background read:** a read that only keeps a page current, so it can
    share an open read. Examples are the first load, a page change, the
    role check that `report` starts after a refused read, and a Setup poll.
  - **Generation:** the counter of the session store. Sign-out and a 401
    advance it, and a reply from an older generation changes nothing.
- **Background:**
  - The dashboard is the Expo app in `ts/app`. Its stores keep API replies
    for the screens, and they read the API again when a page or an action
    asks.
  - In the session store, `load` returns the open read when one exists. It
    starts a new read only when no read is open.
  - `refresh` is the same function as `load`. Check again calls it in
    `useCheckAgain` at `ts/app/src/services/hooks.ts:59`. The links check
    calls it in `checkAgain` of `ts/app/src/state/links.ts`.
  - Try again calls `refresh` at
    `ts/app/src/components/session-notice.tsx:25` and
    `ts/app/src/components/page-state.tsx:107`.
  - After a successful claim, `claimAdmin` waits for `load`. The claim
    endpoint returns only `{ "ok": true }`, from `claim` in
    `crates/ciproxy-api/src/handlers/sessions.rs`. So the new role comes
    only from a session read.
  - After the claim, the shell and the page for no role open
    `/?workspace=<id>`, at `ts/app/src/components/shell/shell-actions.ts:64`
    and `ts/app/src/screens/no-role.tsx:65`. That landing calls `visit`.
  - `visit` shares an open read. It also returns the stored state with no
    read when a read ended less than `sessionFreshFor`, 2 seconds, ago.
  - The API can refuse a request with a 403 for two different reasons. So
    the store handles a refused read and a refused change on two paths.
  - A refused read of private data, such as Setup, goes through `report`.
    There a 403 means that the role may have ended. `report` then starts a
    session read and does not wait for it. This read is the role check. It
    is a background read.
  - A refused change goes through `mutate`. There a 403 can mean that
    another tab signed in and rotated the CSRF token. `mutate` then reads
    the session with `load`, and it repeats the change once only when the
    token changed.
  - This read of `mutate` is the CSRF recovery. It is an explicit read,
    because the repeat needs a token from after the refusal.
  - Each session read first asks GitHub for the open install requests, in
    `session` in `crates/ciproxy-api/src/handlers/sessions.rs`. So a read
    can stay open while the person acts.
  - Sign-out and a 401 advance the generation and drop the open read, at
    `session.ts:81` and `:173`. This is the only guard of the session store
    against an older reply.
  - In the Setup store, `load` reads Setup and the machines with
    `Promise.all`. Then it stores the pair, or the failure of either read.
    It does not check whether a newer read already stored a result.
  - The Setup page calls `load` on mount and starts the poller with `watch`,
    at `ts/app/src/screens/setup/setup-screen.tsx:31`. Try again calls
    `load` too. `ensure` shares only the first read of the document.
  - The poller waits for each read before it schedules the next, so two
    polls never overlap. A poll whose read returns false stops the poller.
    A stopped poller ignores the tab visibility.
  - `load` returns false when Setup no longer waits: a machine joined, and
    the first job completed.
  - The Setup refresh notice says "Setup shows the last state that it read,
    and tries again by itself."
  - The HTTP client sets no timeout, at `ts/app/src/api/client.ts:101`. So
    a stalled reply can stay open for a long time.
  - The claim case needs a claim that commits after the older read read the
    roles, while the claim reply still arrives first. That window is the
    end of one session read, or a delay of one reply on the network.
  - The Setup case needs a first job that ends between two server reads, a
    later poll that answers first, and an earlier reply that stalls. The
    first job ends once for each workspace.
  - `ts/app/_tests_/session.test.ts` covers the sign-out and 401 races, the
    role check, and a CSRF recovery with a fresh read. No test keeps an
    older read open during a claim or a CSRF recovery.
  - `ts/app/_tests_/setup.test.ts` covers the kept view after a failed read
    and the stop of polling at completion. No test keeps an older load open
    while a poll answers.
- **Problem:** Both stores can let an older reply decide what a newer read
  or action shows. These orders give a wrong result:
  - Claim: an older session read with the earlier role answers the read
    after a successful claim. The landing then shows the page for no role
    again, or the shell with the earlier role. A page change within 2
    seconds reuses that state.
  - CSRF recovery: after another tab signed in, the API refuses a change
    with a 403. The CSRF recovery gets an older read with the old token, so
    `mutate` does not repeat the change.
  - Setup: a load that is not a poll gets an old first-job phase while its
    machines reply stalls. A later poll stores the completed first job and
    stops. The stalled reply then restores the old progress, and no poll
    corrects it.
  - Setup failure: a late failure of that load shows that Setup tries again
    by itself, after the poller stopped.
- **Impact if no action:** A granted Admin role can stay hidden until
  another explicit check, and a change that could succeed stays refused.
  Setup can show an open first job until a reload, with a notice that
  promises a retry that does not run.
- **Root question:** How does each dashboard store keep an older reply from
  replacing newer state, or from answering a newer request?
- **Options:**
  - **A.** Read again after each affected action. The claim and the CSRF
    recovery wait for the open session read, then start one more read. Setup
    gets its own guard: a load stores its reply only when it is newer than
    the last stored reply. Cost: the older session reply still applies
    first, so the stale state shows for a moment. Each new caller must add
    the extra read. Effort: Small.
  - **B.** Drop the open session read after chosen changes. The claim and
    the CSRF recovery advance the generation, then read again. Setup gets the
    guard of A. Cost: each caller must opt in, and Check again and Try
    again still share an older read. Effort: Small.
  - **C.** After a successful claim, set the Admin role in the session store
    and advance the generation. Setup gets the guard of A. Cost: the client
    copies a server result, against the rule of `ts/app/README.md` that
    roles come from the API only. `can_claim_admin` and the list of Admins
    stay old until the next read. The CSRF recovery stays unfixed. Effort:
    Small.
  - **D.** Give the session store one explicit refresh. When a read is
    open, it advances the generation and starts a new read. The claim, the
    CSRF recovery, Check again, and Try again use it. `visit`, `ensure`, and
    the role check keep the shared read. Setup gets the guard of A. Cost: two
    separate rules in two stores, and each later polling store needs its
    own copy. Effort: Small.
  - **E.** Put the read order in one helper under `ts/app/src/state/`, and
    move both stores to it. The helper numbers each read when it starts. A
    reply or a failure applies only when no read that started later has
    applied. A background read shares an open read. An explicit read
    starts a new read. A poll decides from the stored state, not from its
    own reply. Cost: the helper must keep the sign-out and 401 rule and the
    visible failure of the newest read. An explicit read during a
    background read sends one more session read, with its GitHub call.
    Effort: Medium.
  - **F.** Return the full session from the claim endpoint. Cost: a Rust
    change and regenerated API types. The claim would also need the GitHub
    call of the session read. An older read can still land after the
    returned session, so F still needs the generation step of D. The CSRF
    recovery and Setup stay unfixed. Effort: Medium.
- **Left out:**
  - Change nothing: it fails the goal, because the orders above stay
    possible.
  - Stop sharing session reads only: an older read can still answer after
    the new one.
  - A shorter 2-second window, or none: the read after a claim still gets
    the open older read. Each page change would also add a session read,
    with its GitHub call.
  - A revision from the API: inside one tab, the start order of the client
    gives the same result with no Rust change.
  - One shared open read, or a queue, for every Setup read: the page load
    and Try again would get, or wait behind, a read that started before
    them. The client has no timeout, so Try again could wait behind a
    stalled read.
  - A poll that restarts only while the stored state waits: old progress
    still shows for up to 5 seconds.
  - An abort of older requests in the client: A and E are better on each
    point. An aborted poll would also need a new failure kind, so that it
    shows no refresh failure.
  - A first-job state that never moves back: a machine can leave and the
    App can be removed, so it would hide real changes.
  - One combined Setup endpoint: two combined reads can still answer out of
    order, and it needs a Rust change.
  - One shared store that also owns the value, the failure, and polling: E
    meets the goal with a smaller change, and the stores keep different
    state shapes.
- **Wider fix:** E. One helper holds the order rule, so each store and each
  later polling page gets the rule by default.
- **Previous examples:** Sign-out and a 401 already advance the generation,
  so a reply from before them changes nothing. The tests "sign out clears
  local access and a stale refresh cannot restore it" and "401 ends access
  and an expired sign-in survives a stale request" in
  `ts/app/_tests_/session.test.ts` cover this.
- **Related:** `plans/follow-ups/setup-progress-cache.md` changes the same
  Setup store. Its fix adds two Setup reads: a watch from the shell, which
  is a background read, and a fresh read for the landing, which is an
  explicit read. Neither fix waits for the other. The fix that lands second
  follows the read rule of E.
- **Recommendation:** E. One rule gives every explicit read a new read, and
  it stops every older reply in both stores.
- **Action:** fix after merge, apply E, because each wrong order needs one
  stalled reply in a short window, and one more check or a reload
  recovers.
  - Add the helper under `ts/app/src/state/`. It numbers each read when the
    read starts. A reply or a failure applies only when no read that
    started later has applied.
  - In the session store, `ensure`, `visit`, and the role check of `report`
    make background reads. `refresh`, the read after a successful claim, and
    the CSRF recovery of `mutate` make explicit reads. Keep the 2-second
    window of `visit`.
  - Keep one rule for a 403. A refused read goes through `report`, and its
    role check is a background read. A refused change goes through
    `mutate`, and its CSRF recovery is an explicit read.
  - A discarded reply changes neither the stored state nor the time of the
    last read.
  - In the Setup store, `ensure` and the poll make background reads. The
    page load and Try again make explicit reads. Keep the Setup reply and
    the machines reply as one pair.
  - The poll decides from the stored state after its read settles. A stored
    failure keeps the poll running, as now.
  - Keep the sign-out and 401 rule. They drop every open read, so no older
    reply restores the signed-in view.
  - A discarded reply sets no failure. The failure of the newest read stays
    visible in `SessionRefreshNotice` and in the Setup refresh notice.
  - The test "403 reads refresh the session so a lost role becomes visible"
    in `ts/app/_tests_/session.test.ts` starts the role check. Then it
    awaits that open read through `refresh`. `refresh` then starts a new
    read, so await the role check through `visit`, with the same assertions.
  - Add cases through the public stores, with `appWith`, `deferred`, and
    `manualSchedule` from `ts/app/_tests_/support.ts`. The cases need no
    network and no delay.
  - Claim case: keep an older session read with the earlier role open.
    Claim, and answer the read after the claim with the Admin role. Then
    release the older read. Expect the Admin role, also from `visit`.
  - CSRF recovery case: keep an older session read with the old token open.
    Let the API refuse a change of `mutate` with a 403, and answer the new
    read with a new token. Then release the older read. Expect one repeat of
    the change, with the new token.
  - Setup case: keep the machines reply of a page load open. Fire a poll
    that answers with a completed first job. Then release the older reply.
    Expect the completed state, no refresh failure, and a stopped poller.
  - Setup failure case: release the older reply as a failure. Expect no
    refresh failure.
  - Run `node --test ts/app/_tests_/session.test.ts` and
    `node --test ts/app/_tests_/setup.test.ts`. The complete gate does not
    run these tests. Run each new case 10 times.
  - When a change fixes only one store, use D for the session store and the
    guard of A for Setup. Do not copy the shared open read of the session
    store into Setup.
