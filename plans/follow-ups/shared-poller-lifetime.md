# Pollers restart after their page closes

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, the wider fix of review finding 3 that its finding check
  split off. The restart of the links store joined it as the same class,
  from a problem outside the scope of that check.
- **Source:** Review.
- **Category:** Performance.
- **Location:** `ts/app/src/state/poller.ts:58`, `run`,
  `ts/app/src/state/poller.ts:63`, `start`,
  `ts/app/src/state/links.ts:106`, in `resume`,
  `ts/app/src/state/billing-return.ts:101`, in `follow`,
  `ts/app/src/state/jobs-live.ts:78`, `watch`,
  `ts/app/src/state/setup.ts:94`, `watch`, and
  `ts/app/src/state/billing-page.ts:81`, `watch`.
- **Reason:** The wider fix changes the shared poller and stores that the
  reviewed branch did not change, and that branch fixed its own two leaks
  directly.

- **Terms:**
  - **Poller:** `createPoller` in `ts/app/src/state/poller.ts`, a repeated
    background read. It waits for each read, pauses while the tab is
    hidden, and reads at once when the tab shows again.
  - **Tab watcher:** the listener on the `visibilitychange` event of the
    document that a running poller registers through
    `services.env.visibility`.
  - **Page lifetime:** the time from the mount of a page that reads in the
    background to its close.
- **Background:**
  - `start()` registers a tab watcher and schedules the next read, at
    `poller.ts:63-72`. `stop()` clears the timer and removes the watcher, at
    `:73-78`.
  - A read that returns false ends the cycle by itself, at `:58`. Its tab
    watcher stays registered.
  - `start()` works again after `stop()` or after a cycle that ended by
    itself. It then saves the remover of a new watcher in place of the old
    one, so the old watcher stays for the life of the document.
  - Six stores own a poller: Savings, at `ts/app/src/state/savings.ts:86`,
    Jobs Live, at `jobs-live.ts:65`, Setup, at `setup.ts:80`, the billing
    page store, at `billing-page.ts:67`, the Checkout return, at
    `billing-return.ts:91`, and the organization links, at `links.ts:159`.
  - Each store guards its restarts by hand:
    - Savings opens a page lifetime when its page mounts. `open()` returns
      the close, and a read that answers after the close starts nothing.
    - The Checkout return checks `active` before it starts again after a
      check, at `billing-return.ts:101`. Its retry stops the ended cycle
      first, so it releases the old watcher.
    - The links store starts again in `resume()` while a page watches, at
      `links.ts:101-108`. After a cycle that ended by itself, that start
      leaves the old tab watcher registered.
    - Jobs Live, Setup, and the billing page store start their one poller
      in `watch()` and stop it in the returned function. So the stop of a
      second watcher of the same store ends the poll of the first one.
  - Each poll goes through `session.report`. A 401 ends the signed-in
    view, and a 403 reads the session again. A Savings read also runs a
    metering refresh on the server.
- **Problem:** The shared poller has no page lifetime. It starts again
  after a stop, and a cycle that ends by itself keeps its tab watcher. So
  each store needs its own restart guard. A store without one keeps a tab
  watcher, or reads again, after its page closed.
- **Impact if no action:** A closed or restarted page can keep a listener
  for the whole document, and a missed guard can keep reading, with a
  server refresh for Savings. Each new store with a poller must write and
  test its own guard.
- **Root question:** How does each page own its background reads, so that
  no read starts after the page closes?
- **Options:**
  - **A.** Keep a guard in each store. Its stop ends a pending start, and
    each restart checks the guard. Fix the links store with a stop before
    `resume()` starts again. This keeps six guards that each store writes
    and tests by itself. Effort: Medium.
  - **B.** Give each store a synchronous page subscription that owns its
    polling, as Savings has now. The page opens it before its first read,
    and its close ends the polling. A read loads data, and it can start
    polling only while a subscription is open. Migrate the other five
    stores. Effort: Medium.
  - **C.** Add request cancellation to the HTTP client and the polled
    reads, and bind the cancellation and the polling to one page lifetime.
    This reaches the shared transport and its other callers. Effort:
    Medium.
  - **D.** Give the shared poller a page lifetime:
    - `open()` registers the one tab watcher, counts its openers, and
      returns a close function.
    - `start()` arms a cycle only while the lifetime is open. A cycle that
      ends by itself keeps the one watcher.
    - Each page subscription opens the lifetime: Savings, Jobs Live,
      Setup, the billing page store, the Checkout return, and the links
      store.
    - The lifetime replaces the local guards: the page lifetime of
      Savings, the `active` check and the retry stop of the Checkout
      return, and the `watching` check of the links store.
    - Add poller cases for a start after the close and for a restart after
      a cycle that ended by itself, and keep the store regressions.
    - The opener count also gives the watcher count of the Setup store.
    - Effort: Medium.
- **Left out:**
  - A stop of only the scheduled timer, because a later continuation of a
    read can still start a new timer.
  - Ignoring state changes after the page closes, because the reads and
    the tab watcher keep running.
  - A server that returns only complete Savings reports, because it needs
    an API change and waits in the request, and it does not reach the
    other stores.
  - A query-cache library that ties polling to mounted observers, because
    D gives the same lifetime rule with no new dependency and no rewrite
    of every store.
- **Wider fix:** D. One page lifetime in the shared poller keeps every
  store from a restart after its page closed and from a second tab watcher.
- **Previous examples:**
  - The Savings store restarted its poller when a read answered after the
    page closed. The branch that recorded this item fixed it with option B
    for Savings only.
  - The retry of the Checkout return started again without a stop and
    left the old watcher. The same branch fixed it with a local stop.
  - The links store still leaves the old watcher when `resume()` starts
    again, at `links.ts:106`.
  - `plans/follow-ups/setup-progress-cache.md` records that a second Setup
    watcher stops the shared poll.
- **Related:** `plans/follow-ups/setup-progress-cache.md` counts the
  watchers of the Setup store. The opener count of D gives that count for
  every store, so the two fixes share one mechanism. Neither item waits for
  the other.
- **Recommendation:** D. One lifetime in the shared poller gives every
  store the same rule, so no new store can restart after its page closed.
- **Action:** fix after merge, apply D, because it changes the shared
  poller and stores that the reviewed branch did not change.
