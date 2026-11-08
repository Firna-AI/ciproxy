# Billing store

These traits and types form the billing area of
[ciproxy-store-interface](../../README.md).
[Billing](../../../../docs/protocol/billing.md) defines the requirements.

BillingStore adds the confirmed tier, add-ons, paid bounds, and Free anchor.
It exposes retry and ended-banner dates without changing job execution.
BillingSession owns exclusive command and event reconciliation.
It retains each owned renewal outcome separately from the billing banner.
Each renewal retains whether the provider attempted its payment.
Verified automatic collection stops remain available after a service restart.
Each stop event retains its eligible, superseded, or completed decision.
The subscription-ID lookup resolves a billing workspace before provider reads.
State and event receipts commit atomically.
App removal stores a durable billing cancellation request.
BillingSession reads, saves, and consumes paid quotes in the locked workspace.
A `StoredQuote` keeps the reviewed selection, amounts, source, and expiry.
Its `QuotePayment` can hold a frozen signed offset with its source snapshots.
It also holds the customer balance that the immediate preview counted.
`consume_quote` marks one unused quote and saves its command atomically.
It returns false for an unknown or used quote, or while a command is pending.
Saving a quote prunes expired quotes while no command is pending.
Quote consumption creates a `PaymentWork` journal atomically with the command.
`PaymentWork` keeps the immutable quote and offset facts and source retirement.
It keeps the submission and exact-invoice state and the restoration identity.
It also keeps the audit and alert-emission state.
It survives quote pruning and command cleanup.
The workspace purge deletes it, also when it is unfinished.
BillingSession reads it with `payment` and `payments`, and saves it with
`save_payment`.
`BillingStore::billing_payment_workspaces` finds the workspaces whose payment
work is unfinished.
Services consume these boundaries through dyn.
