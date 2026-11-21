# Paid moves

This page states how a move between paid tiers is reviewed, paid, and
checked. [Changing tiers](billing.md#changing-tiers) states the other rules
of a move.

## The quote

- The review of a move between paid tiers gives the new monthly total, the
  prorated charge of today, and the day of each later charge.
- ciproxy stores each such review as a quote with its selection, amounts,
  current period, and proration time.
- The proration time is the time of the review.
- The move names its quote, and ciproxy takes the selection and the
  proration time from that quote, not from the browser.
- Stripe prorates the move as of that time, so the charge matches the review.
- A quote starts at most one move.
- A new move can start from a quote for 30 minutes after the review, and
  only before the period ends.
- A retry of the move names the same quote.
- Each retry of the move keeps its proration time and its amounts, also
  after a restart or after those 30 minutes.
- ciproxy refuses to start a move from an unknown or expired quote, the
  quote of another workspace, or a quote whose move ended.
- ciproxy also refuses to start it when the subscription, its tier and
  add-ons, its billing day, its period, its earlier credits and charges, or
  the credit balance of the customer changed after the review.
- The page then asks the customer to review the move again.
- Before it asks for the new review, the page reads the billing state of
  the workspace again.
- The page asks for the new review only when that read succeeds and shows
  that the move can still start.
- The new review starts no payment.
- The move starts only when the customer confirms the new review, also
  when its charge did not change.
- **Decision:** each review needs its own confirmation. Rejected: a move
  that starts by itself after the new review. Reason: Stripe makes no
  charge that the customer did not confirm.
- **Decision:** ciproxy stores each quote. Rejected: a signed quote that the
  browser sends back, and a new preview with a second confirmation. Reason:
  a stored quote best keeps the move and its retries the same as the review.

## Earlier credits and charges

- An earlier change in the period can leave a credit or a charge that
  Stripe keeps for a later invoice. An example is the credit of an extra
  machine that the customer removed.
- The charge of today in the review of a move up includes those credits
  and charges.
- The payment of a move up carries the same credits and charges, and
  Stripe computes the charge from them, so the charge matches the review.
- Each of those credits and charges reaches the customer once, so a later
  invoice does not repeat it.
- When a move up ends unpaid, the value of those credits and charges goes
  to a later invoice.
- **Decision:** the payment of a move up carries the credits and charges
  that its review counted. Rejected: taking them out of the review,
  computing the charge in ciproxy, invoicing them before the move, and
  invoicing each add-on change at once. Reason: Stripe keeps computing the
  charge.

## The invoice check

- After each paid move, ciproxy compares Stripe's invoice for the move with
  its quote.
- When Stripe makes no invoice for the move, ciproxy records that it has
  none.
- ciproxy never compares a renewal invoice as the invoice of a move.
- When they differ, ciproxy records the difference and reports it as an
  error event for the ciproxy team.
- The record and the event hold no secret and no card data.
- A difference does not refuse or roll back the payment or the new tier.
- Reason: the check catches a later difference between Stripe's charge and
  the quote.

## Related docs

- [Billing](billing.md): the tiers, the add-ons, Checkout, and the
  subscription states.
