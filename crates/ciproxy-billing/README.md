# ciproxy-billing

Stripe billing for a ciproxy workspace.
Dashboard and webhook callers use this crate through dyn BillingApi.
Persistence belongs to ciproxy-store-interface and ciproxy-store-pg.
This crate has no open licence.

## Responsibilities

Keep confirmed tiers, add-ons, payment state, and period dates with Stripe.
Keep authorization, signed-event verification, and command retries together.
Keep Stripe keys and signing secrets out of diagnostics.

## What This Crate Does

Provisions the four monthly USD prices and a restricted customer portal.
Free creates neither a Stripe customer nor a card.
Admin and Billing roles can pay and change billing.
A Viewer can read the billing state and banner dates.
The authenticated host supplies the current role and workspace identifier.
Never build BillingAccess from client JSON.

Checkout uses cards and no tax.
It keeps one durable command through lost replies and payment confirmation.
Discovery finds an existing session by its saved command metadata before replay.
This also covers retries after Stripe removes an old idempotency key.
Checkout return and webhook paths retrieve current session state.
They apply confirmed coverage through the same reconciliation method.
Paid changes preserve the Stripe period and prorate the difference.
`PaidReview` quotes an exact paid selection at the supplied observation time.
It holds the workspace billing lock and verifies the subscription owner.
It rounds that time down to a whole second for Stripe's proration date.
A time outside the current Stripe period returns ReviewRequired.
It saves a `StoredQuote`, then returns `PaidQuote` with its id and expiry.
The quote freezes the reviewed items, catalog prices, upgrade mode, and amounts.
It also keeps the source subscription, customer, items, anchor, and period.
A new payment can use it for 30 minutes, and only before the period ends.
The quote reads both period bounds from that same subscription snapshot.
It leaves the stored period unchanged while renewal delivery is late.
The HTTP adapter renders preview and payment fields with one form builder.
It also requests a recurring preview for the new monthly total.
An upgrade uses the immediate preview's amount due for today's charge.
A downgrade keeps its credit for the later invoice and charges nothing today.
Signed invoice totals preserve negative downgrade credits.
The recurring day comes from Stripe's explicit anchor configuration or anchor.
The current period start never supplies that day.
Upgrades invoice at once with pending_if_incomplete.
The old tier stays until payment succeeds.
An open subscription-update invoice supplies its upgrade payment page.
A renewal invoice never supplies that page.
Confirmed expiry releases the old change for a fresh attempt.
Use change only with the id of a stored quote.
An unused quote must be unexpired, and its source must match the subscription.
Then one transaction consumes it and saves its Change command before Stripe.
The command copies the quote's proration time and item replacements.
Its retries reuse them with the same idempotency key, also after expiry.
A used quote recovers only its own unfinished command.
An unknown quote, or one that fails these rules, returns ReviewRequired.
A valid unused quote gets PaymentConfirming while another payment is unfinished.
These refusals send no Stripe mutation.
Use change_add_ons for quantities alone. It reconciles the current Stripe tier
inside the billing lock before it resolves the selection.
Its quantity-only commands carry no quote.
Both operations share durable commands, retry recovery, and Stripe proration.
Add-on changes apply at once with create_prorations.
The portal permits invoices, card changes, and period-end cancellation.
It permits undo-cancellation before the period ends.
It disables tier and quantity edits.

Each command holds a dedicated PostgreSQL billing session lock.
The command persists before the Stripe request.
A retry reads current items before it replays a paid change.
A definite rejected mutation releases its command for corrected input.
Failed reads, HTTP 409 or 429, server errors, and network errors retain the
command while its outcome is unknown.
Every reconciliation path saves an observed pending expiry for a Change.
After a fresh read shows no pending update, an applied or ended result wins.
An observed expiry or voided upgrade invoice can release the Change sooner.
Otherwise, that read releases it 24 hours after the command started.
This fallback applies to Change only.
It never releases Checkout, Cancel, or a still-pending Stripe update.
A pending change without matching saved input keeps PaymentConfirming.
It creates no new command and offers no unrelated payment page.
Current object reads also prevent delayed events from restoring old state.
Known subscription events resolve their workspace from stored Stripe IDs.
Signed invoice parents use the same lookup.
These events acquire the billing lock and check receipts before Stripe reads.
Unknown identities retain discovery routing and cannot apply unowned snapshots.
Every owned subscription and invoice read applies collection observations
before another read, including expanded invoices and collection pauses.
Returned mutation snapshots use the same validation and observation path.
Shared completion rules release paid commands when a subscription ends.
A late end event preserves a new Checkout command.
Direct paid commands reconcile an ended subscription before any mutation.
State, command completion, and an event receipt commit in one transaction.
The store creates the Free anchor when it creates the workspace.
Confirmed paid bounds and later Free anchors are available to metering.
Billing changes no job, organization, or machine record.

Last-organization removal stores a cancellation request in its transaction.
Suspension and removal of another organization leave billing alone.
The host polls BillingStore::billing_cancellations and calls
BillingApi::reconcile_removal for every result.
Retry Busy and provider failures.
The request stays durable during unfinished Checkout and failed HTTP calls.
The paid tier and add-ons stay until Stripe actually ends the subscription.
Billing retains each renewal outcome independently of the upgrade and banner.
It refreshes open renewals through their own invoice reads.
Paid and void outcomes stay durable after their banner clears.
Later subscription events cannot restore a recovered renewal failure.
A change saves a renewal seen before its Stripe mutation.
A delayed failed-payment event can also supply the renewal reference.
The upgrade invoice supplies only its payment page and change progress.
Only an owned open unpaid renewal supplies the failed-payment state.
An explicitly unattempted renewal cannot supply that state.
Renewal records retain Stripe's attempted value across service restarts.
An unpaid renewal keeps that state even after another renewal or charge
succeeds.
A paid renewal, or subscription end, clears its failure.
Signed automatic collection stops need fresh matching invoice and subscription
reads before they can end coverage.
Manual stops, paused collection, payment, and void invoices cannot request
an end.
The verified stop persists before an immediate Stripe cancellation request.
Each stop event keeps an Eligible, Superseded, or Completed decision.
A fresh mismatch supersedes that event before further reconciliation.
Its later deliveries cannot restore cancellation authority.
A distinct later automatic stop keeps its own decision.
Retries survive provider failures and a service restart.
The request creates no invoice or proration.
Coverage changes to Free only after Stripe confirms the same subscription ended.
A matching confirmed end completes eligible decisions, including after a
lost cancellation reply.
Failed renewal dates come from signed invoice.payment_failed events.
Reconciliation keeps the failed invoice identity and event date together.
It applies the date only to that concrete failed renewal.
Unrelated events cannot supply a missing failed-payment date.
An invoice creation date is not a failed-payment date.
The failed date can be absent until that event arrives.
The next retry can be absent while Stripe schedules it.
A UI shows the ended banner only before ended_banner_until.
That date is the end of the first Free month.
The caller applies banner priority and the payer-specific actions.

## Quick Start

Construct BillingService at the runtime composition boundary:

```rust
use std::sync::Arc;
use ciproxy_billing::clock::SystemClock;
use ciproxy_billing::service::api::BillingApi;
use ciproxy_billing::service::runtime::{BillingService, Redirects};
use ciproxy_billing::stripe::api::StripeApi;
use ciproxy_billing::webhook::SigningSecret;
use ciproxy_store_interface::billing::store::BillingStore;

fn billing(
    store: Arc<dyn BillingStore>,
    stripe: Arc<dyn StripeApi>,
    secret: SigningSecret,
    redirects: Redirects,
) -> Arc<dyn BillingApi> {
    Arc::new(BillingService::new(
        store, stripe, Arc::new(SystemClock), secret, redirects,
    ))
}
```

The service and PostgreSQL queries block.
An async HTTP host runs these calls on blocking workers.
Pass the exact raw webhook body and Stripe-Signature header to webhook.
Return a successful response after Applied, Duplicate, or Ignored.
Retry Busy and transient failures through Stripe delivery retries.
Reject InvalidSignature.
Poll Confirming on the return page without opening another Checkout.
Use trusted dashboard redirects from runtime configuration.

## Development

Run from the repository root with PostgreSQL 15 and DATABASE_URL set:

```sh
cargo build --package ciproxy-billing
cargo test --package ciproxy-billing
cargo fmt --all -- --check
cargo clippy --package ciproxy-billing --all-targets -- -D warnings
```

Unit tests use unimock boundaries.
Integration tests use a real HTTP fake Stripe and isolated PostgreSQL databases.
Each scripted subscription mutation requires an expected request and outcome.
Literal request fixtures specify prices, quantities, deletions, and flags.
Complete form comparison catches missing, extra, and duplicate fields.
The checked provider reports mismatches on the calling test thread.
The fake also fails on unused required request pairs.
GET snapshot edits need no mutation pair.
The existing example fixtures are synthetic recorded-shape samples.
The fixtures under tests/_tests_/fixtures/live are minimal real test-mode
recordings from origin/tmp/billing-live-fixtures at 30dfcf4.
Their snapshot keys identify the recorded response source.
They retain actual event identifiers and provider fields.
Tests sign projected events with a throwaway test signing key.
Derived cases change only the fields needed for their stated scenario.
The fixtures contain no account credential, signing secret, or card data.
The optional listener below captures additional real signed test-mode events.
The complete repository gate remains the final verification step.

### Stripe test mode

The [Stripe test mode guide](stripe-test-mode.md) gives the smoke run, the
listener, the browser and account checks, and the external account setup.

### Key Code

- [src/service/api.rs](src/service/api.rs) owns the dyn BillingApi entry points.
- [src/service/runtime.rs](src/service/runtime.rs) constructs BillingService.
- [src/service/paid_review.rs](src/service/paid_review.rs) defines and
  implements the authorized paid-plan preview boundary and stores its quote.
- [src/service/commands.rs](src/service/commands.rs) turns a quote or
  quantities into one durable paid change command.
- [src/service/completion.rs](src/service/completion.rs) owns the rules for
  command completion.
- [src/service/renewal.rs](src/service/renewal.rs) refreshes failed renewals.
- [src/service/collection_stop.rs](src/service/collection_stop.rs) retries
  verified final failures and confirms the Stripe end.
- [src/service/collection_decisions.rs](src/service/collection_decisions.rs)
  owns every transition of a stop event's cancellation decision.
- [src/service/observations.rs](src/service/observations.rs) validates owned
  reads and persists their collection observations before returning.
- [src/service/event_routing.rs](src/service/event_routing.rs) resolves known
  subscription owners before provider reads and discovers unknown identities.
- [src/stripe/api.rs](src/stripe/api.rs) owns the dyn StripeApi boundary.
- [src/stripe/http/client.rs](src/stripe/http/client.rs) owns the HTTP adapter.
- [src/stripe/http/preview.rs](src/stripe/http/preview.rs) reads immediate
  and recurring invoice previews without changing subscription items.
- [src/stripe/forms.rs](src/stripe/forms.rs) renders the reviewed items and
  proration date for both the preview and the payment.
- [src/webhook.rs](src/webhook.rs) verifies raw signatures and routes events.
- [src/catalog.rs](src/catalog.rs) owns USD prices and checkout reviews.
- [tests/_tests_](tests/_tests_) contains the public integration scenarios.

### Related Docs

- [Billing](../../docs/protocol/billing.md) defines prices and state changes.
- [App removal](../../docs/protocol/app-removal.md) defines cancellation.
- [Workspaces](../../docs/protocol/workspaces.md) defines ownership.
- [Metering](../../docs/protocol/metering.md) defines UTC periods.
- [Stripe API versions](https://docs.stripe.com/api/versioning) gives the pin.
- [Stripe pending updates](https://docs.stripe.com/billing/subscriptions/pending-updates)
  defines payment-gated item changes.
- [Stripe webhooks](https://docs.stripe.com/webhooks) defines signed delivery.
- [Stripe testing](https://docs.stripe.com/testing) gives test cards and clocks.
