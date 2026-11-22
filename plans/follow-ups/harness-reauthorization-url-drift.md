# Five harness sessions lack the reauthorization URL

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, a problem
  outside the scope that the work on the fixtures of the terms step found.
- **Location:** `ts/app/scripts/harness-scenarios.ts:184`,
  `ts/app/scripts/harness-scenarios.ts:210`,
  `ts/app/scripts/harness-scenarios.ts:219`,
  `ts/app/scripts/fixtures-core.ts:85`,
  `ts/app/scripts/browser-install.ts:59`, and
  `crates/ciproxy-api/src/handlers/sessions.rs:138`.
- **Reason:** The drift is older than the branch that found it, and its fix
  moves a browser smoke that the terms step work does not change.
- **Source:** Implementation.
- **Category:** Test.
- **Terms:**
  - **Fixture harness:** `ts/app/scripts/harness-server.ts` serves the
    dashboard export with a fake API. Each scenario in
    `ts/app/scripts/harness-scenarios.ts` sets its responses.
  - **Check again:** a button on the dashboard pages that show in place of
    a workspace, such as the install page. It checks the access again.
  - **Reauthorization URL:** the `reauthorization_url` field of the session
    that `GET /api/session` returns. Check again can leave for it.
  - **Terms step:** the dashboard page at `/continue` that shows the terms
    notice before a GitHub sign-in. `pages.termsStep` in
    `ts/app/src/routing/pages.ts` names it.
  - **Build check:** each API response names its build in the
    `x-ciproxy-build` header. After a new build, an install action must
    still send its request before the page changes.
- **Background:**
  - The API gives the reauthorization URL `/continue` for every session
    whose state is not `ready`, and no URL for a `ready` session.
  - `crates/ciproxy-api/src/handlers/sessions.rs` sets both from
    `principal.browser_only()`. Lines 138 to 147 give the state
    `app_removed`, `access_ended`, or `install_required` only for a
    browser-only sign-in. Line 150 gives `/continue` only then.
  - Before 22 November 2026, the API gave `/api/auth/github` in the same
    place. The five scenarios below gave no URL then too, so the drift is
    older than the terms step.
  - On a page whose sign-in has no workspace, Check again leaves for the
    reauthorization URL. Without a URL, it reads the session again.
    `checkAgainTarget` in `ts/app/src/view/access.ts` holds this rule.
  - `session()` in `ts/app/scripts/fixtures-core.ts` gives no URL by
    default, at line 85. `recordedScenario` in
    `ts/app/scripts/harness-scenarios.ts` builds each scenario from it, and
    sets the state afterwards.
  - Five scenarios set a state that is not `ready` and keep no URL.
    `install`, `install-pending`, and `personal` get `install_required` at
    line 184. `app-removed` gets `app_removed` at line 210. `access-ended`
    gets `access_ended` at line 219.
  - The other scenarios with such a state already give `/continue`:
    `install-reauthorize`, `late-first-wait`, `root-removed-old-claim`,
    `root-removed-old-outcome`, `root-reinstall-open`, and
    `root-access-recovery`.
- **Problem:** In the five scenarios, the harness gives a session that the
  API never gives. Two pages have Check again: the install page while it
  waits for an owner, in `install-pending`, and the ended-access page, in
  `access-ended`. There, Check again reads the session again in the
  harness, but opens the terms step in production.
- **Impact if no action:** The browser smokes do not run the production
  path of Check again on those two pages. The smoke
  `install-native-navigation` checks the build check through a session read
  that the install page never makes in production.
- **Evidence:**
  - `installNavigation` in `ts/app/scripts/browser-install.ts` runs as
    `install-native-navigation`, which `ts/app/scripts/browser-registry.ts`
    registers. It uses `install-pending` on `/install`.
  - Its first part checks the "Install on another organization" link. A
    modified click opens `/install` in a new tab and sends no request. A
    plain click there sends one `POST /api/installation-links/start`.
  - Its second part starts at line 59. It sets the harness build to
    `fixture-build-b` and clicks Check again. It waits for the
    `/api/session` response and asserts `x-ciproxy-build: fixture-build-b`.
    Then a plain click on the install link must send the POST before the
    page changes.
  - On 22 November 2026, one run gave `install-pending` the URL
    `/continue`. `install-native-navigation` failed at 1440, 720, and 390
    pixels, while it waited for `/\/api\/session$/`:

    ```text
    page.waitForResponse: Timeout 30000ms exceeded while waiting for event "response"
    ```

    The case `install-pending` still passed. With the scenario restored,
    both passed.
  - The shell cases `install`, `install-pending`, `personal`, `app-removed`,
    and `access-ended` in `ts/app/scripts/browser-cases.ts` do not click
    Check again. The two other uses of the URL, the install page and the
    landing, also need `reauthorization_required`, which these scenarios
    keep false.
  - The page of a person with no role fits the second part. The scenario
    `no-role` on `/jobs?workspace=10000000-0000-4000-8000-000000000001`
    shows the heading "Waiting for an Admin of Northstar", Check again, and
    "Install on another organization". Its sign-in is `ready`, so Check
    again reads the session, also in production.
    `failedSessionCheck` in `ts/app/scripts/browser-review-actions.ts`
    already counts that read.
- **Root question:** How can each harness session give the reauthorization
  URL that the API gives, while the build check of the install action keeps
  a page that reads the session?
- **Options:**
  - **A.** Give each of the five scenarios `pages.termsStep`. Move the
    second part of `install-native-navigation` to the `no-role` scenario.
    A later scenario with such a state can drift again. Effort: Small.
  - **B.** At the end of `recordedScenario`, give each session whose state
    is not `ready` the URL `pages.termsStep` when it has none. Move the
    second part of the smoke as in A. One rule then covers every scenario,
    new ones too. Effort: Small.
  - **C.** Do B. Also make the default of `session()` in
    `ts/app/scripts/fixtures-core.ts` follow its `state` argument, so the
    Node tests follow the rule too. `ts/app/_tests_/access.test.ts` and
    `ts/app/_tests_/session.test.ts` build such sessions. Their assertions
    do not read the URL, but each needs a check. Effort: Small.
- **Left out:**
  - Change nothing, with a comment that these sessions skip the URL on
    purpose: the harness would still give sessions that the API never
    gives.
  - One extra scenario with no URL, only for `install-native-navigation`:
    it keeps a session that the API never gives.
  - A timer that makes the install page read the session again: it changes
    the product only for a smoke.
  - A test that compares each harness session with the API rule: a test
    for the class needs the user's approval.
  - A default in `session()` alone: the harness sets the state after
    `session()` returns, so the default does not reach the five scenarios.
- **Wider fix:** B. One rule in the harness follows the API for every
  scenario. Option C also reaches the Node tests, which pass their own URL
  where it matters.
- **Previous examples:** On 22 November 2026, the terms step work changed
  four fixture values by hand from `/api/auth/github` to `/continue`:
  `lateApprovalWait()` in `ts/app/scripts/fixtures-late-approval.ts`,
  `removedWithOldClaim()` in `ts/app/scripts/fixtures-returns.ts`, the
  scenario `install-reauthorize`, and the default of `link()` in
  `ts/app/scripts/fixtures-core.ts`.
- **Recommendation:** B. One rule keeps every scenario in line with the
  API, and the build check moves to a page where Check again reads the
  session in production too.
- **Verification:**
  - Keep every assertion of `installNavigation`. Keep its new-tab part on
    `install-pending`.
  - See the moved build check fail once first. The failure above, on
    `install-pending` with `/continue`, is the expected first failure.
  - Run `npm --prefix ts run app:export`, then
    `node ts/app/scripts/harness-server.ts --port=4340`. Set
    `CIPROXY_PLAYWRIGHT_PATH` to a `playwright-core/index.mjs`.
  - Run this command 10 times, and record the pass counts:

    ```sh
    node ts/app/scripts/browser-smoke.ts \
      --case=install-native-navigation,install,install-pending,personal,app-removed,access-ended
    ```

  - Add no retry, sleep, skip, or longer timeout.
- **Action:** fix after merge, apply B, because the drift is older than the
  branch that found it, and its fix changes a browser smoke outside the
  terms step work.
