# Terms lines differ from their mockups in look

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  1, a problem that the check of finding 5 found inside the scope. Review
  round 2 of the same pull request added the grant line facts, the contrast
  figures, and options D, E, and F on the same day, from finding 10 and its
  check.
- **Location:** `ts/app/src/components/inline-link.tsx:30`, the style of
  `InlineLink`. `ts/app/src/screens/billing/checkout-summary.tsx:150` and
  `:174`, the `tight` gap.
  `ts/app/src/screens/settings/grant-admin-dialog.tsx:97`, the grant line,
  and `:106`, the body gap. `ts/app/src/components/text.ts:44`, the style
  `small` of the grant line words.
  `ts/app/src/components/terms-notice.tsx:12`.
  `docs/mockups/src/styles/base.css:18`, the text colour of the mockups,
  `:69`, the class `.text-link`, and `:125`, the class `.stack-tight`.
  `docs/mockups/src/components/dialog.css:44`, the class `.dialog-body`.
  `docs/mockups/src/screens/billing/business-use-note.tsx`,
  `docs/mockups/src/screens/settings/grant-admin-dialog.tsx`, and
  `docs/mockups/src/screens/auth/terms-notice.tsx`. For options E and F:
  `ts/site/src/styles/site.css:76`,
  `ts/site/src/styles/starlight/content.css:47`, and `ts/tokens/index.ts`.
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
    link to the dashboard. The grant dialog got the grant line, which says
    that an Admin also accepts new versions of the Terms for the customer.
    The checkout review got the business-use line. Both Terms links open a
    new tab.
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
  - The grant line is the last part of the body of the grant dialog. The
    body has a 12 px gap, from `body: { gap: 12 }` in the styles of
    `ts/app/src/screens/settings/grant-admin-dialog.tsx`. The mockup
    `Dialog` gives every body a 14 px gap, through the class
    `.dialog-body` of `docs/mockups/src/components/dialog.css`.
  - The grant line words use the style `small` of
    `ts/app/src/components/text.ts`, in `colors.ink2`, `#525252`. The
    mockup line is a `<p className="small">`, which takes the text colour
    of the mockups, `--c-ink`, `#0A0A0A`, from
    `docs/mockups/src/styles/base.css`.
  - Before pull request #125, the terms notice of the sign-in page had the
    same link look difference. `ts/app/src/screens/sign-in.tsx` gave its
    two links `{ color: colors.ink, textDecoration: "underline" }`. Commit
    `20c0302` of 6 November 2026 added the page with this look. Its mockup
    uses `TextLink`.
  - Since pull request #125, the sign-in page and the terms step share
    `TermsNotice` of `ts/app/src/components/terms-notice.tsx`, which also
    uses `InlineLink`. So a change of `InlineLink` changes all three
    notices. The terms step at `/continue` shows the same link look
    difference as the sign-in page.
  - The business-use line and the terms notice are captions, in
    `colors.muted`, `#6B6B6B`. Against those words, the WCAG contrast ratio
    of the dashboard link in `colors.ink` is 3.7:1. That of the `.text-link`
    colour `colors.ink2` is 1.5:1. The `.text-link` underline in
    `colors.border2` has 1.35:1 on white. So the dashboard link stands out
    more from the words around it.
  - One dashboard link part has the mockup look: the underlined look of
    `TextLink` in `ts/app/src/components/text-link.tsx`. That look is
    opt-in, and only the links of the script steps on the job page use it.
    The default look of that `TextLink`, and the link of
    `ts/app/src/components/feature-parts.tsx`, show an underline only on
    hover.
  - No plan or doc records why the dashboard look differs from the mockups.
    The comment of `InlineLink` says only that the link takes the font and
    the size of the text around it, and stays underlined, so it reads as a
    link among the words.
- **Problem:** The links of the two new dashboard lines and of the terms
  notice use the dashboard look, not the `.text-link` look of their
  mockups. The gaps above the business-use line and the grant line are
  4 px and 12 px, where the mockups use 6 px and 14 px. The grant line
  words are ink2, where the mockup uses ink.
- **Impact if no action:** The grant dialog, the checkout review, the
  sign-in page, and the terms step keep a small visual difference from the
  spec. They work the same.
- **Root question:** Which link look, word colour, and gaps do the two
  Terms lines and the terms notice take, so that the dashboard and its
  mockups agree?
- **Options:**
  - **A.** Change nothing, and accept the differences. Effort: Small.
  - **B.** Change the dashboard to the mockup look. Give `InlineLink` the
    look of `.text-link`: `colors.ink2`, weight 500, an underline in
    `colors.border2` with a 3 px offset, and the hover look. Set the gap
    above the business-use line to 6 px, and the body gap of the grant
    dialog to 14 px. Give the grant line words `colors.ink`. This also
    changes the terms notice of the sign-in page and the terms step. The
    link contrast against the caption words then falls from 3.7:1 to
    1.5:1. Effort: Small.
  - **C.** Change the mockups to the dashboard look. Give the links of the
    three notices `colors.ink`, the weight of their text, and a plain
    underline. Give the business-use line a 4 px gap, the body of the grant
    dialog a 12 px gap, and the grant line words `colors.ink2`. Each needs
    a new class, because `.text-link`, `.stack-tight`, and `.dialog-body`
    serve many other mockups. Effort: Small.
  - **D.** Take the gaps from the mockups and the link look from the
    dashboard. In the dashboard, set the gap above the business-use line to
    6 px and the body gap of the grant dialog to 14 px. In the mockups,
    give the three notices the dashboard look, with new classes: links in
    `colors.ink` with a plain underline, and the grant line words in
    `colors.ink2`. This keeps the stronger link contrast of the dashboard,
    3.7:1 against the caption words, where `.text-link` has 1.5:1.
    Effort: Small.
  - **E.** Change `.text-link` of the mockups, and its two copies in the
    site, to the dashboard look. The copies are `.text-link` in
    `ts/site/src/styles/site.css` and the article links in
    `ts/site/src/styles/starlight/content.css`. This changes every text
    link of the mockups, the marketing pages, and the docs. The gaps and
    the grant line words change as in D. Effort: Medium.
  - **F.** Put the link look into `@ciproxy/tokens`, after a choice of B,
    C, D, or E. `InlineLink`, `.text-link`, and its two site copies then
    read one source. The token package then holds a link look beside the
    colours, fonts, and radii, so `ts/tokens/README.md` changes too.
    Effort: Medium.
- **Left out:**
  - A new look for the two new lines only: the terms notice keeps its
    difference, and the three notices then look different from each other.
  - One link part for the mockups and the dashboard: the mockups style with
    CSS classes, and the dashboard styles with style objects in React
    Native Web, so one part cannot serve both. F shares the look through
    the tokens instead.
  - A change of `.stack-tight` or `.dialog-body` for every mockup: it
    changes many mockup screens outside these three notices.
    `plans/follow-ups/dashboard-mockup-spacing-drift.md` holds the body
    gaps of the other dialogs.
- **Wider fix:** F. The link look then has one source, so the dashboard,
  the mockups, and the site cannot drift apart again.
- **Previous examples:**
  - `plans/follow-ups/sign-in-button-github-mark.md` records the same
    class for the GitHub mark of two buttons.
  - The terms notice has had the same link look difference since commit
    `20c0302`.
- **Related:**
  - `plans/follow-ups/sign-in-button-github-mark.md`, the same class: a
    look difference between a dashboard page and its mockup, also on the
    sign-in page and the terms step. The user can answer both items in one
    message, and each answer stands alone.
  - `plans/follow-ups/dashboard-mockup-spacing-drift.md`, the same class
    outside these three notices: the body gaps of the other dashboard
    dialogs, the review summary stack, and the other dashboard text links.
    Both items name the 12 px body gap of the grant dialog, so their
    options can conflict there. The user can answer both items in one
    message.
- **Recommendation:** B. The mockups are the spec, and one change of
  `InlineLink`, two gaps, and one word colour bring all three notices in
  line with them. F best protects the codebase, because the link look then
  has one source.
- **Action:** ask the user, because the fix decides which side of a mockup
  and product difference is right, the options trade link contrast against
  the mockup look, and no record says why the dashboard look differs.
