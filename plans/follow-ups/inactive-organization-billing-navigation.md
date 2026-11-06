# Billing links lose the current mockup state

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from PR #32, owner review round 1, finding 4.
- **Source:** Review.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/components/inactive-banner.tsx:80`,
  `docs/mockups/src/screens/billing/plans.tsx:69`,
  `docs/mockups/src/components/usage-banner.tsx:106`,
  `docs/mockups/src/fixtures/org-links.ts:117`, and
  `docs/mockups/src/entries/app/billing/change-plan/states.mockup.tsx:48`.
- **Reason:** This mockup defect does not affect installation execution.

- **Terms:**
  - **Fixture:** sample data under `docs/mockups/src/fixtures/` that
    defines the situation shown by a mockup.
  - **Catalogue path:** a path from `docs/mockups/src/components/paths.ts`
    that names one catalogue screen, which every mockup link uses.
  - **Plan and usage:** the billing page with the tier, the starts and own
    machines of the period, and the add-ons.
  - **Plans:** the billing page that compares the three tiers and chooses
    one.
  - **Moment:** one day of the fixture story, with its tier, its
    organizations, and its use.
- **Background:**
  - The fixture data tells one story, as `docs/mockups/README.md` requires.
  - On 4 December, mokly-ai is on Free and keeps two organizations,
    mokly-ai and mokly-labs. The entry `freeOrganizations` in
    `change-plan/states.mockup.tsx` shows Plan and usage at that moment, at
    the catalogue path `paths.freeOrganizations`.
  - On 8 December, a third organization, mokly-studio, joins inactive,
    because Free includes one organization. `org-links.ts` holds this
    what-if, and `decemberOrganizationCount` counts three organizations.
  - The December screens of the inactive organization show these three
    organizations, including inactive mokly-studio. Their Plan and usage
    links should preserve that situation.
  - The inactive banner of `inactive-banner.tsx` offers "Plan and usage" and
    "Choose Team" for the plan hold. "Choose Team" opens
    `paths.plansInactive`, the December Plans state.
  - The December Plans state in `plans.tsx` says "Your workspace now: 3
    organizations and 3 own machines". Its breadcrumb "Plan and usage"
    opens Plan and usage.
  - The banner of the `free-organizations` scene, at `usage-banner.tsx:106`,
    counts the two organizations of `org.ts`.
  - `usageNotice` in `docs/mockups/src/screens/billing/usage-notices.tsx`
    shows the Free pace notice only for the scenes `free` and
    `free-organizations`, by name.
- **Problem:** The "Plan and usage" link of the inactive banner and the
  breadcrumb of the December Plans state both open
  `paths.freeOrganizations`. That destination shows the earlier
  two-organization state from 4 December.
- **Impact if no action:** Following the link removes the third
  organization from the displayed count. Its next plan action also opens
  the earlier September state.
- **Root question:** How should billing navigation preserve the current
  organization state?
- **Options:**
  - **A.** Add the matching December Plan and usage state. Point both links
    to it. It also needs a catalogue entry and the visual check at three
    widths. Effort: Small.
  - **B.** Do A and share one fixture for the December organization data and
    billing destinations. Effort: Medium.
  - **C.** Keep both actions visible with no target, as the December tier
    buttons do. `ChooseTeam` then takes an optional Plan and usage target.
    No mockup then shows the December Plan and usage page, and a reviewer
    cannot follow the action. Effort: Small.
  - **D.** Give every billing and organization state one moment record,
    with the target of each billing page, and let every link read it. It
    changes screens beyond the December screens, such as the 4 December
    links, so it needs the user's approval. Effort: Large.
- **Left out:**
  - Removing the links removes useful actions. C keeps the actions.
  - Keeping the earlier destination preserves the wrong state.
  - Changing the 4 December state to 8 December breaks the flow
    `free-keeps-organizations`, which shows 4 December with two
    organizations.
  - A policy check that compares the moment of a link and of its target adds
    a new check, which needs the user. B and D remove the wrong case.
- **Wider fix:** B. It prevents connected screens from selecting their data
  and destinations independently, within the December screens. D would
  cover every state, but it needs the user's approval, and this item does
  not include it.
- **Previous examples:**
  - `plans/follow-ups/overflow-flow-report-dates.md` records an earlier
    mismatch between a flow and its fixture.
  - The 4 December Plan and usage page itself opens the September Plans
    state, with one own machine and a pace to 30 September. Its banner, pace
    notice, and Plan panel link to `paths.plans`, at `usage-banner.tsx:108`,
    `usage-notices.tsx:48`, and `plan-usage.tsx:105`.
  - That navigation predates the change that added this item. B leaves it
    unchanged.
- **Recommendation:** B. Shared fixture data best protects consistency
  across connected screens.
- **Action:** fix after merge, apply B, because this mockup defect does not
  affect installation execution.
  - The new December Plan and usage state shows 8 December with the three
    organizations. Its banner counts them.
  - Its banner, pace notice, and Plan panel open `paths.plansInactive`, not
    the September `paths.plans`. A new usage scene gets the pace notice only
    when `usageNotice` admits it.
  - Keep the data in `docs/mockups/src/fixtures/`, and join the targets in
    a scene module, as
    `docs/mockups/src/screens/settings/organization-scenes.tsx` does.
  - Keep the 4 December and September states as they are.
  - Use the `mockups` and `product-ui` skills. Run
    `npm --prefix ts run mockups:check`.
  - Inspect each changed screen at desktop widths of 1440 and 720 pixels,
    and at a mobile width of 390 pixels. Follow the changed links in the
    flows `free-adds-organization` and `admin-turns-on-organization`.
  - Add no test that reads mockup text.
