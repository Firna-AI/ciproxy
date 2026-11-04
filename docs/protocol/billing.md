# Billing

This doc states the tiers of ciproxy, their add-ons, and how an
[organization](security.md#terms) pays for them through Stripe.
[Metering](metering.md#soft-limits) states how ciproxy counts the use
against a tier.

## Terms

- **tier:** Free, Team, or Business. Team and Business are the paid tiers.
  The [dashboard](../architecture.md#components) says "plan" for a tier, as
  in "Plan and usage" and "Change plan".
- **add-on:** extra [starts](metering.md#terms) or an extra
  [own machine](agent.md#terms) that a paid tier buys by the month.
- **subscription:** the Stripe subscription of a paid tier.

## Tiers

| | Free | Team | Business |
| --- | --- | --- | --- |
| Price per month | $0 | $49 | $299 |
| Starts per month | 2,000 | 25,000 | 150,000 |
| Own machines | 1 | 3 | 10 |
| Job history | 7 days | 30 days | 90 days |

- Prices are in US dollars.
- Stripe bills each paid tier monthly.
- A tier includes its starts in each [period](metering.md#periods).
- ciproxy keeps the details of each finished job for the job history of the
  tier.
- Every organization starts on Free.
- The customer picks a paid tier by hand, and ciproxy never picks one by
  itself.
- Free needs no Stripe customer and no card.
- No tier price includes the Boat cost of
  [overflow](overflow.md#the-boat-account).
- **Decision:** three tiers, each priced by the starts and the own machines
  that it includes. [Starts](metering.md#starts) gives the rejected price
  bases and the reason.
- **Open:** which features ship at launch, and in which tier. It waits for
  item 8, the tier features at launch.
- **Open:** whether the organizations of one Business subscription share
  one pool of starts and own machines. It waits for item 8, the tier
  features at launch.

## Add-ons

- Team and Business can buy add-ons, and Free cannot.
- The customer buys extra starts in blocks of 10,000, for $10 a month for
  each block.
- The customer buys extra own machines for $20 a month each.
- [Soft limits](metering.md#soft-limits) states how the add-ons count
  against the use.
- The customer sets the add-on quantities in the checkout and on Plan and
  usage.
- Plan and usage shows the new monthly total of a change, and a save then
  applies it.
- A saved change takes effect at once.
- Stripe prorates the difference for the rest of the period.
- ciproxy never changes an add-on quantity by itself.
- The add-ons end when the subscription ends.

## Checkout

- Only an Admin or a person with the Billing role can choose a tier, change
  the add-ons, pay, or open the customer portal with its invoices.
- [Roles](security.md#roles) defines these roles.
- **Decision:** the roles of ciproxy decide who pays. Rejected: the billing
  managers of the GitHub organization, because the GitHub App would then
  need the "Members" permission.
- A person who cannot pay still sees the banners of a failed payment, and
  the [offer](metering.md#the-use-and-the-offer) at a soft limit.
- In place of the action of a banner or an offer, that person sees the
  words "Ask an Admin or a Billing person".
- **Decision:** a person who cannot pay sees the state without the action.
  Rejected: hiding the banners from that person. Reason: everyone sees the
  state, and only the payers act.
- A move from Free to a paid tier goes through Stripe Checkout.
- The customer picks the tier and the add-on quantities in the dashboard,
  then reviews the monthly total.
- The review says what Stripe charges today and the day of each later
  charge, for example "the 4th of each month".
- The customer then pays on Stripe Checkout.
- ciproxy changes the tier only when Stripe confirms the payment.
- The return page waits a few seconds for the confirmation.
- After the confirmation, the organization is on the new tier.
- Stripe emails the receipt.
- A checkout that does not finish leaves the tier as it was.
- The return page then says that the tier stays as it was and that jobs
  keep running.
- It offers to try again.
- **Decision:** Stripe takes every payment. Reason: no card data reaches
  ciproxy, and the ciproxy team already uses Stripe for billing.
- **Open:** how a visitor reaches a paid tier from the pricing page of the
  site. It waits for item 10, the paid-tier flow from the pricing page.

## Customer portal

- On a paid tier, "Invoices and card" on Plan and usage opens the Stripe
  customer portal.
- The portal shows the invoices.
- The portal changes the card.
- The portal cancels the subscription at the end of the period.
- Until the end of the period, the portal also undoes the cancel.
- The portal changes neither the tier nor the add-ons.

## Changing tiers

- For a move between paid tiers, the customer picks the new tier and its
  add-on quantities in the dashboard.
- A move between paid tiers takes effect at once.
- Stripe prorates the price difference for the rest of the period.
- A move to Free that the customer chooses takes effect at the end of the
  paid period.
- The customer chooses it with "Move to Free" on the Plans page, or with a
  cancel in the portal.
- The organization keeps the paid tier and its add-ons until the end of the
  period.
- After the move to Free, jobs keep running, also past the soft limits of
  Free.
- Until the end of the period, Plan and usage shows the day on which the
  paid tier ends.
- Until then, the customer can undo the move to Free in the Stripe
  customer portal.
- The Plans page marks the current tier, and the customer cannot choose it
  again.
- The Plans page also marks the tier that covers the use, as
  [Soft limits](metering.md#soft-limits) states.

## Subscription states

- A subscription is active, payment failed, or ended.
- While the subscription is active, the organization has its paid tier.
- While the subscription is active, Plan and usage shows the day on which
  it renews.
- The subscription is in the payment failed state while Stripe retries a
  failed payment.
- The organization keeps its paid tier while Stripe retries.
- A successful payment makes the subscription active again.
- The subscription ends after the last failed retry, or at the end of the
  period after a move to Free.
- When the subscription ends, the organization moves to Free that day.
- No subscription state stops a job or keeps a job from starting.

## Failed payments

- When Stripe cannot charge the card, every page of the organization shows
  a banner.
- The banner gives the date of the failed payment and the date of the next
  try.
- The banner says that the tier stays active and that jobs keep running.
- The banner offers "Update card", which opens the Stripe customer portal.
- Stripe retries the payment by its retry settings.
- After the last failed retry, Stripe ends the subscription.
- Every page then shows that the tier ended, says that jobs keep running on
  all own machines, and offers "Choose a plan".
- This banner takes the place of the soft limit banner.
- This banner shows until the first Free period ends, or until the
  organization moves to a paid tier.

## Related docs

- [Architecture](../architecture.md#data-flow): the data flow of a checkout
  and of a failed payment.
- [Metering](metering.md#soft-limits): the starts, the periods, the machine
  count, and the soft limits.
- [Security](security.md#key-custody): the custody of the Stripe keys and
  of the card.
- [Overflow](overflow.md#the-boat-account): the Boat account of the
  customer, which Boat bills for overflow.
