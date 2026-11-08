# Browser checks load Playwright in two ways

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from pull request #55, orchestrator check of fix
  round 1.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `ts/app/scripts/browser-driver.ts`, `ts/app/README.md`, and
  `ts/site/_tests_/browser/search.test.ts`.
- **Reason:** The change of the app scripts is outside the files of #55.
- **Terms:**
  - **Playwright:** the browser automation library that the browser checks
    use. Its `playwright-core` package uses an installed Chrome.
- **Background:**
  - The app browser checks load `playwright-core` from the path in
    `CIPROXY_PLAYWRIGHT_PATH`, which the operator supplies.
  - Pull request #55 adds `playwright-core` as a development dependency of
    `ts/site`, and its browser tests import it.
- **Problem:** Two packages load the same library in two ways, with two
  setup steps in their guides.
- **Impact if no action:** Each browser check needs its own setup, and the
  two ways can drift apart.
- **Root question:** How do the browser checks get Playwright?
- **Options:**
  - **A.** The app scripts import the `playwright-core` development
    dependency by default. `CIPROXY_PLAYWRIGHT_PATH` stays as an override.
    Update `ts/app/README.md`. Effort: Small.
  - **B.** Change nothing. Effort: Small.
- **Left out:** Removing `CIPROXY_PLAYWRIGHT_PATH`, because it deletes an
  operator setting that is on the primary branch.
- **Wider fix:** A.
- **Previous examples:** None found.
- **Recommendation:** A. One default source serves every browser check.
- **Action:** fix after merge, apply A, because the app scripts are
  outside the files of #55.
