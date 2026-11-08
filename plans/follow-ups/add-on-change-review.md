# Add-on changes lack a price before save

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the billing pages.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/handlers/checkout.rs:69`,
  `crates/ciproxy-api/src/handlers/checkout.rs:163`,
  `crates/ciproxy-api/src/wire/billing.rs:90`,
  `ts/app/src/view/billing-addons.ts:15`, and
  `docs/protocol/billing.md:76`.
- **Reason:** The fix is a change of the API, which the UI branch does not
  own.

- **Terms:**
  - **Review:** `GET /api/workspaces/{id}/checkout/review`, the price of
    one exact plan choice: the monthly total, the charge of today, and the
    day of later charges.
  - **Same-plan choice:** the current paid plan with other add-on
    quantities, such as one more block of extra starts.
  - **Payment module:** `ts/app/src/api/billing-payment.ts`, the one place
    in the dashboard that sends a payment request.
  - **Quote:** the priced review of one exact choice, which the server
    stores, and which a payment request names.
- **Background:**
  - `docs/protocol/billing.md` says that the customer sets the add-on
    quantities in the checkout and on Plan and usage. Plan and usage shows
    the new monthly total of a change, and a save then applies it.
  - On 8 November 2026, the user chose server-stored quotes for the paid
    review. The review stores a quote, and the payment request names it.
    The server refuses an unknown, used, or outdated quote, and the
    dashboard then asks for a new review. Pull request #39 adds that
    contract. The fix of this item builds on it.
  - The review rejects a choice with the current plan. `checkout.rs:69`
    answers `409 conflict` when the chosen tier equals the confirmed tier,
    and also for Free.
  - `PlanUsagePage` in `wire/billing.rs:90` gives two priced choices only:
    `current`, the saved add-ons, and `offer`, the cheapest choice that
    covers the use. Each has `monthly_cents`. The page has no unit price.
  - `PUT /api/workspaces/{id}/billing/add-ons` at `checkout.rs:163` saves
    any quantities, but nothing prices them first.
  - The dashboard computes no price, so it can show a total only for
    `current` and for an `offer` that keeps the plan.
- **Problem:** A payer who edits the add-ons to any other quantity sees
  "Monthly total not available", at `ts/app/src/view/billing-addons.ts:47`.
  Save stays disabled, at `ts/app/src/screens/billing/add-ons-panel.tsx:128`.
  The same-plan offer is the only change that a payer can save.
- **Impact if no action:** A payer cannot lower the add-ons, or raise them
  to a quantity other than the offer, from Plan and usage. The protocol
  requirement of a total before each save stays unmet.
- **Root question:** How does the API price any same-plan add-on choice
  before the payer saves it?
- **Options:**
  - **A.** Let the review accept a same-plan choice. It prices the choice
    with the authoritative Stripe preview, the new monthly total and the
    prorated charge of today, and stores a quote, as it does for a plan
    change. The add-on save names that quote, and the payment module sends
    it. A refused quote sends the payer back to a new review. Effort:
    Medium.
  - **B.** Add a separate add-on review route that prices one exact add-on
    choice for the current plan, with the same fields, and stores a quote
    of the same contract. The add-on save names that quote. Effort:
    Medium.
- **Left out:**
  - Unit prices or price constants for the dashboard to multiply, because
    the dashboard must not compute a price.
  - Only the two priced choices of today, because they cannot price any
    other add-on quantity, so a payer still cannot see the total of that
    change before the save.
  - An add-on save that names no quote, because the quote contract of
    8 November 2026 makes each payment request name a stored quote.
- **Wider fix:** A. One review and one quote contract then price every
  change, and every payment keeps going through the one payment module.
- **Previous examples:** None found.
- **Recommendation:** A. It reuses the review, its quote, and the payment
  module, so no second price or payment path appears.
- **Action:** fix after merge, apply A, because the change is API work
  outside the UI branch, the protocol already requires the total, and the
  quote contract that the user chose on 8 November 2026 settles how the
  save names the price.
