# Search counts pages instead of headings

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from
  `conductor/ciproxy-mvp-milestone-23-site`.
- **Location:** `ts/site/src/styles/starlight/`, and
  `docs/mockups/src/screens/docs/search-dialog.tsx`.
- **Reason:** The approved site task permits CSS changes to the existing
  search and defers gaps that need new search behaviour.

- **Terms:**
  - **Pagefind:** The static search engine and result interface used by the
    Starlight docs framework in `ts/site`.
- **Background:** The docs search mockup shows four matching headings under
  three page groups. Its count is the number of matching headings.
  Starlight uses Pagefind's default interface for real search results.
- **Problem:** The default interface counts matching pages through
  `searchResult.results.length`. It does not count the heading matches of
  each page. CSS cannot change that computed value.
- **Impact if no action:** The result count differs from the mockup when a
  page has more than one matching heading.
- **Root question:** How should the search count the heading results that
  a reader can open?
- **Options:**
  - **A.** Retain the default page count. This keeps the design gap but
    preserves the real search results. Effort: Small.
  - **B.** Count heading matches in a search component using the Pagefind
    search API. Render the count and the grouped results from the same
    result collection. Effort: Medium.
- **Left out:** CSS cannot compute a total from search data. A different
  search engine adds migration work that this requirement does not need.
- **Wider fix:** B. One result collection supplies both the count and the
  rows, so the two stay consistent.
- **Previous examples:** The installed `@pagefind/default-ui` component
  uses `searchResult.results.length` for its one-result and many-results
  messages. Its grouped result component renders heading matches below
  each page.
- **Related:** `site-search-keyboard-selection.md` covers selection and
  keyboard controls in the same search component.
- **Recommendation:** B. It makes the count agree with the openable
  heading results without replacing the search engine.
- **Action:** fix after merge, apply B, because heading counts require a
  search component change beyond the approved CSS scope.
