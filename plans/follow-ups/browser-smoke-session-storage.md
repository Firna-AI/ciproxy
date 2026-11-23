# Browser smokes fail on sessionStorage access

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, the dashboard screens
  work.
- **Location:** the init script of the shell and page browser smokes under
  `ts/app/scripts/`, as `ts/app/scripts/README.md`, "Browser checks", names
  them.
- **Reason:** The same cases fail on commit `64211612`, before the provider
  screens changed, so the fault is older than that change.

- **Source:** Implementation.
- **Category:** Test.
- **Background:**
  - The dashboard has browser smokes that drive headless Chrome against a
    fixture harness, at the widths 1440, 720, and 390 px.
  - On 23 November 2026, the shell smoke failed 15 cases at 3 widths, 45
    checks of 297: the `root-*` cases (11), the `late-*` cases (3), and
    `auth-expired`.
  - The page smoke failed `viewer-saved-checkout-usage` and
    `viewer-saved-checkout-plans` at 3 widths, 6 checks of 318.
  - Each fails with "The page emitted a JavaScript or console error":
    `SecurityError: Failed to read the 'sessionStorage' property from
    'Window'`, raised in the init script of the smoke itself.
  - The same cases fail on commit `64211612`. Once, `auth-expired` at 390 px
    passed there.
- **Problem:** The smoke's own init script reads `sessionStorage` on a page
  where Chrome forbids it, such as an opaque origin or a page before the
  app's origin loads. So these cases test nothing about the app.
- **Impact if no action:** 51 browser checks always fail, so a real fault
  in those pages stays hidden, and each run needs a manual baseline
  comparison.
- **Root question:** How does the smoke prepare browser storage only on a
  page where the app's origin allows it?
- **Options:**
  - **A.** Guard the init script: skip the storage step when the page has no
    usable origin, and set the storage after the first app navigation.
    Effort: Small.
  - **B.** Seed the storage through the Chrome DevTools protocol for the
    app's origin before the first navigation. Effort: Small.
- **Left out:** Removing the cases, because they cover the root, late, and
  expired-session pages.
- **Wider fix:** B. Storage set for an origin works for every case,
  whatever page loads first.
- **Previous examples:** None found.
- **Recommendation:** B. It sets the storage where Chrome allows it and
  keeps the pages under test unchanged.
- **Action:** fix after merge, apply B, because the fault is older than this
  branch and outside its scope.
