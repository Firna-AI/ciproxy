# Repository gate skips the dashboard tests

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  1, a problem outside the scope that the check of finding 4 raised.
- **Location:** `xtask/src/check/phases.rs:96-104`, the TypeScript phase of
  the gate. `.github/workflows/ci.yml`, the CI jobs. `ts/package.json`, the
  root scripts. `ts/app/package.json`, the `test` script. `ts/app/_tests_/`,
  the Node tests. `ts/app/scripts/`, the browser smokes and the fixture
  harnesses. `xtask/README.md`, the phase table. `ts/app/README.md`, under
  "Development".
- **Reason:** The gap is older than the branch that found it, and new checks
  in the gate or a new CI job need the approval of the user.

- **Terms:**
  - **Complete gate:** `cargo xtask check`, the repository check. The CI job
    "Repository gate" runs it for each push and pull request.
  - **Dashboard Node tests:** the tests in `ts/app/_tests_`.
    `npm --prefix ts --workspace @ciproxy/app run test` runs them on the
    Node test runner.
  - **Browser smokes:** three scripts in `ts/app/scripts` that drive
    headless Chrome through the dashboard pages: `browser-smoke.ts`,
    `browser-pages-smoke.ts`, and `m20-browser-smoke.ts`.
  - **Fixture harness:** a Node server in `ts/app/scripts` that serves the
    dashboard export in `ts/app/dist` with a typed fixture API and no live
    API.
- **Background:**
  - The complete gate has twelve phases. Its TypeScript phase runs
    `tokens:check`, `app:lint`, `app:typecheck`, `site:lint`,
    `site:typecheck`, `api:lint`, and `api:typecheck`, at
    `xtask/src/check/phases.rs:96-104`.
  - No phase runs the dashboard Node tests or the browser smokes. The
    `app-export` phase only builds the pages.
  - `ts/app/README.md` says: "The complete gate lints, typechecks,
    format-checks, and exports the app. It does not run the Node tests or
    the browser checks below." Commit `20c0302` of 6 November 2026 added
    this sentence. So the gap is older than the branch that found it.
  - `.github/workflows/ci.yml` has two jobs. "Repository gate" runs
    `cargo xtask check --keep-going` on `ubuntu-24.04`, after
    `npm --prefix ts ci`. "macOS crates" builds and tests only the macOS
    crates. So no CI job runs the dashboard tests.
  - The `test` script of `ts/app/package.json` runs
    `node --test "_tests_/**/*.test.ts"`. `ts/package.json` has no root
    script for it. `ts/tokens` and `ts/site` have a `test` script too. The
    gate runs the tokens tests through `tokens:check`, and not the site
    tests.
  - On 22 November 2026, the 258 dashboard Node tests passed in about 8
    seconds.
  - Each browser smoke needs the export in `ts/app/dist`, a running fixture
    harness, Google Chrome, and `playwright-core`. The Machines and Settings
    smoke uses a second harness.
  - A smoke loads `playwright-core` only from the path in
    `CIPROXY_PLAYWRIGHT_PATH`, and it stops without that path.
    `CIPROXY_CHROME_PATH` sets the browser, by default
    `/usr/bin/google-chrome`.
  - `ts/app/scripts/README.md` says that the repository has no browser
    dependency. But `ts/site/package.json` has `playwright-core` `^1.65.0`
    as a development dependency, and `npm --prefix ts ci` installs it in
    `ts/node_modules`.
  - On 22 November 2026, the complete shell smoke ran 294 checks, each of
    its cases at 1440, 720, and 390 pixels. The cases took about 7 minutes
    in total. The page smoke and the Machines and Settings smoke add more.
  - The review of pull request #125 asked for browser smoke checks of the
    new Terms lines in the checkout review and the Admin grant dialog,
    because a model test cannot see the rendered links. CI does not run
    those checks either.
- **Problem:** No CI job runs the dashboard Node tests or the browser
  smokes. A change can break a dashboard view model, a page, or a journey,
  and CI still passes.
- **Impact if no action:** A dashboard regression reaches the primary
  branch unless a person runs the tests and the smokes by hand. For
  example, a change that removes the business-use line of the checkout
  review passes CI.
- **Root question:** Which dashboard checks run automatically for each push
  and pull request?
- **Options:**
  - **A.** Change nothing. The tests and the smokes stay manual. Effort:
    Small.
  - **B.** Add the Node tests to the gate. One root script runs the `test`
    script of every package under `ts`, and the TypeScript phase runs it.
    This adds the dashboard tests, about 8 seconds, and the site tests.
    The tokens tests would run twice, so `tokens:check` can drop them.
    Update `xtask/README.md`, `AGENTS.md`, and `ts/app/README.md`. The
    smokes stay manual. Effort: Small.
  - **C.** Do B, and add a gate phase that runs the three browser smokes
    after the app export. The phase starts both fixture harnesses, runs the
    smokes with the `playwright-core` of `ts/node_modules`, and stops the
    harnesses. Each gate run, also on a local machine, then needs Google
    Chrome and takes many minutes more. Effort: Medium.
  - **D.** Do B, and run the three browser smokes in a separate CI job with
    the same steps. The local gate stays about as fast as now and needs no
    browser. A broken smoke then shows only in CI, after the push. Effort:
    Medium.
- **Left out:**
  - A root script for the dashboard tests alone: B costs the same, and it
    also runs the site tests and the tests of each new package.
  - A Node test that renders the dashboard screens: it adds new tests
    instead of a run of the existing ones, and no render setup exists.
  - A pre-push hook: it runs only on a machine that installs it, and CI
    still runs nothing.
- **Wider fix:** B for the Node tests. One root script runs the `test`
  script of every package, so the tests of a new package join the gate with
  no change. The browser smokes need C or D.
- **Previous examples:**
  - `plans/follow-ups/site-calculation-tests-in-gate.md` records the same
    class for the site tests.
  - `plans/follow-ups/image-checks-in-gate.md` records the same class for
    the image checks.
- **Related:**
  - `plans/follow-ups/site-calculation-tests-in-gate.md`: the same class
    for `ts/site`. Option B also does its task, because B runs the site
    Node tests.
  - `plans/follow-ups/browser-tests-playwright-source.md`: the same code.
    Options C and D can set `CIPROXY_PLAYWRIGHT_PATH` to the copy in
    `ts/node_modules`. Its option A makes that copy the default.
- **Recommendation:** D. CI then runs every existing dashboard check for
  each push, and a local gate run stays fast and needs no browser. C
  protects more, because a local gate run then also finds a broken smoke
  before the push.
- **Action:** ask the user, because the recommended option adds checks to
  the gate and a new CI job, and the always-on rules reserve a new gate for
  the user.
