# Terms lines differ from their mockups in look

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  1, a problem that the check of finding 5 found inside the scope.
- **Location:** `ts/app/src/components/inline-link.tsx:30`, the style of
  `InlineLink`. `ts/app/src/screens/billing/checkout-summary.tsx:150` and
  `:174`, the `tight` gap.
  `ts/app/src/screens/settings/grant-admin-dialog.tsx:93`.
  `ts/app/src/components/terms-notice.tsx:12`.
  `docs/mockups/src/styles/base.css:69`, the class `.text-link`, and
  `:125`, the class `.stack-tight`.
  `docs/mockups/src/screens/billing/business-use-note.tsx`,
  `docs/mockups/src/screens/settings/grant-admin-dialog.tsx`, and
  `docs/mockups/src/screens/auth/terms-notice.tsx`.
- **Reason:** The fix decides which side of a mockup and product difference
  is right, and no record says why the dashboard look differs.

- **Terms:**
  - **Mockups:** the clickable screen designs under `docs/mockups`, which
    Mokly renders with react-dom and CSS classes. They are part of the spec
    of the dashboard screens.
  - **Grant dialog:** the dashboard dialog that gives a member the Admin
    role.
  - **Checkout review:** the dashboard page that shows the price of a plan
    change before the person pays.
  - **Business-use line:** the line "Paid plans are for business use only,
    as the ciproxy Terms say." of each checkout review with a pay button.
  - **Terms notice:** the line "By signing in, you agree to the ciproxy
    Terms and Privacy policy." on the sign-in page and on the terms step at
    `/continue`.
- **Background:**
  - On 22 November 2026, pull request #125 added two lines with a Terms
    link to the dashboard. The grant dialog says that an Admin also accepts
    new versions of the Terms for the customer. The checkout review got the
    business-use line. Both Terms links open a new tab.
  - Both links use `InlineLink` of `ts/app/src/components/inline-link.tsx`.
    It shows the link in `colors.ink`, `#0A0A0A`, in the weight of the text
    around it, with a plain underline. It has no hover look.
  - Their mockups use `TextLink` and the class `.text-link` of
    `docs/mockups/src/styles/base.css`. It shows the link in `colors.ink2`,
    `#525252`, with weight 500 and an underline in `colors.border2`,
    `#DEDEDE`, with a 3 px offset.
  - On hover, `.text-link` changes to `colors.ink`, and its underline takes
    the text colour.
  - In `ts/app/src/screens/billing/checkout-summary.tsx`, the business-use
    line sits 4 px below the charge note, in a view with the style
    `tight: { gap: 4 }`.
  - The mockups put the charge note and the business-use line in a
    `.stack-tight` block, which has a 6 px gap.
    `docs/mockups/src/screens/billing/checkout-review.tsx` and
    `docs/mockups/src/screens/billing/api-review.tsx` do this.
  - Before pull request #125, the terms notice of the sign-in page had the
    same link look difference. `ts/app/src/screens/sign-in.tsx` gave its
    two links `{ color: colors.ink, textDecoration: "underline" }`. Commit
    `20c0302` of 6 November 2026 added the page with this look. Its mockup
    uses `TextLink`.
  - Since pull request #125, the sign-in page and the terms step share
    `TermsNotice` of `ts/app/src/components/terms-notice.tsx`, which also
    uses `InlineLink`. So a change of `InlineLink` changes all three
    notices.
  - No dashboard link part has the mockup look.
    `ts/app/src/components/text-link.tsx` and the link of
    `ts/app/src/components/feature-parts.tsx` show an underline only on
    hover.
  - No plan or doc records why the dashboard look differs from the mockups.
    The comment of `InlineLink` says only that the link takes the font and
    the size of the text around it, and stays underlined, so it reads as a
    link among the words.
- **Problem:** The two new dashboard lines differ in look from their
  mockups. Their Terms links use the underlined ink look of the dashboard,
  not the `.text-link` look. The business-use line sits 4 px below the
  charge note, where the mockups use 6 px.
- **Impact if no action:** The grant dialog and the checkout review keep a
  small visual difference from the spec. They work the same.
- **Root question:** Which look do the Terms links and the business-use
  line take, so that the dashboard and its mockups agree?
- **Options:**
  - **A.** Change nothing, and accept both differences. Effort: Small.
  - **B.** Change the dashboard to the mockup look. Give `InlineLink` the
    look of `.text-link`: `colors.ink2`, weight 500, an underline in
    `colors.border2` with a 3 px offset, and the hover look. Set the gap
    above the business-use line to 6 px. This also changes the terms notice
    of the sign-in page and the terms step. Effort: Small.
  - **C.** Change the mockups to the dashboard look. Give the links of the
    three notices `colors.ink`, the weight of their text, and a plain
    underline. Give the business-use line a 4 px gap. Both need a new
    class, because `.text-link` and `.stack-tight` serve many other
    mockups. Effort: Small.
- **Left out:**
  - A new look for the two new lines only: the terms notice keeps its
    difference, and the three notices then look different from each other.
  - One link part for the mockups and the dashboard: the mockups style with
    CSS classes, and the dashboard styles with style objects in React
    Native Web, so one part cannot serve both.
  - A change of `.text-link` or `.stack-tight` for every mockup: it changes
    many mockup screens outside these three notices.
- **Wider fix:** None, because the difference is one link style and one
  gap.
- **Previous examples:**
  - `plans/follow-ups/sign-in-button-github-mark.md` records the same
    class for the GitHub mark of two buttons.
  - The terms notice has had the same link look difference since commit
    `20c0302`.
- **Related:** `plans/follow-ups/sign-in-button-github-mark.md`, the same
  class: a look difference between a dashboard page and its mockup, also on
  the sign-in page and the terms step. The user can answer both items in
  one message, and each answer stands alone.
- **Recommendation:** B. The mockups are the spec, and one change of
  `InlineLink` and one gap value bring all three notices in line with them.
- **Action:** ask the user, because the fix decides which side of a mockup
  and product difference is right, and no record says why the dashboard
  look differs.
