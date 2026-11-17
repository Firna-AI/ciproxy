# Dashboard keeps duplicate base controls

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 605, with the Back
  defect of the Jobs tabs that its check found.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `ts/app/src/components/feature-parts.tsx:31`,
  `ts/app/src/components/panel.tsx:16`,
  `ts/app/src/components/detail-list.tsx:26`,
  `ts/app/src/components/status-label.tsx:26`,
  `ts/app/src/components/text-link.tsx:64`,
  `ts/app/src/components/link-tabs.tsx:81`, and
  `ts/app/src/screens/settings/settings-tabs.tsx:98`.
- **Reason:** The cleanup changes many screens and needs browser checks
  against the mockups, and no requirement fails while both sets stay.

- **Terms:**
  - **Feature parts:** the controls in
    `ts/app/src/components/feature-parts.tsx`: `Panel`, `DetailRow`,
    `StatusLabel`, `TextLink`, `FeatureHeader`, and `FeatureDialog`.
  - **Standalone components:** one module for each control under
    `ts/app/src/components/`, such as `panel.tsx`, `detail-list.tsx`,
    `status-label.tsx`, `text-link.tsx`, `detail-header.tsx`,
    `page-state.tsx`, and `link-tabs.tsx`.
  - **`@firna/ui`:** the shared component library. The app checks it first,
    before it builds a control.
  - **Mokly:** the tool that renders the approved mockups under
    `docs/mockups`, with react-dom and CSS classes.
- **Background:**
  - The dashboard is the Expo app in `ts/app`. It renders React Native Web.
  - Two sets of base controls are live. 19 screen files of Machines and
    Settings import the feature parts. The Jobs, Savings, and billing pages,
    and Settings under Overflow, use the standalone components.
  - Three parallel branches built the two sets on 8 November 2026. No
    branch saw the files of the others:
    - `58b12be`, "feat(app): add Machines and Settings screens", added
      `feature-parts.tsx` and `settings-tabs.tsx`. It started from
      `a942f98`.
    - `69b4f1c`, "feat(app): add Boat setup and overflow settings", added
      the standalone `panel.tsx`. It also started from `a942f98`.
    - `166b2c9`, "feat(app): add jobs, savings and billing dashboard",
      added `detail-list.tsx`, `status-label.tsx`, `text-link.tsx`,
      `link-tabs.tsx`, and `detail-header.tsx`. It started from `52704fb`.
  - The copies differ in their capabilities:
    - The standalone `Panel` is a region that its title names, with
      `aria-labelledby`, and it can move the focus to its title. The
      feature parts `Panel` cannot.
    - The standalone `Panel` has 14 px padding on a phone. The feature
      parts `Panel` keeps 16 px.
    - `FeatureHeader` repeats `PageHeader` at
      `ts/app/src/components/page-state.tsx:18` and `DetailHeader` at
      `ts/app/src/components/detail-header.tsx:16`. It has no breadcrumb
      landmark and no chevron.
    - `ts/app/src/screens/overflow/disconnect-dialog.tsx:32-35` and
      `ts/app/src/screens/billing/move-to-free-dialog.tsx:60` repeat the
      phone placement of `FeatureDialog`.
    - `SettingsTabs` repeats the tab anchor of `LinkTabs`, with a count
      and a sideways scroll on a phone.
  - The copies also differ in their defaults:
    - Back on links and tabs that stay on the same route: the feature parts
      `TextLink` and `SettingsTabs` push a history entry. The standalone
      `TextLink` and `LinkTabs` replace it.
    - The size of labels and links: 13 px in the feature parts, 14 px in
      the standalone components.
    - The status word: ink, wrapping, with a 6 px gap, in the feature
      parts. Ink2, on one line, with an 8 px gap, in the standalone
      component.
  - The standalone set already matches the mockups in three points: 14 px
    panel padding on a phone, the status word, and 14 px detail rows.
  - The Back defect of the Jobs tabs:
    - `LinkTabs` renders each tab as `<Link href={tab.href} asChild>`,
      with no `push`, at `ts/app/src/components/link-tabs.tsx:81`.
    - The Jobs tabs Live and History open the same route and differ only in
      the query `tab=history`, from `jobsHref` at
      `ts/app/src/view/jobs-links.ts:42-55`.
    - So expo-router replaces the history entry, and Back from History
      skips Live.
    - The header comment of `link-tabs.tsx` says that the tabs work with
      the back button. The Settings tabs push for this reason, at
      `settings-tabs.tsx:96-98`.
    - The finding check found this through a read of the expo-router code.
      Nobody ran it.
  - `feature-parts.tsx` has 300 lines. The gate fails on a source file
    over 300 lines.
  - `@firna/ui` has no panel, no text link, and no label-value row. Both
    sets take `StatusDot` from it.
  - The Overflow panel of Machines, in
    `ts/app/src/screens/machines/overflow-panel.tsx`, uses the feature parts
    `Panel` and `TextLink`. It shows a refresh failure beside its kept
    figures.
- **Problem:** Two sets of base controls implement the same controls apart.
  They already differ in mobile spacing, named panel regions, link
  navigation, status text, and Back behaviour.
- **Impact if no action:** A fix of a shared control reaches only some
  pages. Each later change must handle both sets. The Jobs tabs keep the
  Back defect.
- **Root question:** How should all dashboard pages share their base
  controls?
- **Options:**
  - **A.** Keep the feature parts as the base. Move the standalone
    consumers and the capabilities that they need to the feature parts.
    Delete the duplicate definitions. Effort: Medium.
  - **B.** Use the standalone components everywhere. Add the missing
    capabilities there, move every caller, and delete the duplicate
    definitions and styles. Run the existing browser checks.
    Effort: Medium.
  - **C.** Move every duplicated base control to the standalone modules,
    with one app module for each base component of the mockups:
    - `Panel`, `DetailList` with `DetailRow`, `StatusLabel`, and
      `TextLink`.
    - The page header: `FeatureHeader` goes into `PageHeader` and
      `DetailHeader`, which get `actions` and a monospace title.
    - One dialog frame: `FeatureDialog` moves into its own module and
      serves every dialog, also the two dialogs that repeat its placement.
    - `SettingsTabs` is built on `LinkTabs`, with the count and the
      sideways scroll.
    - Decide each conflicting default once, against the mockups. Make
      `TextLink` and the link tabs push.
    - Add a Back case for the Jobs tabs to the browser checks.
    - Keep the refresh failure of the Overflow panel.
    - Update `ts/app/README.md`.
    - Cost: more call sites than B, with three headers, two dialogs, and
      the Settings tabs. Machines and Settings change visibly, so check
      them against the mockups.
    - Effort: Medium.
  - **D.** Do C, and add one line to the independence test of the shared
    `delegating` skill. The line says that work items that each need the
    same new base control or calculation do not start in parallel until
    that piece is on the primary branch. The skill lives in
    `futex-ai/skills`, outside this repository, so this part needs the
    user. Effort: Medium.
- **Left out:**
  - Compatibility wrappers: they have the same effort grade as B. They
    keep a second API where the adapted props `muted`, `first`, `mono`, and
    push can drift again.
  - A third component package: it adds migration work without a gain over
    B. One package shared with the mockups cannot work, because Mokly
    renders react-dom with CSS classes, and the app renders React Native
    Web.
  - `@firna/ui` as the base: it has no panel, no text link, and no
    label-value row.
  - A fix of only the drifted details in both copies: two implementations
    stay.
  - An import rule for the wrapped primitives: it fails the goal for the
    class. `Panel`, `DetailRow`, and the header wrap only `View` and
    `Text`, and 29 files import the router `Link` directly. So the rule
    could guard only the status dot and the dialog frame. It is also a new
    lint, which needs the user.
- **Wider fix:** D. The reuse rule of the `product-ui` skill is in force,
  but it could not apply, because no branch saw the files of the others.
  C is the fix inside this repository. The skill part of D waits for the
  user in `plans/follow-ups/parallel-shared-code-duplication.md`.
- **Previous examples:** The same review found a second case. Billing and
  metering each built the monthly price calculation on parallel branches,
  `4304e3f` and `a0a57e9`, both from `4bfa4e5`, on 4 November 2026.
- **Related:**
  - `plans/follow-ups/parallel-shared-code-duplication.md`, a shared wider
    fix. It holds the skill part of D. Neither item waits for the other.
  - `plans/follow-ups/machines-boat-start-status.md`, the same code. Its
    fix changes `ts/app/src/screens/machines/overflow-panel.tsx`, which C
    moves to the standalone `Panel` and `TextLink`. Only the order of the
    work changes. The options do not conflict.
- **Recommendation:** C, changed from B. B as written leaves the header,
  the dialog frame, and the tabs duplicated, and it treats conflicting
  defaults as additions. C settles them for a few more call sites. D best
  protects the codebase, but its skill part needs the user.
- **Action:** fix after merge, apply C, because the severity is Low, and
  the change reaches many screens that need browser checks.
