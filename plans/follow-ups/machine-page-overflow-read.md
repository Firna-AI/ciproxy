# Machine page keeps stale overflow facts

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 1201, with the
  missing failure state of the machine page that its check found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/screens/machines/machine-detail.tsx`, the
  `overflowStore.ensure()` call and the `overflowSummary(overflow)` input of
  `slotsModel`, `ts/app/src/state/page-store.ts`, `PageRead.ensure` in
  `createPageRead`, `ts/app/src/state/machines-overflow.ts`,
  `overflowSummary`, `ts/app/src/view/machines.ts`, `boatTakesJobs` and
  `elsewhereText`, `ts/app/src/view/machines-slots.ts`, `slotsModel`,
  `ts/app/src/screens/machines/README.md`, and
  `docs/mockups/src/screens/machines/machine-detail.tsx`.
- **Reason:** The stale text needs a machine with no slots, an overflow
  change, and a return to the page that skips the machine list.

- **Terms:**
  - **Overflow summary:** the reply of
    `GET /workspaces/{id}/overflow/summary`. It says whether Boat takes
    jobs now, and it gives the Boat spend of the month.
  - **Summary store:** the store of the overflow summary in
    `ts/app/src/state/machines-overflow.ts`, built with `createPageRead`.
    One store lasts for the document, and the machine list and the machine
    page share it.
  - **No-slots machine:** an online machine without KVM, or without room
    for one slot. `machineStatus` in `ts/app/src/view/machines.ts` returns
    `"no-slots"` for it.
  - **`ensure()`:** the read of `PageRead` that runs once for each
    document. Later calls share that first read and its result.
- **Background:**
  - The machine page shows the slots of one machine. For a no-slots
    machine, it says "No slots." and then a sentence from `elsewhereText`,
    such as "Jobs run on your other machines and on Boat."
  - `elsewhereText` takes the Boat part from `boatTakesJobs(summary)`.
    `boatTakesJobs(null)` is false. `overflowSummary` gives null while the
    read loads and after a failed first read.
  - The page uses the summary only for a no-slots machine.
  - On each visit, the machine page calls `overflowStore.ensure()`.
    `ensure()` keeps its first promise for the whole document, so a later
    visit sends no read. A failed first read stays failed.
  - The machine list calls `overflowStore.load()` each time it opens. So a
    return through the list reads the summary again.
  - The Overflow panel of the list shows a failed first read and a failed
    refresh with Try again. It uses `overflowCopy.failed`, `refreshTitle`,
    and `refreshBody` of `ts/app/src/view/machines-copy.ts`.
  - The machine page passes only `overflowSummary(overflow)` to the slots
    text. So a failed read shows as the fact that Boat takes no jobs. A
    failed refresh keeps an old Boat fact with no warning and no Try again.
  - The Machines README says that the Overflow panel reads the summary each
    time the list opens, and with Try again, never on an interval. Each
    summary read asks Boat for the costs of the open starts.
  - `PageRead.ensure()` has one caller, the machine page, and no test. The
    billing page stores of `createPageStore` also carry `ensure()`, and
    none of them calls it.
  - The `product-ui` skill asks for an explicit unavailable or error state
    that names what failed and gives one recovery action.
  - A read-only probe of the review called `ensure()` twice, with an
    enabled and then a disabled summary queued. It sent one request and
    kept "No slots. Jobs run on Boat."
  - The browser check `followMachineLinks` in
    `ts/app/scripts/m20-browser-machine-states.ts` follows the links of a
    machine page and goes Back. `m20Control(origin, scenario, reset)` sets
    a harness scenario. The scenario `machines` has the machine
    `lab-no-kvm`. The scenario `overflow-unconnected` has no Boat account.
- **Problem:** The machine page reads the summary once for each document.
  After an overflow change, a return to the page that skips the list keeps
  the old text. A missing or failed summary shows as the fact that Boat
  takes no jobs, and a failed refresh shows no warning.
- **Impact if no action:** The page can promise Boat after an Admin turned
  overflow off. It can also say that jobs wait after Boat became available.
  A failed read hides that the page does not know where jobs go.
- **Root question:** How does the machine page base its text for a machine
  with no slots on the current overflow state, and show when that state is
  unknown?
- **Options:**
  - **A.** Call `load()` on each visit of the machine page. Add a
    navigation regression for changed overflow settings. Cost: `ensure()`
    stays with no caller. Effort: Small.
  - **B.** Share one entry-read hook between the machine list and the
    machine page. Read the summary on entry, and poll only the machines.
    Cost: the hook serves two views only, and `ensure()` stays with no
    caller. Effort: Small.
  - **C.** Add the overflow facts to an explicit response for the machine
    page. Cost: API changes, and the Boat cost reads must stay outside the
    machine poll. C drops in part the assumption that the summary reads
    only when a page opens or a person asks. Stored facts could then come
    with each machine poll. Effort: Medium.
  - **D.** Read the summary with `load()` each time the machine page opens,
    as the list does.
    - Delete `ensure()` and its `first` promise from `PageRead`. The type
      check then refuses a read that lasts for the document on every page
      store.
    - Update the Machines README: the machine page also reads the summary
      each time it opens. It uses the summary to say where jobs go while
      the machine has no slots.
    - Add a browser check next to `followMachineLinks`. Open `lab-no-kvm`
      in the scenario `machines`. Set `overflow-unconnected` with
      `reset: false`. Follow Job history, then go Back.
    - Expect "No slots. Jobs run on your other machines." The current code
      keeps "No slots. Jobs run on your other machines and on Boat."
    - Cost: each step from the list to a machine sends one more summary
      read, with its Boat calls.
    - Effort: Small.
  - **E.** D, but the machine page reads the summary only while the machine
    shows no slots. It reads when the page opens, and when a poll moves the
    machine into that state.
    - Cost: the read condition must follow each use of the summary on the
      page. Otherwise a new use shows a guess and starts no read. On a
      direct open, the guess shows for one more round trip.
    - Benefit: a machine with slots sends no summary read. A machine that
      loses its slots while the page is open gets a fresh read.
    - Effort: Small.
  - **F.** Replace `ensure()` in `PageRead` with one entry read, such as
    `visit()`. It shares the read that is open, and otherwise it starts a
    new read.
    - The list and the machine page call it when they open. Try again keeps
      `load()`.
    - Add a case to `ts/app/_tests_/page-store.test.ts`: a visit during an
      open read sends no request, and a visit after that read sends one.
      Also add the browser check of D.
    - Cost: a read that started before a change and stalls answers the next
      visit, because the HTTP client has no timeout.
    - Benefit: a quick step from the list to a machine sends no second Boat
      read. It is also the background-read rule of option E of
      `plans/follow-ups/explicit-session-refresh-order.md`.
    - Effort: Small.
  - **G.** D, and give the machine page a state for the summary.
    - While no summary is known, say where jobs go only from the facts that
      the page knows.
    - After a failed read or a failed refresh, show the failure with Try
      again, as the Overflow panel does with `overflowCopy.failed`,
      `refreshTitle`, and `refreshBody`.
    - Add the state to the Machines mockups first, as the `mockups` skill
      requires. Add the cases for a missing summary to
      `ts/app/_tests_/machines-slots.test.ts`.
    - G drops the assumption that a missing or failed summary means that
      Boat takes no jobs, with no failure state on the machine page. The
      source is the code, and no reason is recorded.
    - Cost: a new page state and new copy, in the same text that
      `plans/follow-ups/machines-boat-start-status.md` changes.
    - Effort: Medium.
- **Left out:**
  - Permanent caching: it fails the goal, because it misses later changes.
  - A summary poll on an interval: it breaks the read rule of the Machines
    README, and each poll asks Boat for costs.
  - A read each time the browser tab shows again: alone, it fails the
    goal. A return by Back in a tab that stays visible starts no read.
  - A new summary store each time a page opens: D is better on every
    point. Both read each time the page opens. But a new store shows the
    guess for a missing summary during each read, drops the summary that
    the list shares, and needs new code.
  - A mark of the summary as old after a save in the overflow settings:
    not a real choice. The spend limit, a key that Boat refuses, and other
    people also change the summary.
  - A server push of overflow changes: not a real choice. The dashboard
    reads every page over HTTP and has no push channel. A read when the
    page opens already meets the goal.
- **Wider fix:** D. B must know which two views read the summary, so it is
  a direct fix. D removes the read that lasts for the document from every
  page store. For the other read stores, option E of
  `plans/follow-ups/explicit-session-refresh-order.md` stays the wider fix.
- **Previous examples:**
  - `plans/follow-ups/setup-progress-cache.md` records another stale view
    from a read that lasts for the document, in the Setup store.
  - Commit `3428fd37` of 17 November 2026 moved the summary store onto
    `createPageRead`, which carries `ensure()`. It gave the Overflow panel
    its refresh failure, but not the machine page.
- **Related:**
  - `plans/follow-ups/machines-boat-start-status.md`: the same code, and
    options that conflict.
    - Its option D adds a start status to this summary. The machine page
      shows that status through this store. So the read rule of this item
      decides how current that status is on the machine page.
    - C here moves the Boat facts to a machines response. That item would
      then need its status in two responses. A, B, D, E, F, and G do not
      conflict with it.
    - The state of G for a missing summary must follow the rule of that
      item that keeps a missing observation apart from a known hold.
  - `plans/follow-ups/explicit-session-refresh-order.md`: the same cause,
    and a shared wider fix.
    - In the Setup store, the shell calls `ensure()` and the page calls
      `load()`, so the first Setup read goes out twice. Here, `ensure()`
      keeps the first read for the whole document.
    - Both stores lack one entry read that shares an open read and
      otherwise starts a new read. F builds that read in the page store.
    - Option E of that item puts every read store on one read rule, the
      page store too. D removes `ensure()` before that work. F is the first
      step of that rule for the page store.
    - The options do not conflict, and neither fix waits for the other.
- **Recommendation:** D, with G for a missing or failed summary. This
  differs from the review, which recommended B. D gives the machine page
  the read on opening that the list already uses. It also deletes the only
  page-store read that lasts for the document. G adds the failure state
  that the `product-ui` skill asks for.
- **Action:** fix after merge, apply D with G for the missing or failed
  summary, because it is Medium and Rare. The stale text needs a machine
  with no slots, an overflow change, and a return that skips the list.
