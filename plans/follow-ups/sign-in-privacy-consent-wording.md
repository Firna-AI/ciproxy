# Sign-in footer reads as privacy consent

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, a problem
  outside the scope that the checks of review findings 2 and 3 found.
- **Location:** `ts/app/src/components/terms-notice.tsx`, near line 11.
  `docs/mockups/src/screens/auth/terms-notice.tsx`, near line 11.
- **Reason:** The sign-in screen and its mockup are outside the legal text
  change.

- **Background:**
  - The terms notice says "By signing in, you agree to the ciproxy Terms
    and Privacy policy".
  - Since 22 November 2026, one shared part holds the notice: `TermsNotice`
    in `ts/app/src/components/terms-notice.tsx` for the dashboard, and in
    `docs/mockups/src/screens/auth/terms-notice.tsx` for the mockups. The
    sign-in page and the terms step at `/continue` both show it in their
    footer. The terms step is the page before each GitHub sign-in that does
    not start on the sign-in page.
  - The privacy policy is a notice. It relies on the contract, legitimate
    interests, and legal obligations, not on consent.
- **Problem:** "Agree to the Privacy policy" reads as consent, which the
  policy does not ask for.
- **Impact if no action:** A person can think that ciproxy uses their data
  because they consented, and that a withdrawal of consent stops that use.
- **Root question:** How does the sign-in page ask for acceptance of the
  terms and point to the privacy policy without asking for consent?
- **Options:**
  - **A.** Change the footer to "By signing in, you agree to the ciproxy
    Terms. The Privacy policy explains how we use your data." Change the
    mockup first. Effort: Small.
  - **B.** Keep the footer. Effort: Small.
- **Left out:** A consent box fails the goal, because the policy relies on no
  consent.
- **Wider fix:** None, because this is one line of copy.
- **Previous examples:** None found.
- **Recommendation:** A. It keeps the acceptance of the terms and states the
  policy as a notice.
- **Action:** fix after merge, apply A, because the sign-in screen and its
  mockup are outside the legal text change.
