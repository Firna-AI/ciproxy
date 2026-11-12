# Billing pages

Plan and usage, Plans, the Checkout review, and the return from Stripe
Checkout are the billing pages of the [dashboard](../../../README.md). They
show the plan and its use, and take each payment through Stripe.

## Prices and payments

- Prices, totals, offers, charges, and dates come from the API. The app
  computes no price.
- Plan and usage saves add-ons only at a quantity that the API prices: the
  saved add-ons, or the offer that keeps the plan. Another quantity shows
  "Monthly total not available" and cannot be saved.
- One payment runs at a time. While Stripe confirms a payment or a billing
  change runs, Plan and usage and Plans say so, read again every 5 seconds,
  and start no second payment. A Checkout that stays open on Stripe keeps the
  payment pending until Stripe ends the session.
- While a payment is pending, Plan and usage and Plans offer a person who
  may pay "Continue payment" with the exact Stripe address that this tab
  received from the API, for a Checkout or for the Stripe page of a paid
  change. Another tab or browser has no such address, so its page only says
  that Stripe confirms the payment.
- A payment that waits on its Stripe page says "Your payment is not
  finished", with "Open the Stripe page" and "Check again". It names the
  Stripe page, not a bank, because the API does not say why Stripe waits.
- One payment gate decides whether a payment may start: a payer role in the
  latest session read, a successful latest Plan and usage response, the API
  permission, and no payment or billing change that waits. When the gate is
  closed, Checkout says why: a failed Plan and usage read with Try again,
  before the words of a refused review; a person who cannot pay; a payment
  or a billing change in progress, with "Open plan and usage"; or "Checking
  your plan".
- A payment or a Move to Free whose result the page did not get, such as no
  answer, a server error, or a plain conflict, says only what the next Plan
  and usage read proves. A payment says "Unable to confirm this payment",
  with "Open plan and usage" and Try again, until that read shows the choice
  as the current plan or a payment in progress. The dialog of an unconfirmed
  Move to Free closes with "Close", because the cancellation may have taken
  effect. A refusal that the server names keeps its own words.
- Plan and usage shows each UTC date of the billing period, also its last
  date when the period ends during that day.

## The review and its quote

- The review shows the charge that the API gives. When the API cannot price a
  change, the review says so, and no payment starts.
- Every payment request goes through one module. It sends the reviewed or
  quoted choice, never the quantities that the person still edits. A move
  between paid plans sends only the ID of the quote of its review. Free
  Checkout sends the reviewed choice, and an add-on save the quoted add-ons.
- A review of a move between paid plans can pay only with its quote. Without
  it, Checkout says "Unable to price this change", with Try again, which
  reads the review again. No pay button shows.
- A retry of a payment, with Try again or Check again of its result, sends
  the same request again. So a paid change keeps the quote of its review,
  and the server continues the same payment.
- When the API answers a paid change with `409 review_required`, the pay
  button stays busy while the page reads Plan and usage again. That read
  also brings the current banner. Checkout shows the refusal only after the
  read, so it never judges the refusal from an older page.
- After that read, the payment gate decides first, as above. When the read
  shows the tier of the review as the current plan, Checkout says so, with
  no action. Otherwise Checkout says "This review is out of date", with
  Review again, and shows no price and no pay button.
- Review again reads a new review and sends no payment. Only a new click on
  the pay button of the new review pays, also when its charge did not
  change.
- A refused quote shows no payment result, and never the words of a payment
  whose result the page did not get.

## Stripe returns

The billing runtime names the workspace in each Stripe return. It has one
Checkout success URL, one cancel URL, and one portal return URL. The billing
service replaces `{WORKSPACE_ID}` in each with the workspace of the payer,
and Stripe replaces `{CHECKOUT_SESSION_ID}` with the exact Checkout session.
The [control plane](../../../../../crates/ciproxy-control-plane/README.md)
sets these URLs on its `PUBLIC_URL`, the dashboard origin:

```text
<origin>/checkout-return?workspace={WORKSPACE_ID}&session_id={CHECKOUT_SESSION_ID}
<origin>/plans?workspace={WORKSPACE_ID}
<origin>/billing?workspace={WORKSPACE_ID}
```

Before a tab leaves for Stripe Checkout or the Stripe portal, it saves the
workspace of its page in its own tab storage. `/checkout-return` and `/plans`
without `workspace` use the workspace saved before Checkout, and `/billing`
the one saved before the portal. Such a return comes from a configured URL
without `{WORKSPACE_ID}`, or from a Checkout that started before the runtime
filled the workspace, because a Checkout keeps the URLs that it saved. A
query with `workspace` always wins. With nothing saved, the return page says
that it cannot tell the workspace of the payment. It never guesses one.

The return page checks the session at once, and every 3 seconds while Stripe
confirms. Only a confirmation from the API changes the plan on the page. When
Stripe needs the person again, the page links to the Stripe address that the
API gives. After a confirmed payment, only an Admin gets Add machines, and a
Billing person gets only Open plan and usage.

## Key Code

- [checkout-screen.tsx](checkout-screen.tsx) is the Checkout review. Its pay
  button sends the submission of the ready summary. Its Review again calls
  `checkoutReviewAgain` of [billing-flows.ts](../../state/billing-flows.ts),
  which starts the read of a new review before it clears the refused
  payment.
- [billing-checkout-summary.ts](../../view/billing-checkout-summary.ts)
  decides what the summary shows: the price with the one submission that may
  pay, a closed payment gate, or a refused review.
- [billing-payment.ts](../../api/billing-payment.ts) sends every payment
  request. Its `planSubmission` gives no submission for a paid review without
  its quote, and its `isReviewRequired` finds a real refusal of a quote.
- The payment store of [billing-payment.ts](../../state/billing-payment.ts)
  runs one payment at a time. It shows a refused quote only after the Plan
  and usage read.

## Related docs

- [Checkout](../../../../../docs/protocol/billing.md#checkout) and
  [Paid moves](../../../../../docs/protocol/billing-paid-moves.md#the-quote)
  hold the requirements of these pages.
- The [dashboard scripts](../../../scripts/README.md#browser-checks) drive
  these pages in a browser.
