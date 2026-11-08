# Portal buttons drop their progress and failure

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  pull request #53, review finding 6, with a problem that its finding check
  found in the Keep plan notice.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `ts/app/src/screens/billing/checkout-screen.tsx:78`, in
  `Review`, `ts/app/src/screens/billing/payment-result.tsx:52`, in
  `ResultAction`, `ts/app/src/screens/billing/add-ons-panel.tsx:134`,
  `ts/app/src/screens/billing/usage-notices.tsx:76`,
  `ts/app/src/screens/billing/usage-panels.tsx:60`, and
  `ts/app/src/components/billing/workspace-banner.tsx:106`.
- **Reason:** Update card shows only for a payment outcome without a Stripe
  page, which the server sends in unusual states, and the portal request
  must then fail too. The fix costs the same later.

- **Terms:**
  - **Billing portal:** the Stripe page for the invoices, the saved card,
    and Keep plan.
  - **Portal action:** `createPortalAction` in
    `ts/app/src/state/billing-flows.ts:125`, a store that asks the API for a
    new portal session, records its progress and its failure, and leaves
    for the Stripe page.
- **Background:**
  - A failed charge of a paid change gives the result "Stripe could not
    charge your card" with Update card and Try again, when the outcome has
    no Stripe page. `ts/app/src/view/billing-outcome.ts:140` builds that
    action.
  - The server gives a payment outcome without a Stripe page only in
    unusual states, for example an upgrade during a past-due renewal, at
    `crates/ciproxy-billing/tests/_tests_/expiry_recovery_tests.rs:143`, or
    a change whose update invoice is void, at
    `crates/ciproxy-billing/src/service/completion.rs:49`.
  - Each press asks the API for a new portal session, so a retry is safe.
    The server only creates the session, at
    `crates/ciproxy-billing/src/service/runtime.rs:96`.
  - The portal buttons today:
    - Checkout: `Review` creates the portal store, at
      `checkout-screen.tsx:78`, and calls `portal.run()`, but it never
      reads `portal.state`.
    - `PaymentResult` gets only `onPortal`. Its Update card button has no
      busy state, at `payment-result.tsx:52-61`, and no failure output.
    - The add-ons panel of Plan and usage uses the same `PaymentResult`,
      at `add-ons-panel.tsx:134`.
    - The Keep plan button of the usage notices has no busy state, at
      `usage-notices.tsx:76`. Only the caption of the Plan panel shows its
      failure.
    - The Plan panel and the workspace banner already show progress and
      failure: "Unable to open the Stripe billing portal. Try again.", at
      `usage-panels.tsx:60-70`, and the banner at
      `workspace-banner.tsx:106-115`.
- **Problem:** Each surface wires the portal action by itself. Checkout and
  the add-ons panel show no progress and no failure for Update card, and
  Keep plan shows no progress. So a failed portal request looks as if the
  button did nothing.
- **Impact if no action:** A customer with a failed card cannot tell why
  Update card did nothing, and repeated presses seem to have no effect.
- **Root question:** How does a customer who presses a portal button see
  that the billing portal opens, or that it failed, and recover?
- **Options:**
  - **A.** Read `portal.state` in Checkout. Show a retryable error, and
    pass the portal progress to Update card. Its regression needs a page
    smoke journey with a harness scenario without a Stripe page. Effort:
    Medium.
  - **B.** Make one portal control own its request, its progress, and its
    failure. Use it in Checkout, the add-ons panel, the Plan panel, the
    Keep plan notice, and the workspace banner. Keep their labels and
    their navigation. Keep the effective payer permission outside the
    control: each surface decides whether the control shows. These
    surfaces need mockup states for progress and failure. Effort: Medium.
  - **C.** Add a shared feedback component for every billing action
    store, also the cancellation actions. It reaches more workflows than
    the portal needs, and it must keep the uncertain words of a failed
    Move to Free. Effort: Medium.
  - **D.** Make the portal state a required input of the payment outcome
    model. The model marks Update card busy while the request runs. After
    a failure, it adds the existing text "Unable to open the Stripe billing
    portal. Try again." Checkout and the add-ons panel pass their portal
    state. Add the failed-portal case to the existing billing views test.
    Cost: on Plan and usage, a portal failure then also shows in the caption
    of the Plan panel. Effort: Small.
  - **E.** Keep one portal store for each workspace, cached like the banner
    store. Every portal button shows its busy state, and the workspace
    banner shows the failure on every workspace page. Clear the failure on
    a page change, and remove the caption of the Plan panel. Cost: on a
    phone, the failure can show far from the pressed button and out of
    view. Its regression needs a new page smoke scenario. Effort: Medium.
- **Left out:**
  - Change nothing, because a failed recovery action stays silent.
  - A redirect of Update card to another dashboard page, because D is
    better on every point. D keeps the direct portal action at the same
    effort, and a redirect adds a step.
  - One Checkout page model from all four stores of `Review`, because D
    with the combined payment gate of Checkout gives the same cover with
    less change.
- **Wider fix:** B. A portal control that owns its feedback cannot drop the
  result on any surface that uses it.
- **Previous examples:** The workspace banner and the Plan panel already
  show their portal failures. No earlier defect of this class was found.
- **Related:**
  - `plans/follow-ups/billing-cached-role.md` gives the effective payer
    permission that decides whether each portal button shows. B takes that
    permission outside the shared control, so apply B with that fix or
    after it.
  - `plans/follow-ups/stripe-return-workspace.md`, same component: its C
    routes the open-Checkout link of `ResultAction` through one shared
    exit helper. If both dashboard fixes land, the portal control of B can
    use that helper. Its recommended A changes none of these files, and
    neither item waits for the other.
- **Recommendation:** B. One portal control owns progress and failure on
  every surface. D is the direct option if a later split is needed.
- **Action:** fix after merge, apply B, because Update card without a Stripe
  page needs an unusual server state, and the fix costs the same later.
