# Dashboard keeps duplicate base controls

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 605. Round 2 of the
  same review added option E on 18 November 2026, from finding 1202.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `ts/app/src/components/feature-parts.tsx:31`,
  `ts/app/src/components/panel.tsx:16`,
  `ts/app/src/components/detail-list.tsx:26`,
  `ts/app/src/components/status-label.tsx:26`,
  `ts/app/src/components/text-link.tsx:64`,
  `ts/app/src/components/link-tabs.tsx:84`,
  `ts/app/src/screens/settings/settings-tabs.tsx:98`,
  `ts/app/src/components/shell/nav-link.tsx:166`, and
  `ts/app/src/screens/jobs/history-view.tsx:44`.
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
  - **History mode of a link:** how an in-app link of the router changes
    the browser history. With `push`, the link adds an entry. With no mode,
    the link gets the router default. That default replaces the current
    entry when the link opens the route that shows, with another query.
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
      `TextLink`, `SettingsTabs`, and the standalone `LinkTabs` push a
      history entry. The standalone `TextLink` replaces it.
    - The size of labels and links: 13 px in the feature parts, 14 px in
      the standalone components.
    - The status word: ink, wrapping, with a 6 px gap, in the feature
      parts. Ink2, on one line, with an 8 px gap, in the standalone
      component.
  - The standalone set already matches the mockups in three points: 14 px
    panel padding on a phone, the status word, and 14 px detail rows.
  - The history mode of the in-app links:
    - 29 files use the router `Link` directly, with 34 links. 9 of them
      pass `push`. Each other link gets the router default.
    - The Jobs tabs Live and History share `/jobs` and differ only in the
      query `tab=history`. `LinkTabs` pushes, so Back from History opens
      Live. The journey `history-filters-detail-and-back` of the page smoke
      in `ts/app/scripts/browser-pages-data-journeys.ts` checks it.
    - The sidebar items, `NavLink` at
      `ts/app/src/components/shell/nav-link.tsx:166`, use the router
      default. A click on the item of the current page from a sub-view, such
      as Machines from one machine page, or Jobs from History, replaces the
      entry of that sub-view. Back then skips the sub-view. No doc states
      what Back must do in this case.
    - A click on the tab that already shows pushes a second entry for the
      same address, on the Settings tabs and on the Jobs tabs. The next
      Back then shows the same tab.
    - "Clear filters" in History, at
      `ts/app/src/screens/jobs/history-view.tsx:44`, must keep replacing
      the entry. A History filter change replaces the address, as
      `ts/app/README.md` says, and as `history-filters.tsx:55` does.
    - A push to the same route remounts the page, as the Settings tabs and
      the machine links do now.
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
  navigation, status text, and Back behaviour. Each in-app link also
  chooses its own history mode, so Back works differently from link to
  link.
- **Impact if no action:** A fix of a shared control reaches only some
  pages. Each later change must handle both sets. Back skips a sub-view
  after a click on the sidebar item of its page, and shows the same tab
  again after a click on the tab that shows.
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
      sideways scroll. Keep the Back case of the Jobs tabs in the page
      smoke.
    - Decide each conflicting default once, against the mockups. Make the
      standalone `TextLink` push, as `LinkTabs` and the feature parts
      `TextLink` do.
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
  - **E.** Do C, and wrap the router `Link` once, in one app link
    component that sets the history mode of every in-app link:
    - It pushes by default.
    - It adds no entry when the target is the address that already shows.
    - It replaces the entry only where a page asks for that, such as
      "Clear filters" in History.
    - Move `LinkTabs`, `SettingsTabs`, both `TextLink` components, and the
      other in-app links to it. Remove their own `push` props.
    - `NavLink` moves only when the user accepts one change: a click on
      the sidebar item of the current page from a sub-view then adds an
      entry. Until then, `NavLink` keeps the router `Link`, with a comment
      that says why.
    - Extend the journey `history-filters-detail-and-back` of the page
      smoke: a click on the tab that shows adds no entry, and "Clear
      filters" still replaces the address.
    - State the rule in `ts/app/README.md`.
    - Cost: 34 links in 29 files. A push to the same route remounts the
      page, as the Settings tabs and the machine links do now.
    - Effort: Medium.
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
  - A screen `getId` or `dangerouslySingular` keyed on the tab: it fails
    the goal. When a person returns to a view that they visited, the router
    moves the old route to the top and replaces the entry. Back then no
    longer returns to the other view.
  - A full page load for each tab click: `push` keeps Back without a
    reload of the dashboard and all its data.
  - A patch to Expo Router that makes a navigation of the query alone
    push: E uses the supported `push`, and no Expo upgrade must carry a
    patch.
  - A lint or a test for the class of the history mode: E is a structural
    fix, and a new lint needs the user.
- **Wider fix:** D. The reuse rule of the `product-ui` skill is in force,
  but it could not apply, because no branch saw the files of the others.
  C is the fix inside this repository. The skill part of D is in force
  since 20 November 2026: the `delegating` skill builds a shared piece
  before the parallel work items that need it. For the
  history mode of links, E is the wider fix: one component sets it for
  every in-app link, also for links that later pages add.
- **Previous examples:**
  - The same review found a second case. Billing and metering each built
    the monthly price calculation on parallel branches, `4304e3f` and
    `a0a57e9`, both from `4bfa4e5`, on 4 November 2026.
  - The Jobs tabs replaced their entry until 18 November 2026, while the
    Settings tabs pushed. So Back from History skipped Live.
- **Related:**
  - The skill part of D merged on 20 November 2026 in `futex-ai/skills`.
    This item keeps the fix inside this repository.
  - `plans/follow-ups/machines-boat-start-status.md`, the same code. Its
    fix changes `ts/app/src/screens/machines/overflow-panel.tsx`, which C
    moves to the standalone `Panel` and `TextLink`. Only the order of the
    work changes. The options do not conflict.
- **Recommendation:** E, changed from C. C settles the duplicated controls,
  but each in-app link still chooses its own history mode. E gives every
  in-app link one default, so the sidebar items and the tabs keep Back
  too. D best protects the codebase from new copies, but its skill part
  needs the user.
- **Action:** fix after merge, apply E, because the severity is Low, and
  the change reaches 29 files and many screens that need browser checks.
