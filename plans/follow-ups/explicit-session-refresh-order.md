# Dashboard read stores lack one read rule

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from `conductor/mvp-m19-ui`, PR #36, review
  finding 4 on the session store. Review finding 6 on the Setup store
  joined it on 2026-11-07, because both findings share one wider fix. On
  2026-11-17, the final review of the branch
  `calummoore/ci-proxy-load-balancer` found the Setup case again as
  finding 602. The wider fix of its finding 604 joined this item too,
  because both are one class: one read rule for every dashboard read store.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/state/session.ts:87`, `load`, `:137`, `mutate`,
  `:149`, `ensure`, `:157`, `refresh`, `:158`, `visit`, and `:179`,
  `claimAdmin`, `ts/app/src/state/setup.ts:64`, `load`, and `:90`,
  `ensure`, `ts/app/src/state/poller.ts:52`, `run`,
  `ts/app/src/state/page-store.ts`, `createPageStore`,
  `ts/app/src/state/machines.ts:46`, `ts/app/src/state/jobs-live.ts:41`,
  `ts/app/src/state/savings.ts:50`, `ts/app/src/state/settings-store.ts:22`,
  and `ts/app/src/state/settings-members.ts:37`.
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
    at `ts/app/src/screens/setup/setup-screen.tsx:34-36`. Try again calls
    `load` too. `ensure` shares only the first read of the document.
  - On a direct load or a reload of Setup, the page's `load` and the
    shell's `ensure` at `ts/app/src/screens/workspace-page.tsx:58` start at
    the same time. So the page sends `GET setup` and `GET machines` twice.
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
    first job ends once for each workspace. The first poll starts 5 seconds
    after the page opens.
  - `ts/app/_tests_/session.test.ts` covers the sign-out and 401 races, the
    role check, and a CSRF recovery with a fresh read. No test keeps an
    older read open during a claim or a CSRF recovery.
  - `ts/app/_tests_/setup.test.ts` covers the kept view after a failed read
    and the stop of polling at completion. No test keeps an older load open
    while a poll answers.
  - The shared page store is `createPageStore` in
    `ts/app/src/state/page-store.ts`. It moved there from
    `ts/app/src/state/billing-page.ts` on 17 November 2026, so that Machines
    does not import a billing module.
  - The page store numbers each read when it starts, and it drops an older
    reply. It keeps the last good page beside the failure of a later read,
    and it polls while the page waits, or while no good page exists. The
    Usage store and the Plans store use `createPageStore`. The Machines
    overflow summary store uses its read rule alone, `createPageRead`, with
    no poll.
  - Five other read stores copy the same rule by hand: a counter of the
    latest read, and a refresh failure beside the kept data. They are
    `machines.ts`, `jobs-live.ts`, `savings.ts`, `settings-store.ts`, and
    `settings-members.ts`, all under `ts/app/src/state/`.
  - Two generic snapshot types exist: `PageSnapshot` in `page-store.ts`,
    and `ReadSnapshot` at `settings-store.ts:22`.
  - `forget` at `machines.ts:85` advances the counter of the Machines store
    and drops one machine from the kept list.
  - The copies drift. Until 17 November 2026, the Machines overflow summary
    store ignored a failed read after a success. Then it moved onto the page
    store. The Setup store has no order guard at all.
  - Three stores follow other rules. The History store in `jobs-history.ts`
    and the job store in `jobs-detail.ts` replace the view with a failure.
    The Settings overflow store in `overflow.ts` queues each read behind an
    open edit.
  - Work inside the `read` of a page store runs before the counter drops an
    older reply. `forgetEnded` of the billing stores is such work.
- **Problem:** Two stores can let an older reply decide what a newer read or
  action shows, and five stores keep copies of the read rule by hand. These
  cases give a wrong result, or can give one later:
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
    corrects it. The final review of 17 November 2026 reproduced it: after
    the older reply, the store showed zero machines and scheduled no poll.
  - Setup failure: a late failure of that load shows that Setup tries again
    by itself, after the poller stopped.
  - Copies: the five copies work today, but a hand-made copy can break, as
    the copy of the overflow summary store did before 17 November 2026.
- **Impact if no action:** A granted Admin role can stay hidden until
  another explicit check, and a change that could succeed stays refused.
  Setup can show an open first job until a reload, with a notice that
  promises a retry that does not run. Each new read store needs its own
  copy of the rule, and each copy can break again.
- **Root question:** How does every dashboard read store stop an older
  reply from setting its state, failure, or polling, or from answering a
  newer request?
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
  - **E.** Put one read rule and one generic read snapshot in one shared
    module under `ts/app/src/state/`, and build every dashboard read store
    on it.
    - The rule numbers each read when it starts. A reply or a failure
      applies only when no read that started later has applied.
    - A background read shares an open read. An explicit read starts a new
      read.
    - A poll decides from the stored state, not from its own reply.
    - The snapshot keeps the last good data beside the failure of the
      newest read. Keep one of the two generic snapshot types.
    - `createPageStore` is the natural base, because it already owns the
      counter, the refresh failure, and the poll.
    - The stores: the session store, the Setup store, the users of the page
      store, `machines.ts`, `jobs-live.ts`, `savings.ts`,
      `settings-store.ts`, and `settings-members.ts`. The History store,
      the job store, and the Settings overflow store keep their own rules.
    - Cost: the helper must keep the sign-out and 401 rule and the visible
      failure of the newest read. An explicit read during a background read
      sends one more session read, with its GitHub call. About eight stores
      change, with their screens and their tests. `forget` of `machines.ts`
      needs a hook.
    - The work can land in steps of a few stores. Each step keeps the
      product working.
    - Effort: Large.
  - **F.** Return the full session from the claim endpoint. Cost: a Rust
    change and regenerated API types. The claim would also need the GitHub
    call of the session read. An older read can still land after the
    returned session, so F still needs the generation step of D. The CSRF
    recovery and Setup stay unfixed. Effort: Medium.
  - **G.** Build only the Setup store on the shared page store.
    - Give `createPageStore` the poll interval as an option, when it does
      not take one yet. Keep `statusWaits` and the billing interval in the
      billing module.
    - In the shared store, keep the poll running after a failed read when
      no good page exists, as Setup does now. Billing never polls in that
      state, because its screens call `watch` only while a ready page
      waits.
    - Without that change, the store stops the poll when a read fails
      before any good page exists. The Setup page calls `watch` only once
      for each visit, so after a successful Try again, Setup has no poll.
    - Read `{ setup, machines }` as one page, with one `report` call for
      each request.
    - Move the four readers to `snapshot.page`: `setup/setup-screen.tsx`,
      `workspace-page.tsx`, `landing.tsx`, and `savings/savings-screen.tsx`,
      under `ts/app/src/screens/`.
    - Add the delayed-read regression. Add a second regression: a poll
      fails before any good page, then Try again succeeds.
    - Cost: the four readers change shape, and the session store stays
      unfixed.
    - Effort: Small.
- **Left out:**
  - Change nothing: it fails the goal, because the orders above stay
    possible.
  - Stop sharing session reads only: an older read can still answer after
    the new one.
  - A shorter 2-second window, or none: the read after a claim still gets
    the open older read. Each page change would also add a session read,
    with its GitHub call.
  - A revision from the API: inside one tab, the start order of the client
    gives the same result with no Rust change. Also, only the client knows
    that its read failed.
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
  - One store type that also owns the sign-in state of every store: the
    session store keeps its own shape, with the generation, the CSRF token,
    and the 2-second window. E shares the rule and the snapshot, and each
    store keeps its own extra state.
  - A query-cache library: every store uses the app's own `createStore` and
    `createPoller`, and E gives the same rule with no new dependency.
  - One read of the machines and the overflow summary together: the summary
    would then follow the 10-second poll of the machines. Each summary read
    asks Boat for the costs of the open starts.
- **Wider fix:** E. One module holds the read rule and the snapshot, so
  each store and each later read store gets the rule by default.
- **Previous examples:**
  - Sign-out and a 401 already advance the generation, so a reply from
    before them changes nothing. The tests "sign out clears local access and
    a stale refresh cannot restore it" and "401 ends access and an expired
    sign-in survives a stale request" in `ts/app/_tests_/session.test.ts`
    cover this.
  - On 17 November 2026, the final review found that the Machines overflow
    summary store ignored a failed read after a success. Its fix moved the
    store onto `createPageRead`, the read rule of the page store.
- **Related:**
  - `plans/follow-ups/setup-progress-cache.md` changes the same Setup store.
    Its fix adds two Setup reads: a watch from the shell, which is a
    background read, and a fresh read for the landing, which is an explicit
    read. Neither fix waits for the other. The fix that lands second follows
    the read rule of E.
  - `plans/follow-ups/shared-poller-lifetime.md` changes the same Setup
    `watch`. E keeps the poller in place. G moves it into the page store,
    so the steps of that item move with it. Neither fix waits for the other.
  - `plans/follow-ups/saved-payment-lifetime.md`, the same class in other
    code. Its removal runs inside the `read` of the billing page stores,
    before the counter drops an older reply. The rule of E orders the reads
    of one store only, so neither item closes the other.
- **Recommendation:** E. One rule gives every explicit read a new read, and
  it stops every older reply in every read store. G is the smallest fix of
  Setup alone.
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
    the machines reply as one pair. The shell's `ensure` then shares the
    open page read, so a direct load sends each Setup request once.
  - The poll decides from the stored state after its read settles. A stored
    failure keeps the poll running, as now.
  - Keep the sign-out and 401 rule. They drop every open read, so no older
    reply restores the signed-in view.
  - A discarded reply sets no failure. The failure of the newest read stays
    visible in `SessionRefreshNotice` and in the Setup refresh notice.
  - Build the other read stores on the same module: the users of the page
    store, `machines.ts`, `jobs-live.ts`, `savings.ts`, `settings-store.ts`,
    and `settings-members.ts`. Keep their behaviour and the assertions of
    their tests. Give `forget` of `machines.ts` a hook.
  - Keep each `read` free of side effects, except `report`, because work
    inside a `read` runs before the rule drops an older reply.
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
  - Also run the tests of each other moved store under `ts/app/_tests_/`,
    such as `machines-store.test.ts`, `jobs-store.test.ts`,
    `savings-store.test.ts`, `settings-stores.test.ts`,
    `settings-members.test.ts`, and the billing tests.
  - When a change fixes only one store, use D for the session store, and G
    or the guard of A for Setup. Do not copy the shared open read of the
    session store into Setup.
