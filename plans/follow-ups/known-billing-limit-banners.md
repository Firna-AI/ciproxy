# Known billing scenes omit the soft-limit banner

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, a problem outside the review scope that the check of
  owner finding 4 found.
- **Source:** Review.
- **Category:** Mockup.
- **Location:** `docs/mockups/src/screens/billing/plans.tsx:141`,
  `docs/mockups/src/screens/billing/checkout-review.tsx:139`, and
  `docs/mockups/src/screens/billing/checkout-result.tsx:49`.
- **Reason:** These three known-data screens had no banner before the branch
  that found the gap, so the gap is outside its review scope.

- **Terms:**
  - **Known-data scene:** a mockup scene of the fixture story, which shows
    each page with every fact. Its banner names only the facts that the API
    gives.
  - **Current-contract scene:** a mockup scene that shows a moment with only
    the facts that the API gives, under a `current-contract` folder.
  - **Moment:** one day of the fixture story with its use, a `UsageScene` of
    `docs/mockups/src/fixtures/usage.ts`, such as `free-over` for
    4 September.
  - **`apiBanner`:** the one renderer of the soft-limit, failed-payment, and
    ended-plan banners of the mockups, at
    `docs/mockups/src/components/api-banner.tsx:49`.
- **Background:**
  - `docs/protocol/metering.md:93-94`: while a workspace is above a soft
    limit, every page of the workspace shows one banner.
    `docs/protocol/billing.md:211-213`: the banner of a failed payment or of
    an ended tier takes its place, so a page shows one banner.
  - Two moments of these screens pass a soft limit on Free: `free-over`, on
    4 September, and `inactive-organization`, on 8 December. Both are in
    `passedLimit` at `docs/mockups/src/fixtures/current-api.ts:41-47`.
  - These known-data states show no banner:
    - Plans, in the states `from-free` and `inactive-organization`.
      `plans.tsx:141` shows the banner only in the pending states.
    - The Checkout review `to-team`, at the moment `free-over`.
      `checkout-review.tsx:139-141` shows it only with `api`.
    - The Checkout return states `confirming`, `still-confirming`, and
      `cancelled`, before Stripe confirms Team on 4 September.
      `checkout-result.tsx:49-52` shows it only in the current-API states.
      The state `confirmed` is on Team with no passed limit, so it rightly
      shows no banner.
  - The entries of these states are
    `docs/mockups/src/entries/app/billing/plans/index.mockup.tsx`,
    `docs/mockups/src/entries/app/billing/checkout/review.mockup.tsx`, and
    `docs/mockups/src/entries/app/billing/checkout-return/states.mockup.tsx`.
  - The flow at
    `docs/mockups/src/entries/user-flows/free-upgrades-to-team.mockup.tsx`
    opens Plans, the review, and the return. At `:14-16`, it says that a
    banner on every page says that jobs keep running and offers Team.
  - The Plan and usage pages of the same moments show the banner through
    `sceneBanner`, at
    `docs/mockups/src/screens/billing/billing-scenes.tsx:33-42`. The
    current-contract states of the three screens show it through
    `apiBanner`, in the lines that the Location names.
  - `apiBanner` words each banner as the API does, also on a known moment.
    It takes the link of the page: `plans` on a billing page, `usage` on
    Plans itself, and `other` on every other page. It takes `canPay`, and a
    person who cannot pay reads "Ask an Admin or a Billing person" in place
    of its action. `momentPages` and `usageBanner`, at
    `docs/mockups/src/components/usage-banner.tsx:58-82`, give the pages of
    each moment.
  - Plans takes a `role`. The review and the return take no role, and they
    show a payer.
  - On 2026-11-08 the user kept the product words of these banners, and the
    known-data banners changed to those words. That answer settled the
    words, not which known-data screens show a banner.
- **Problem:** The known-data Plans, Checkout review, and Checkout return
  states at a soft-limit moment show no banner, while the protocol asks for
  one on every page.
- **Impact if no action:** The known-data spec shows these billing pages
  without the banner that the product and their current-contract states
  show, so a reader can take the missing banner as the target.
- **Root question:** How do the known-data billing scenes show the one
  banner of each moment that passes a soft limit?
- **Options:**
  - **A.** Give each of these states the banner of its moment through
    `apiBanner`. On Plans, link `usage` to the Plan and usage of the moment,
    and give `canPay` from the role. On the review, link `plans` to its back
    link. On the return states before the confirmation, link `plans` to the
    Plans of 4 September. Keep the accepted words, add no fact, and keep
    `confirmed` without a banner. Update each entry or flow text that
    describes these pages. Run the mockup check, and look at the changed
    scenes at each width. Effort: Small.
  - **B.** Make the moment a required property of the signed-in shell,
    `docs/mockups/src/components/shell/app-shell.tsx`. Each screen then
    names its moment, or none, and the shell draws its banner, so that no
    screen leaves it out by accident. The Organizations tab keeps its own
    banner. Cost: the 14 screen files that use the shell change. B also
    adds a new build error for a screen that names no moment, so it needs
    the user. Effort: Medium.
- **Left out:**
  - No change, because it keeps pages without the banner that
    `metering.md:93-94` requires.
  - Removal of the banner from the current-contract states, because it
    breaks the same requirement.
  - A story in which these screens pass no limit, because the Free to Team
    flow starts from the passed limit of 4 September.
- **Wider fix:** B, a detection fix. It is blocked, because a new build
  error needs the user.
- **Previous examples:** None found. The other known-data screens with a
  moment already show its banner, such as Plan and usage, Jobs Live, the
  Machines pages, and the Labels tab.
- **Recommendation:** A. It shows the one banner on each page with the
  renderer that already exists. B best protects the catalogue, but it needs
  the user.
- **Action:** fix after merge, apply A, because the gap is older than the
  branch that found it and outside its review scope.
