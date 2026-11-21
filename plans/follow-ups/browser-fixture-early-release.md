# An early fixture release loses its signal

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from pull request #51, fresh review finding 7.
- **Source:** Review.
- **Category:** Test.
- **Location:** `ts/app/scripts/harness-boat-api.ts:96`,
  `ts/app/scripts/harness-state.ts:65`,
  `ts/app/scripts/harness-boat.ts:38`, and
  `ts/app/scripts/browser-boat-account.ts:57`.
- **Reason:** The two permitted review fix rounds are complete, and this
  test issue is noncritical.
- **Terms:**
  - **Browser fixture:** The local test API that supplies dashboard
    responses without real credentials.
  - **Held connection:** A Boat key replacement request that the fixture
    pauses while the browser checks disabled controls.
  - **Release:** The controller command that ends the fixture's hold.
- **Background:**
  - The `overflow-change-key-pending` browser journey replaces a Boat key.
    It checks that Cancel, the overflow switch, and Save stay disabled
    while the change runs.
  - The `overflow-change-key-held` scenario sets `holdConnection` to true.
    Its release callback starts as null.
  - `serveBoat` reads the connection body before it stores that callback.
    `updateHarness` invokes the callback when it receives `{ release: true }`.
  - The app sets its pending mark before the request leaves the browser.
    Thus the disabled-control checks can finish before the fixture stores
    its callback.
- **Problem:** A release that arrives before callback registration does
  nothing. The controller clears the callback field, and `holdConnection`
  stays true. After the body read, the connection installs a callback and
  waits forever for another release.
- **Impact if no action:** The browser journey can time out although the
  app works. Its result depends on the order of the connection and release
  requests. The wait for "Connected to the account northstar" then reaches
  the default 30-second browser timeout.
- **Evidence:**
  - Two independent in-memory probes called `serveBoat` and `updateHarness`
    with a deferred body read. They used promises and event order, without
    a timer, file, or running server.
  - Releasing before completing the body read left 10 of 10 requests
    pending. A second release completed every request with status 200.
  - Releasing after hold registration completed 10 of 10 requests with
    one release.
  - Stored browser runs passed 28 times at each of 1440, 720, and 390
    pixels. No browser hang was observed in those 84 passes. The probe
    demonstrates a narrow timing window, rather than a failed browser rerun.
- **Root question:** How can the fixture hold the connection through the
  lock checks, then end its hold under either request order?
- **Options:**
  - **A.** Clear `holdConnection` when release arrives, then invoke any
    existing callback. A later connection skips the hold. This keeps two
    fields for one operation. A later await between the flag check and
    callback registration could restore the race. Effort: Small.
  - **B.** Create one shared hold object when the scenario starts. Give it
    a promise and its release function together. The connection awaits
    that promise, and the controller calls that release function. An early
    release remains effective. Effort: Small.
  - **C.** Expose a non-secret hold-readiness boolean through the fixture
    state. Make the journey wait for readiness before release. This needs
    a bounded readiness read and leaves the trap for callers that omit it.
    Effort: Small.
- **Left out:**
  - A sleep, retry, longer timeout, removed assertion, or no change hides
    the ordering defect or leaves it present.
  - Removing the hold moves the race into the disabled-control checks.
  - Holding the app request with `page.route` replaces the fixture API.
    The browser checks route only third-party traffic.
  - A release that waits for registration can wait forever if no request
    arrives. A browser request event does not prove callback registration.
  - Moving the lock checks into Node loses rendered-control coverage.
    The Node store tests do not render the app.
- **Wider fix:** B. A scenario-owned hold object exists before either
  caller runs. Its type has no held state without a release function.
- **Previous examples:** None found for this lost-release class.
- **Related:** `ts/app/scripts/browser-boat-account.ts` also owns
  `overflow-rejected-disconnect`. Leave that journey unchanged.
- **Recommendation:** B. The shared promise preserves an early release
  without a readiness poll or a second state field.
- **Verification:**
  - Preserve every existing browser assertion and the hold through all
    three disabled-control checks.
  - Run the timer-free event-order probe before and after the fix. Test
    release before the body read and release after hold registration.
    One release must complete the request in either order after the fix.
  - Repeat `overflow-change-key-pending` at 1440, 720, and 390 pixels.
    Record the pass and failure counts before and after the fix. Report
    when normal repeats do not reproduce the hang.
  - Use the fixture and browser commands in `ts/app/README.md`. No live
    credentials are needed.
  - Add no permanent test of fixture internals, retry, sleep, skip, removed
    assertion, or longer timeout. Keep the app's pending behavior unchanged.
- **Decision:** On 8 November 2026, the user selected B for work after merge.
- **Action:** fix after merge, apply B
