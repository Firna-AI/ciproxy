# Another browser cannot continue an open payment

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the billing pages.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/wire/billing.rs:55`,
  `crates/ciproxy-api/src/handlers/checkout.rs:52`,
  `crates/ciproxy-billing/src/service/checkout.rs:29`,
  `crates/ciproxy-billing/src/service/commands.rs:107`,
  `ts/app/src/view/billing-usage.ts:156`, and
  `docs/protocol/billing.md:111`.
- **Reason:** The fix needs a new API answer, which the UI branch does not
  own.

- **Terms:**
  - **Open payment:** a Stripe Checkout that the person has not finished,
    or a paid change that waits for a bank confirmation on the hosted
    invoice page.
  - **Pending:** `payment_pending` of `PlanStatus` is true, because a
    Checkout or a payment command is open, at
    `crates/ciproxy-api/src/handlers/page_context.rs:177`.
  - **Quote:** the priced review of one exact choice, which the server
    stores, and which a payment request names.
- **Background:**
  - `docs/protocol/billing.md:111-114` says that while Stripe confirms a
    payment, the Plans page opens no second checkout, because no page asks
    for a second payment while the first one confirms. A checkout that
    does not finish offers to try again, at `:117-120`. When the bank asks
    to confirm a charge, the Stripe payment page opens, at `:157`.
  - `POST /checkout` answers `open` with the hosted Checkout URL. A paid
    change, through `POST /billing/change` or `PUT /billing/add-ons`,
    answers `payment_required` with the hosted invoice URL of the latest
    open subscription-update invoice, at `service/commands.rs:161`.
  - The server keeps the open Checkout. `CheckoutProgress` at
    `crates/ciproxy-store-interface/src/billing/commands.rs:47` holds its
    `session_id` and its hosted `url`, until Stripe confirms or expires
    it. The hosted invoice URL of a paid change is not stored. It comes
    from Stripe during the change.
  - The server already answers a repeated request with the open payment:
    - A repeated `POST /checkout` reads the stored session from Stripe,
      and answers `open` with its URL while it is open, whatever the
      selection, at `service/checkout.rs:29` and `:119`.
    - Both paid change routes go through `change_items` at
      `service/commands.rs:48`. A repeat with the same selection reuses
      the open command and answers the invoice URL again, at `:107` and
      `:161`. Another selection gets `409`, at `:110`.
  - On 8 November 2026, the user chose server-stored quotes for the paid
    review. Each payment request names the quote of its review, and the
    server refuses an unknown, used, or outdated quote. Pull request #39
    adds that contract. Another browser does not hold the quote of an
    open payment.
  - The review answers `409` while a Checkout or a command is open, at
    `handlers/checkout.rs:52`, so the Plans page cannot reach a payment
    request while pending.
  - `PlanStatus` at `wire/billing.rs:55` has `payment_pending`,
    `operation_in_progress`, and `can_pay`, but no URL and no selection of
    the open payment.
  - The Checkout form sets no `expires_at`, at
    `crates/ciproxy-billing/src/stripe/forms.rs:20`, so an open Checkout
    stays open until the provider session expires.
  - The dashboard keeps the exact URL of the last `open` or
    `payment_required` answer in the storage of the tab that received it,
    one record for each workspace, in
    `ts/app/src/state/billing-open-payment.ts`. While a fresh read says
    pending and `can_pay`, Plan and usage and Plans offer "Continue
    payment" with that URL. A confirmed or not-pending read clears it.
- **Problem:** Another tab, another browser, or a tab whose storage is gone
  has no URL. Plan and usage then says "Stripe is confirming a payment"
  with no action, and the review refuses a new choice. So the person can
  neither continue nor replace the open payment.
- **Impact if no action:** A payer who left an unfinished Checkout on
  another device cannot pay from the dashboard until the provider session
  expires. A bank confirmation that opens in a lost tab cannot be found
  again from the dashboard. Jobs keep running, because the limits are
  soft.
- **Root question:** How does any payer, in any browser, continue the exact
  open payment of a workspace?
- **Options:**
  - **A.** `PlanStatus` gives a payer the open payment: its kind and its
    URL. The Checkout URL comes from `CheckoutProgress`. The invoice URL
    needs a stored copy, or a Stripe read during each page read. Effort:
    Medium.
  - **B.** A payer-only resume route answers the exact open payment: the
    stored Checkout URL, or the hosted invoice URL of the open
    subscription-update invoice read from Stripe, or a typed answer that
    nothing is open. Page reads stay as they are, and the dashboard calls
    the route only while pending with `can_pay` and no saved URL. Effort:
    Medium.
- **Left out:**
  - A repeat of the payment request, with the selection of the open
    payment in `PlanStatus`. The server answers such a repeat with the open
    payment today. Under the quote contract of 8 November 2026, a payment
    request names the quote of its review. Another browser holds no such
    quote, the review refuses a new one while the payment is open, and the
    server refuses a used quote.
  - A new Checkout while one is open, because no page may ask for a second
    payment.
  - A dashboard that ends an open session at Stripe, because it could end
    a payment that the person completes on another device.
  - A shorter Checkout expiry through `expires_at`, because it cannot
    continue the exact open Checkout or a bank confirmation. It only ends
    the wait sooner.
- **Wider fix:** B. One server answer covers Checkout and bank
  confirmation in every browser.
- **Previous examples:** The same-tab case, which this branch fixed with
  the tab record of `ts/app/src/state/billing-open-payment.ts`.
- **Recommendation:** B. It gives the exact open payment from the server,
  for both kinds, with no payment request and no provider URL in every
  page read.
- **Action:** fix after merge, apply B, because the change is API work
  outside the UI branch, and the protocol already requires a page that can
  finish or retry a payment.
