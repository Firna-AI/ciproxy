# A site search focus test fails on main

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, which
  completes the legal pages and found the failure in a site test run.
- **Location:** `ts/site/_tests_/browser/search.test.ts`, the test "same-page
  search continues keyboard focus at the opened heading", near line 32.
- **Reason:** The failure comes from docs changes that merged before the
  legal pages branch, so its fix belongs in its own branch from `main`.

- **Terms:**
  - **Site browser tests:** the Node tests under `ts/site/_tests_/browser/`.
    They build and serve the production site, and drive Google Chrome with
    `playwright-core`.
- **Background:**
  - The test opens `/docs/`, opens the docs search, and types `runs-on`.
  - It presses ArrowUp, which selects the last result, and then Enter.
  - It then waits for the address `/docs/#how-a-job-runs`, and checks that
    Tab continues at that heading.
  - The repository gate does not run the site browser tests. They run by
    hand with `npm --prefix ts --workspace @ciproxy/site run test:browser`.
    The follow-up item `site-calculation-tests-in-gate.md` covers the site
    tests that the gate leaves out.
- **Problem:** The test times out on `main` at commit `c5cd00b`, with this
  failure text:

  ```text
  page.waitForURL: Timeout 30000ms exceeded.
  waiting for navigation to "http://127.0.0.1:<port>/docs/#how-a-job-runs" until "load"
  ```

  - It failed in four of four runs on 22 November 2026, with Google Chrome
    158.0.8127.36, on `c5cd00b` and on the legal pages branch.
  - Both site browser tests pass at commit `5105713`, the first parent of
    the merge commit `c5cd00b`.
  - That merge added macOS text to the docs pages Overview, Add a machine,
    Quickstart, Agent install options, Labels and VM sizes, and Plans and
    starts.
  - Source: that text changed the results of the query `runs-on`. A replay
    in Chrome on 22 November 2026 showed that ArrowUp now selects
    `/docs/reference/labels/#default-labels`, not the heading "How a job
    runs" of `/docs/`.
- **Impact if no action:** The site browser tests fail for every branch, so
  a real regression of the search focus can hide behind this failure.
- **Root question:** How does the test open a heading of the current page
  in a way that a change to other docs pages does not break?
- **Options:**
  - **A.** Select the result by its name. The test presses ArrowDown until
    the active result is the heading "How a job runs", then presses Enter.
    The test then depends on the name, not on the order. Effort: Small.
  - **B.** Change the query to words that only the heading "How a job runs"
    of `/docs/` holds. A later docs page with the same words breaks the
    test again. Effort: Small.
  - **C.** Keep ArrowUp, and change the expected address to the new last
    result. The next docs change can break the test again. Effort: Small.
- **Left out:** A change of the docs text, because the docs text is right
  and the test must follow the docs.
- **Wider fix:** A. It removes the dependence on the order of the results
  for this test.
- **Previous examples:** None found.
- **Recommendation:** A. The test keeps its purpose and no longer breaks
  when another page adds the query words.
- **Action:** fix after merge, apply A, because the failure is outside the
  legal pages change, and one fix from `main` reaches every open branch.
