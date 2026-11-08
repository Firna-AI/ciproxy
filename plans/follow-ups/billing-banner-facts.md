# Banners lack the plan and the passed limit

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the billing pages.
- **Source:** Implementation.
- **Category:** Mockup.
- **Location:** `crates/ciproxy-api/src/wire/pages.rs:36`,
  `crates/ciproxy-billing/src/service/transition.rs:71`,
  `docs/mockups/src/components/usage-banner.tsx:151`,
  `docs/mockups/src/components/usage-banner.tsx:172`,
  `docs/mockups/src/components/usage-banner.tsx:203`, and
  `ts/app/src/view/banner.ts:82`.
- **Reason:** The choice decides which side of a mockup and product
  mismatch is right, and that choice belongs to the user.

- **Terms:**
  - **Banner:** the one notice that every workspace page shows, which
    `GET /api/workspaces/{id}/banner` chooses.
  - **Known-data banner:** a banner of the fixture story in
    `docs/mockups/src/components/usage-banner.tsx`, with every fact.
  - **Plan and usage:** the billing page of a workspace,
    `GET /api/workspaces/{id}/billing`, with the plan, the use, and the
    current period.
- **Background:**
  - `docs/protocol/metering.md:93-95` says that while a workspace is above
    a soft limit, every page shows one banner, which says that jobs keep
    running and gives the offer. `docs/protocol/billing.md:199-210` says
    that the banner of a failed payment gives its date and the date of the
    next try, and that a later banner shows that the tier ended.
  - `PageBanner` at `wire/pages.rs:36` has four kinds:
    - `payment_failed`: the failed date, the next retry date, and the
      action.
    - `subscription_ended`: `until`, the exclusive end of this notice, and
      the action. `until` is the end time of the paid plan plus one
      calendar month, at
      `crates/ciproxy-billing/src/service/transition.rs:76-85`.
    - `inactive_organizations`: each inactive organization and its holds.
    - `soft_limit`: the offer that covers the use, and the action. The
      offer is the next choice, not the current plan or its limit.
  - When a paid plan ends, the billing service sets `free_started_at` to
    the end time, at `transition.rs:71`, and sets the tier to Free, at
    `:87`. It keeps no earlier tier. While the ended banner shows, that
    time is the start of the current Free period, as
    `docs/protocol/metering.md:61-62` says. Plan and usage gives it as
    `period_start`, at `crates/ciproxy-api/src/handlers/page_context.rs:174`.
  - The dashboard reads the facts of the banner only from `GET /banner`,
    in `ts/app/src/state/billing-banner.ts`.
  - The known-data banners name more:
    - The passed limit and the current plan: "mokly-ai passed the 25,000
      starts of Team" at `:151`, "4 own machines on Team, which includes 3"
      at `:172`, "mokly-ai used the 2,000 starts of Free" at `:132`, and
      "2 organizations on Free, which includes 1" at `:108`.
    - The current plan: "Team stays active while Stripe tries again" at
      `:195`.
    - The ended plan and the day it ended: "Team ended on 18 December
      after failed payments" at `:203`.
    - Other facts: the machine count at `:111` and `:211-212`, the
      organizations that run jobs at `:111`, the starts and the machines of
      Free at `:210-211`, and the add-on actions at `:157-159` and
      `:174-176`. No known-data scene shows two passed limits at once.
  - The over-limit banners of Team also give add-on prices, such as the
    price of a block of starts. The API gives no unit prices, and
    `plans/follow-ups/add-on-change-review.md` keeps them out of the
    dashboard.
  - The product words each banner with the facts of the API, in
    `ts/app/src/view/banner.ts:82-114`: "`<workspace>` passed a limit of its
    plan", "Your plan stays active while Stripe tries again", and "Your
    paid plan ended after failed payments". The current-contract scenes
    under `docs/mockups/src/entries/app/billing/current-contract/` show
    this copy. Each sentence is true.
  - On 8 November 2026, the user kept the product for the plan facts of
    two workspace lists, the workspace picker and the workspace switch,
    and changed their mockups to match the app. That answer does not
    cover the banners. If the user applies the same reason here, A
    follows.
- **Problem:** The known-data banners and the product disagree on the plan,
  the passed limit, and the day a plan ended.
- **Impact if no action:** The banners stay true but less precise, and the
  mockups keep two versions of them.
- **Root question:** What must each banner name about its cause?
- **Options:**
  - **A.** Accept the current-contract wording as the spec, and change the
    known-data banners to it. Effort: Small.
  - **B.** Add typed facts to the banner response, and word each banner
    from them. `soft_limit` gets the current plan, and each passed limit
    with its kind, its use, and its included amount. `subscription_ended`
    gets the ended plan and its end day. `payment_failed` gets the current
    plan. No add-on prices. The end day comes from `free_started_at`. The
    name of the ended plan needs a new field in the stored billing state,
    because the service keeps no earlier tier. The same change updates
    `docs/protocol/metering.md:93-95` and `docs/protocol/billing.md:199-210`,
    which say what each banner gives. It adds a known-data scene with two
    passed limits, and it changes the known-data banners for the other
    facts that B does not add, as A does. Effort: Medium.
  - **C.** Read Plan and usage beside the banner while a soft-limit,
    failed-payment, or ended-plan banner shows. Word the plan, each passed
    limit, and the use from it, and the end day from `period_start`. No
    API change. The ended plan stays unnamed, as in "Your paid plan ended
    on 18 December". This drops the shared assumption that a banner takes
    its facts only from the banner response: that is the current code of
    `billing-banner.ts`, and the protocol asks for one banner on every
    page, not for one read. Cost: one more read on each page change while
    such a banner shows, and the two reads can disagree for a moment after
    a plan change. Effort: Medium.
- **Left out:** A day built from `until`, because `until` is the end time
  plus one calendar month. A month end clamps, so one month back from
  `until` can give a wrong day. It fails the goal.
- **Wider fix:** None.
- **Previous examples:** An older follow-up item on the plan facts of the
  workspace lists said: "A note of the task that built the app says that a
  shell banner also omits the tier and the count. Nobody checked that
  note." The answer of 8 November 2026 deleted that item. This item checks
  the note: `PageBanner` carries no plan. Source:
  [the old item, lines 65 to 67](https://github.com/Firna-AI/ciproxy/blob/a942f9861b6a1a59b7732eafc08aeb95e77ac7aa/plans/follow-ups/workspace-list-plan-facts.md#L65-L67).
- **Related:** `plans/follow-ups/jobs-notice-facts.md` asks the same
  question for the Jobs notices. The two answers do not depend on each
  other.
- **Recommendation:** B. Typed facts in the one banner response keep every
  page on one consistent read, which C cannot do.
- **Action:** ask the user, because the choice decides which side of a
  mockup and product mismatch is right.
