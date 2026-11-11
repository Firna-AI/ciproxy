# Pages without the payment URL cannot continue it

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from `conductor/m21-jobs-savings-and-billing-ui`,
  the handoff of the billing pages. Extended on 2026-11-08 with a new
  problem that the check of owner finding 1 of pull request #53 found,
  which has the same cause.
- **Source:** Implementation and Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-api/src/wire/billing.rs:56`,
  `crates/ciproxy-api/src/handlers/checkout.rs:52`,
  `crates/ciproxy-billing/src/service/checkout.rs:29`,
  `crates/ciproxy-billing/src/service/commands.rs:119-134`,
  `ts/app/src/view/billing-usage.ts:156`,
  `ts/app/src/screens/billing/checkout-screen.tsx:106-109`, and
  `docs/protocol/billing.md:115`.
- **Reason:** The fix needs a new API answer, which the UI branch does not
  own.

- **Terms:**
  - **Open payment:** a Stripe Checkout that the person has not finished,
    or a paid change that waits on the hosted invoice page of Stripe.
  - **Pending:** `payment_pending` of `PlanStatus` is true, because a
    Checkout or a payment command is open, at
    `crates/ciproxy-api/src/handlers/page_context.rs:177`.
  - **Quote:** the priced review of one exact choice, which the server
    stores, and which the request of a move between paid plans names.
- **Background:**
  - `docs/protocol/billing.md:115-118` says that while Stripe confirms a
    payment, the Plans page opens no second checkout, because no page asks
    for a second payment while the first one confirms. A checkout that
    does not finish offers to try again, at `:121-124`. When the bank asks
    to confirm a charge, the Stripe payment page opens, at `:161`.
  - `POST /checkout` answers `open` with the hosted Checkout URL. A paid
    change, through `POST /billing/change` or `PUT /billing/add-ons`,
    answers `payment_required` with the hosted invoice URL of the latest
    open subscription-update invoice, at `service/commands.rs:222-229`.
  - The server keeps the open Checkout. `CheckoutProgress` at
    `crates/ciproxy-store-interface/src/billing/commands.rs:51` holds its
    `session_id` and its hosted `url`, until Stripe confirms or expires
    it. The hosted invoice URL of a paid change is not stored. It comes
    from Stripe during the change.
  - The server already answers a repeated request with the open payment:
    - A repeated `POST /checkout` reads the stored session from Stripe,
      and answers `open` with its URL while it is open, whatever the
      selection, at `service/checkout.rs:29` and `:119`.
    - Both paid change routes go through `change_items` at
      `service/commands.rs:38`. A repeat of `POST /billing/change` with
      the quote of the open move reuses that move, also after the 30
      minutes of the quote, and answers the invoice URL again, at
      `:44-56`, `:119-132`, and `:222-229`. A repeat of
      `PUT /billing/add-ons` with the same add-ons does the same. Another
      request while a move is open gets `409`, at `:134`.
  - On 8 November 2026, the user chose server-stored quotes for the paid
    review. Pull request #39 added that contract, and it merged on that
    day as `f29911f`. A move between paid plans names the quote of its
    review, and only the quote of the open move continues that move.
    `docs/protocol/billing-paid-moves.md` names the quotes that the server
    refuses with `409 review_required`. Another browser does not hold the
    quote of an open payment.
  - The review answers `409` while a Checkout or a command is open, at
    `handlers/checkout.rs:52`, so the Plans page cannot reach a payment
    request while pending.
  - `PlanStatus` at `wire/billing.rs:56` has `payment_pending`,
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
  - The tab that started a paid change keeps the submission with its
    quote, so a retry continues the same move. On Checkout, an add-on edit
    resets the payment, at
    `ts/app/src/screens/billing/checkout-screen.tsx:106-109`, and drops
    that quote. Each new request first clears the saved URL, at
    `ts/app/src/state/billing-flows.ts:45-46`. So after a lost answer, or
    a `payment_required` answer with no URL, that tab keeps no URL. After
    a `payment_required` answer with a URL, Plan and usage still offers
    Continue payment.
- **Problem:** Another tab, another browser, or a tab whose storage is gone
  has no URL. The tab that started a paid change is in the same state when
  an add-on edit drops its quote after a lost answer. Plan and usage then
  says "Stripe is confirming a payment" with no action, and the review
  refuses a new choice. So the person can neither continue nor replace the
  open payment.
- **Impact if no action:** A payer who left an unfinished Checkout on
  another device cannot pay from the dashboard until the provider session
  expires. The Stripe page of a paid change that opened in a lost tab cannot
  be found again from the dashboard. Jobs keep running, because the limits are
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
    payment today. Under the quote contract of 8 November 2026, a move
    between paid plans names the quote of its review, and only the quote of
    the open move continues it. Another browser, or a tab whose edit
    dropped the quote, holds no such quote, and the review refuses a new
    one while the payment is open.
  - Keeping the quote of an unfinished paid change across an add-on edit
    on Checkout, because it covers only the tab that started the change. B
    covers that tab and every other case with one route.
  - A new Checkout while one is open, because no page may ask for a second
    payment.
  - A dashboard that ends an open session at Stripe, because it could end
    a payment that the person completes on another device.
  - A shorter Checkout expiry through `expires_at`, because it cannot
    continue the exact open Checkout or the Stripe page of a paid change. It
    only ends the wait sooner.
- **Wider fix:** B. One server answer covers Checkout and the Stripe page of
  a paid change in every browser.
- **Previous examples:** The same-tab case, which this branch fixed with
  the tab record of `ts/app/src/state/billing-open-payment.ts`.
- **Related:** `plans/follow-ups/saved-payment-lifetime.md` keeps the link
  that this tab received until its payment ends, so an older read cannot
  remove it. This item covers another browser that has no link. Neither
  item waits for the other.
- **Recommendation:** B. It gives the exact open payment from the server,
  for both kinds, with no payment request and no provider URL in every
  page read.
- **Action:** fix after merge, apply B, because the change is API work
  outside the UI branch, and the protocol already requires a page that can
  finish or retry a payment.
