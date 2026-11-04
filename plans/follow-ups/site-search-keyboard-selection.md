# Search lacks arrow key selection

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from
  `conductor/ciproxy-mvp-milestone-23-site`.
- **Location:** `ts/site/src/styles/starlight/`, and
  `docs/mockups/src/screens/docs/search-dialog.tsx`.
- **Reason:** The approved site task permits CSS changes to the existing
  search and defers gaps that need new search behaviour.

- **Terms:**
  - **Starlight:** The Astro docs framework used by `ts/site`.
  - **Pagefind:** The static search engine and result interface used by
    Starlight.
- **Background:** The docs search mockup selects the first heading result.
  Its footer shows Up and Down to move, and Enter to open the selected
  result. The site keeps the real Starlight and Pagefind search.
- **Problem:** Pagefind's default interface supports Tab between links.
  It does not select results with arrow keys. Its search input prevents
  Enter from submitting and does not open a selected result.
- **Impact if no action:** Keyboard users must use Tab to reach a result.
  The search cannot show the mockup's selection and truthful keyboard
  hints together.
- **Root question:** How can keyboard users select and open search results
  as the approved mockup shows?
- **Options:**
  - **A.** Keep Tab navigation and omit the unsupported keyboard footer.
    This preserves a usable search but keeps the design gap. Effort: Small.
  - **B.** Add a Starlight search override using the Pagefind search API.
    Own the active result, arrow keys, Enter, focus, and keyboard hints in
    that component. Keep real index results and ordinary link behaviour.
    Effort: Medium.
- **Left out:** CSS cannot add keyboard behaviour. Replacing Pagefind adds
  another search engine without a benefit for this requirement.
- **Wider fix:** B. The search component owns its result selection and
  hints, so the displayed keys describe the real behaviour.
- **Previous examples:** The installed `@pagefind/default-ui` input handler
  handles Escape and prevents Enter. It has no arrow key result handler.
  Starlight's Search component supplies opening and closing only.
- **Related:** `site-search-heading-count.md` covers the result count in
  the same search component.
- **Recommendation:** B. It implements the approved keyboard controls with
  the existing search engine.
- **Action:** fix after merge, apply B, because keyboard behaviour requires
  a search component change beyond the approved CSS scope.
