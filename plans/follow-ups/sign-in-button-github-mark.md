# Sign-in buttons lack the mockups' GitHub mark

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, an item that
  the screens milestone found while it built the terms step.
- **Location:** `ts/app/src/view/access.ts`, the actions of `signInPage`
  and `termsStepPage`. `ts/app/src/components/icons.tsx`.
  `docs/mockups/src/screens/auth/sign-in.tsx` and
  `docs/mockups/src/screens/auth/terms-step.tsx`, the `icon={Github}` of
  their buttons.
- **Reason:** The fix decides which side of a mockup and product difference
  is right, and no record says why the dashboard left the mark out.

- **Terms:**
  - **Terms step:** the dashboard page at `/continue` with the terms notice
    and a "Continue with GitHub" button. Every GitHub sign-in that does not
    start on the sign-in page passes it first.
  - **GitHub mark:** the GitHub logo icon. The mockups take it from
    `lucide-react` as `Github`.
- **Background:**
  - The mockups of the sign-in page and of the terms step show the GitHub
    mark at the start of their main button.
  - The dashboard's "Sign in with GitHub" button has shown no icon since
    commit `20c0302`, which added the sign-in page.
  - The dashboard's icon set in `ts/app/src/components/icons.tsx` has no
    GitHub mark. Its icons are inline SVG elements.
  - The terms step button of the dashboard follows the sign-in button, so
    both dashboard pages look the same. Both differ from their mockups.
- **Problem:** Two dashboard buttons differ from the mockups, which are the
  spec of the screens.
- **Impact if no action:** The sign-in page and the terms step keep a small
  visual difference from the spec. They work the same.
- **Root question:** Do the buttons that start a GitHub sign-in show the
  GitHub mark?
- **Options:**
  - **A.** Change nothing, and accept the difference on both pages.
    Effort: Small.
  - **B.** Add the GitHub mark to the dashboard icons, with its licence,
    and show it on both buttons. Effort: Small.
  - **C.** Remove the mark from both mockups. Effort: Small.
- **Left out:** A mark on the terms step only, because the two pages that
  start a GitHub sign-in would then look different.
- **Wider fix:** None, because this is one icon on two buttons.
- **Previous examples:** None found.
- **Recommendation:** B. The mockups are the spec, and both pages then
  match them.
- **Action:** ask the user, because the fix decides which side of a mockup
  and product difference is right, and no record says why the dashboard
  left the mark out.
