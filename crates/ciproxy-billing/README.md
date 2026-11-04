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
Upgrades invoice at once with pending_if_incomplete.
The old tier stays until payment succeeds.
An open subscription-update invoice supplies its upgrade payment page.
A renewal invoice never supplies that page.
Confirmed expiry releases the old change for a fresh attempt.
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
Billing tracks the failed renewal invoice independently of the upgrade invoice.
It refreshes the renewal retry date through its own invoice read.
A change saves a renewal seen before its Stripe mutation.
A delayed failed-payment event can also supply the renewal reference.
The upgrade invoice supplies only its payment page and change progress.
Past-due subscriptions keep the failed-payment state while details arrive.
An unpaid renewal keeps that state even after another charge succeeds.
A paid renewal, or subscription end, clears its failure.
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
Fixtures retain Stripe's event shape with public documentation example values.
They contain no account credential or card data.
They are synthetic recorded-shape samples, not a claimed live Stripe capture.
Real signed test-mode recordings come from the optional listener below.
The complete repository gate remains the final verification step.

### Stripe test mode

Use a dedicated Stripe test sandbox for the smoke run.

The smoke runner reads credentials only from environment variables.
It refuses live-mode keys and hides errors that could contain credentials.
It changes only the named test workspace and Stripe test-mode objects.
Keep DATABASE_URL on a disposable test database.
Set STRIPE_SECRET_KEY and STRIPE_WEBHOOK_SECRET in the environment.
Get them from the Stripe test-mode account and the test event destination.
Do not put them in a command, source file, or terminal log.
Set CIPROXY_STRIPE_TEST_WORKSPACE to a fresh UUID.
Optional CIPROXY_STRIPE_TEST_RETURN_ORIGIN sets the dashboard return origin.

```sh
crates/ciproxy-billing/scripts/stripe-test-mode.sh provision
crates/ciproxy-billing/scripts/stripe-test-mode.sh listen
```

In another terminal, forward real signed Stripe events to the local listener.
Configure the CLI with an environment key.
Keep its secret-bearing startup output in a private local file.
Set STRIPE_WEBHOOK_SECRET to that listener's signing secret before startup.
A deployed host instead registers its HTTPS webhook destination.
Use API version 2026-10-28.endive for that destination.
Subscribe to Checkout completed, async payment, and expiration events.
Subscribe to subscription created, updated, deleted, and pending-update events.
Subscribe to invoice paid, payment_failed, payment_action_required, and updated.
Also subscribe to invoice.marked_uncollectible.
The listener saves sanitized real events under
.context/ciproxy-mvp/billing/stripe-test-mode and prints no raw event data.

```sh
crates/ciproxy-billing/scripts/stripe-test-mode.sh checkout
```

Pay in the displayed Stripe page with the documented test card.
The initial Team selection has two starts blocks and one extra machine.
Its monthly and first payment total is $89 with no tax.
Set CIPROXY_STRIPE_TEST_SESSION to the printed session identifier.
Then run these steps:

```sh
crates/ciproxy-billing/scripts/stripe-test-mode.sh reconcile
crates/ciproxy-billing/scripts/stripe-test-mode.sh upgrade
crates/ciproxy-billing/scripts/stripe-test-mode.sh addons
crates/ciproxy-billing/scripts/stripe-test-mode.sh portal
crates/ciproxy-billing/scripts/stripe-test-mode.sh cancel
crates/ciproxy-billing/scripts/stripe-test-mode.sh status
```

Check the immediate prorated upgrade charge and the unchanged paid bounds.
Check the saved card, invoice list, cancel, and undo-cancel in the portal.
The portal must offer no tier or quantity editor.
Use Stripe test cards for failed charges and bank confirmation.
Use a test-clock customer or Stripe's subscription simulation to exercise
renewal retries, recovery, final failure, and the actual paid-period end.
Check that a failed upgrade on an Active subscription retains Team.
It must not create a renewal banner.
Also fail a renewal, then attempt an upgrade before that renewal recovers.
Keep the renewal banner and its failed-payment date after the upgrade invoice
becomes latest, including after its unpaid update expires.
Check that later renewal retries update the next-try date.
Delay the renewal event until after the upgrade, then repeat its delivery.
The renewal invoice and failure date must still appear.
Pay the renewal and check that its banner clears.
End a subscription with a paid command still pending.
Check that a new Checkout opens and a late old end event preserves it.
Lose an unpaid upgrade reply close to renewal.
Let a webhook observe its expiry, then let renewal replace the latest invoice.
Check that a different selection works after the observed expiry.
Repeat with no observation during the pending window.
A current read must keep it before 24 hours and release it at 24 hours.
A still-pending update must stay protected after that limit.
An applied selection must keep its paid result after that limit.
An old-selection retry must never offer the renewal invoice as an upgrade page.
Check that Move to Free leaves no payment page for an unrelated new selection
while Stripe still holds the old pending update.
Check signed recordings and repeat deliveries through the listener.
These browser and account steps need real Stripe test-mode credentials.
An offline test result does not prove that they passed.

External account setup must enable receipts for successful payments.
It must set the failed-payment retry schedule.
It must cancel the subscription after the last failed retry.
No tax registration, tax rate, or Stripe Tax setup is required.
Set production keys through secret configuration when the service is deployed.
Never run this smoke script with a live key.

### Key Code

- [src/service/api.rs](src/service/api.rs) owns the dyn BillingApi entry points.
- [src/service/runtime.rs](src/service/runtime.rs) constructs BillingService.
- [src/service/completion.rs](src/service/completion.rs) owns the rules for
  command completion.
- [src/service/renewal.rs](src/service/renewal.rs) refreshes failed renewals.
- [src/stripe/api.rs](src/stripe/api.rs) owns the dyn StripeApi boundary.
- [src/stripe/http/client.rs](src/stripe/http/client.rs) owns the HTTP adapter.
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
