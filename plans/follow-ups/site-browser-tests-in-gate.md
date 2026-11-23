# Repository gate runs no site browser tests

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-23, from the branch `calummoore/diagram-request-3jpv`,
  the blocked wider fix of review finding 4.
- **Source:** Review.
- **Category:** Test.
- **Location:** `xtask/src/check/phases.rs`, `ts/package.json`,
  `ts/site/package.json`, and `ts/site/_tests_/browser/`.
- **Reason:** A new gate check needs the user's approval.
- **Related:** `plans/follow-ups/site-calculation-tests-in-gate.md` has the
  same cause: the gate runs no site test. That item connects the Node
  tests. This item covers the browser tests. One branch can do both.
  `plans/follow-ups/site-search-focus-test-result-order.md` records a
  failing search browser test that must pass first.
- **Terms:**
  - **Gate:** `cargo xtask check`, the repository checks that CI runs on
    each push and pull request.
  - **Browser test:** a test under `ts/site/_tests_/browser/` that builds
    the site, serves it, and drives Google Chrome with `playwright-core`.
- **Background:**
  - The site browser tests run with
    `npm --prefix ts --workspace @ciproxy/site run test:browser`. Nobody
    runs them automatically.
  - The branch above added a browser test that checks the layout of the job
    path figure at 720 px and at 840 px. It fails when the figure takes its
    layout from the window width.
  - The search focus tests are in the same folder.
- **Problem:** A change can break a layout or a focus rule that a browser
  test covers, and CI still passes.
- **Impact if no action:** The browser tests protect only the branches whose
  agents remember to run them.
- **Root question:** Should each push run the site browser tests?
- **Options:**
  - **A.** Keep the browser tests manual. Name them in the site README as a
    step for each site change. Effort: Small.
  - **B.** Add a gate step that runs `test:browser` in the existing
    TypeScript or site phase. Install Google Chrome in CI. Fix the failing
    search test first. Each gate run builds the site once more and needs
    Chrome, also on a local machine. Effort: Medium.
  - **C.** Run the browser tests in a separate CI job, not in the local
    gate. A local gate run then does not need Chrome, but the local gate no
    longer proves all of CI. Effort: Medium.
- **Left out:** A screenshot baseline, because the site has no image compare
  tool and no CI job makes the baselines.
- **Wider fix:** B, because every push then runs every browser test.
- **Previous examples:** `plans/follow-ups/site-calculation-tests-in-gate.md`.
- **Recommendation:** B. It makes the existing tests count on every push.
- **Action:** ask the user, because the recommended option adds a gate
  check, which the user approves.
