# Stripe test mode

This guide belongs to the [ciproxy-billing README](README.md#stripe-test-mode).

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

The upgrade step reviews a new quote, or retries the unfinished change's quote.
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
