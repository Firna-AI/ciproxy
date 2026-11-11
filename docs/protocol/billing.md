# Billing

This doc states the tiers of ciproxy, their add-ons, and how a
[workspace](workspaces.md#terms) pays for them through Stripe.
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
- Prices do not include tax, and ciproxy collects no tax.
- **Decision:** ciproxy collects no tax, for now. Rejected: Stripe Tax,
  prices that include tax, Stripe Managed Payments, and tax rates that
  ciproxy keeps itself. Reason: the launch then needs no tax setup.
- Stripe bills each paid tier monthly.
- A tier includes its starts in each [period](metering.md#periods).
- The job history of a tier is the window of finished jobs that its pages
  show. [Data retention](security.md#data-retention) says how long ciproxy
  keeps them.
- Every workspace starts on Free.
- The customer picks a paid tier by hand, and ciproxy never picks one by
  itself.
- Free needs no Stripe customer and no card.
- No tier price includes the Boat cost of
  [overflow](overflow.md#the-boat-account).
- **Decision:** three tiers, each priced by the starts and the own machines
  that it includes. [Starts](metering.md#starts) gives the rejected price
  bases and the reason.

### Tier features

- Every tier includes own Linux machines, Boat overflow, the
  [transparent cache](cache.md#transparent-cache), warm runners, the
  dashboard, and the [savings report](metering.md#savings-report).
- Team adds several GitHub organizations, sticky Docker disks, alerts by
  email and Slack, and email support.
- Business adds single sign-on, an audit log, custom VM images, and
  priority support.
- A tier also includes each feature of the tiers below it.
- Several GitHub organizations and sticky Docker disks ship at launch, as
  [Organizations of a workspace](workspaces.md#organizations-of-a-workspace)
  and [Docker disks](cache.md#docker-disks) say.
- Alerts by email and Slack, single sign-on, the audit log, and custom VM
  images come later.
- Each page that lists a feature that comes later marks it "Coming later".
- **Decision:** each tier keeps its features, and a feature that comes
  later shows "Coming later". Rejected: taking features out of the tiers
  until they ship, and building some or all of them before the launch.
  Reason: the tiers stay as priced, and no page sells a feature as ready
  before it ships.

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
  managers of the GitHub organization. The App's four permissions include
  "Members", read, for membership checks. ciproxy roles grant billing access.
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
- Checkout takes cards only.
- Stripe Checkout returns the customer to the return page with the
  workspace and the exact Checkout session of the payment.
- Reason: the return then works also in a browser that did not start the
  payment.
- The return page waits a few seconds for the confirmation.
- When Stripe still confirms the payment after that wait, the return page
  says so and updates by itself.
- While Stripe confirms a payment, Plan and usage says so, and the Plans
  page opens no second checkout.
- Reason: no page asks for a second payment while the first one
  confirms.
- After the confirmation, the workspace is on the new tier.
- Stripe emails the receipt.
- A checkout that does not finish leaves the tier as it was.
- The return page then says that the tier stays as it was and that jobs
  keep running.
- It offers to try again.
- **Decision:** Stripe takes every payment. Reason: no card data reaches
  ciproxy, and the ciproxy team already uses Stripe for billing.
- Every sign-in button of the
  [public site](../architecture.md#public-site) says "Start free", also on
  the paid tiers of the pricing page.
- **Decision:** every sign-in button of the site says "Start free".
  Rejected: a caption that explains the upgrade, a sign-in that carries
  the chosen tier into the dashboard, and Stripe Checkout before setup.
  Reason: every workspace starts on Free, and the
  [offer](metering.md#the-use-and-the-offer) at a soft limit brings each
  workspace that outgrows Free to the right tier.

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
- A move up, from Team to Business, needs no Stripe Checkout.
- Stripe charges the prorated difference of a move up at once, on the
  saved card.
- **Decision:** a move up charges the saved card at once. Reason: the
  higher tier starts at once.
- The tier of a move up changes only after its charge succeeds.
- When the charge fails, the workspace keeps its tier, and the page offers
  to try again or to update the card.
- When the bank asks the customer to confirm the charge, the Stripe payment
  page opens.
- A move to Free that the customer chooses takes effect at the end of the
  paid period.
- The customer chooses it with "Move to Free" on the Plans page, or with a
  cancel in the portal.
- The workspace keeps the paid tier and its add-ons until the end of the
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

### The quote of a paid move

- [Paid moves](billing-paid-moves.md) states the review, the quote, the
  payment, and the invoice check of a move between paid tiers.

## Subscription states

- A subscription is active, payment failed, or ended.
- While the subscription is active, the workspace has its paid tier.
- While the subscription is active, Plan and usage shows the day on which
  it renews.
- The subscription is in the payment failed state while Stripe retries a
  failed payment.
- The workspace keeps its paid tier while Stripe retries.
- A successful payment makes the subscription active again.
- The subscription ends after the last failed retry, or at the end of the
  period after a move to Free.
- The subscription also ends at the end of the period after the last
  organization of the workspace removes the GitHub App, as
  [The last organization](app-removal.md#the-last-organization-of-a-workspace)
  says.
- A removal of the App by any other organization of the workspace leaves
  the subscription as it is.
- When the subscription ends, the workspace moves to Free that day.
- No subscription state stops a job or keeps a job from starting.

## Failed payments

- When Stripe cannot charge the card, every page of the workspace shows a
  banner.
- The banner gives the date of the failed payment and the date of the next
  try.
- The banner says that the tier stays active and that jobs keep running.
- The banner offers "Update card", which opens the Stripe customer portal.
- Stripe retries the payment by its retry settings.
- After the last failed retry, Stripe ends the subscription.
- Every page then shows that the tier ended, says that jobs keep running on
  all own machines, and offers "Choose a plan".
- This banner shows until the first Free period ends, or until the
  workspace moves to a paid tier.
- The banner of a failed payment and the banner of an ended tier each take
  the place of the [soft limit](metering.md#soft-limits) banner.
- So a page shows one banner.
- **Decision:** at a failed payment, the payment banner takes the place of
  the soft limit banner. Rejected: the payment banner and the soft limit
  banner on one page. Reason: a page keeps one banner.

## Related docs

- [Architecture](../architecture.md#data-flow): the data flow of a checkout
  and of a failed payment.
- [Paid moves](billing-paid-moves.md): the quote, the payment, and the
  invoice check of a move between paid tiers.
- [Workspaces](workspaces.md#organizations-of-a-workspace): the
  organizations that a workspace on each tier can hold.
- [Metering](metering.md#soft-limits): the starts, the periods, the machine
  count, and the soft limits of a workspace.
- [Security](security.md#key-custody): the custody of the Stripe keys and
  of the card.
- [Removing or suspending the App](app-removal.md): the end of the
  subscription when the last organization of a workspace removes the App.
- [Cache](cache.md#docker-disks): the Docker disks of Team and Business.
- [Overflow](overflow.md#the-boat-account): the Boat account of the
  customer, which Boat bills for overflow.
