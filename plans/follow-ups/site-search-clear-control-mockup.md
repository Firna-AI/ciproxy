# Search mockup omits the clear control

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-04, from
  `conductor/ciproxy-mvp-milestone-23-site`.
- **Location:** `docs/mockups/src/screens/docs/search-dialog.tsx`,
  `docs/mockups/src/screens/docs/search.css`, and
  `ts/site/src/styles/starlight/search.css`.
- **Reason:** The site task keeps the real search controls and prohibits
  changes to the mockups.

- **Terms:**
  - **Pagefind:** The search engine and result interface used by the
    Starlight docs framework in `ts/site`.
- **Background:** Pagefind shows a clear button when the query has text.
  The site styles that button as a small cross. Cancel and Escape close
  the dialog, while the clear button clears the query.
- **Task:** Show the existing clear control in the active-query search
  mockup at desktop and phone widths. Keep its purpose distinct from
  closing the dialog. Preserve the working clear action in the site.
- **Impact if no action:** The mockup omits a working control that remains
  visible in the real search dialog.
- **Needs:** A change that owns the docs search mockups. The site task of
  4 November 2026 permits only CSS changes to the real search.
- **Done when:** The active-query mockup shows the clear control beside
  the query. Its position matches the existing site at 1440, 720, and
  390 pixels.
