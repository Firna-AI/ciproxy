# Dashboard gaps and links differ from their mockups

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-22, from pull request #125, the branch
  `conductor/ciproxy-terms-acceptance-and-business-checkout`, review round
  2, a problem outside the scope that the check of finding 10 found.
- **Location:** The body gaps: the style `body` of
  `ts/app/src/screens/settings/grant-admin-dialog.tsx:106`,
  `ts/app/src/screens/settings/organization-dialogs.tsx:183`,
  `ts/app/src/screens/machines/machine-cache.tsx:173`,
  `ts/app/src/screens/billing/move-to-free-dialog.tsx:97`,
  `ts/app/src/screens/overflow/disconnect-dialog.tsx:100`, and
  `ts/app/src/screens/settings/label-dialog.tsx:181`. The summary stack:
  `ts/app/src/screens/billing/checkout-summary.tsx:173`. The links:
  `ts/app/src/components/feature-parts.tsx:128` and
  `ts/app/src/components/text-link.tsx:42`. The mockups:
  `docs/mockups/src/components/dialog.css:44`, the class `.dialog-body`,
  and `docs/mockups/src/styles/base.css:69` and `:119`, the classes
  `.text-link` and `.stack`.
- **Reason:** The differences are older than the branch that found them,
  and the fix decides which side of a mockup and product difference is
  right.

- **Terms:**
  - **Mockups:** the clickable screen designs under `docs/mockups`, which
    Mokly renders with react-dom and CSS classes. They are part of the spec
    of the dashboard screens.
  - **Body gap:** the space between the parts of a dialog body, such as a
    notice and a sentence.
  - **Review summary:** the Summary panel of the checkout review. The
    checkout review is the dashboard page that shows the price of a plan
    change before the person pays.
  - **`.text-link`:** the class of the mockup `TextLink` in
    `docs/mockups/src/components/actions.tsx`. It shows a link in
    `colors.ink2`, `#525252`, with weight 500 and an underline in
    `colors.border2`, `#DEDEDE`, with a 3 px offset. On hover, the link and
    its underline take `colors.ink`, `#0A0A0A`.
- **Background:**
  - The dashboard is the Expo app in `ts/app`. It renders React Native Web
    and styles with style objects. The mockups style with CSS classes.
  - The mockup `Dialog` in `docs/mockups/src/components/dialog.tsx` puts
    every dialog body in `.dialog-body`, which has a 14 px gap.
  - Each dashboard dialog sets its own body gap:
    - 8 px: Disconnect Boat, in `disconnect-dialog.tsx`.
    - 10 px: Move to Free, in `move-to-free-dialog.tsx`.
    - 12 px: the grant dialog that gives the Admin role, the organization
      dialogs, and the clear cache dialog.
    - 16 px: the label dialog.
  - The review summary in `checkout-summary.tsx` stacks its parts with a
    14 px gap, in the style `stack`. Its mockups in
    `docs/mockups/src/screens/billing/checkout-review.tsx` and
    `api-review.tsx` use `.stack`, which has a 16 px gap.
  - The `TextLink` of `ts/app/src/components/feature-parts.tsx` shows an
    underline only on hover, and never in its muted form. It shows a link
    in `colors.ink`, or in `colors.muted` in the muted form.
  - Four of its links have mockups that use the mockup `TextLink`, whose
    `.text-link` always shows an underline:
    - The job links and Job history of the machine page, in
      `ts/app/src/screens/machines/machine-slots.tsx`.
    - Overflow settings of the Overflow panel, in
      `ts/app/src/screens/machines/overflow-panel.tsx`.
    - Organizations of the repositories page of Settings, in
      `ts/app/src/screens/settings/repositories-page.tsx`. This link uses
      the muted form.
  - Most other dashboard text links agree with their mockups on the
    underline. The machine names of the machine table and the job names of
    the job list underline on hover, as the mockup classes `.machine-link`
    and `.job-name-title` do.
  - One small link difference stays. The breadcrumbs of `DetailHeader`, on
    the job page, the plans page, and the checkout review, use the
    `TextLink` of `ts/app/src/components/text-link.tsx`, which underlines on
    hover. Their mockup class `.crumb` shows no underline. This item covers
    that difference too.
  - Commits `58b12be`, `69b4f1c`, and `166b2c9` of 8 November 2026 added
    these values, so they are older than pull request #125. No plan or doc
    records why they differ from the mockups.
  - The Terms links of the grant dialog and the checkout review, and the
    gaps above those two lines, belong to
    `plans/follow-ups/terms-lines-mockup-look.md`. The 12 px body gap of
    the grant dialog is also the gap above its line about the Terms.
- **Problem:** Six dashboard dialogs use body gaps of 8 to 16 px, where the
  mockup `Dialog` gives every body 14 px. The review summary stacks its
  parts 14 px apart, where `.stack` uses 16 px. Four dashboard text links
  show an underline only on hover or never, where their mockups always
  show the underline of `.text-link`.
- **Impact if no action:** Many dashboard screens keep small visual
  differences from their spec. They work the same.
- **Root question:** Which side sets the dialog body gaps, the summary
  stack, and the link look, so that the dashboard and its mockups agree?
- **Options:**
  - **A.** Change nothing, and accept the differences. Effort: Small.
  - **B.** Change the dashboard to the mockups. Give every dialog body a
    14 px gap and the review summary a 16 px stack. Give the four links the
    `.text-link` look, with its hover look. The machine names keep their
    look, so these links need a variant of the part. Remove the hover
    underline of the `DetailHeader` breadcrumbs. Check each changed screen
    against its mockup at both widths. Effort: Medium.
  - **C.** Change the mockups to the dashboard. Give each mockup dialog the
    body gap of its dashboard dialog, the review summary a 14 px stack, the
    four links an underline only on hover, and the `DetailHeader`
    breadcrumbs a hover underline. Each needs a new class, because
    `.dialog-body`, `.stack`, `.text-link`, and `.crumb` serve many other
    mockups. Effort: Medium.
  - **D.** Take the gaps from the mockups and the link look from the
    dashboard. Give every dashboard dialog body a 14 px gap and the review
    summary a 16 px stack. Give the mockups of the four links and of the
    `DetailHeader` breadcrumbs the dashboard look, with new classes.
    Effort: Medium.
  - **E.** Give each value one home, with the values that B, C, or D
    chooses. The one dialog frame that option C of
    `plans/follow-ups/dashboard-duplicate-controls.md` builds sets the body
    gap of every dialog, as `.dialog-body` does in the mockups. The link
    look moves into `@ciproxy/tokens`, as option F of
    `plans/follow-ups/terms-lines-mockup-look.md` says. Effort: Medium.
- **Left out:**
  - One body gap for every mockup dialog, taken from the dashboard: the
    dashboard dialogs use four gaps, so no one value matches them.
  - One component that the mockups and the dashboard share: the mockups
    style with CSS classes, and the dashboard with style objects, so one
    component cannot serve both.
- **Wider fix:** E. The body gap and the link look then each have one
  home, so a new dialog or a new link cannot drift again.
- **Previous examples:**
  - `plans/follow-ups/terms-lines-mockup-look.md` records the same class
    for the two Terms lines and the terms notice.
  - `plans/follow-ups/savings-chart-firna-marks.md` records the same class
    for the savings chart. Its legend gap is 14 px, where the mockup uses
    16 px.
- **Related:**
  - `plans/follow-ups/terms-lines-mockup-look.md`, the same class and some
    of the same code. Both items name the 12 px body gap of the grant
    dialog, so their options can conflict there. The user can answer both
    items in one message.
  - `plans/follow-ups/dashboard-duplicate-controls.md`, the same code. Its
    option C gives the dashboard one dialog frame and one `TextLink`, which
    option E of this item needs.
- **Recommendation:** B. The mockups are the spec, and B brings the six
  dialogs, the summary, and the links in line with them. E best protects
  the codebase, because each value then has one home.
- **Action:** ask the user, because the fix decides which side of a mockup
  and product difference is right, and no record says why the dashboard
  values differ.
