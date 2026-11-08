# Billing store

These traits and types form the billing area of
[ciproxy-store-interface](../../README.md).
[Billing](../../../../docs/protocol/billing.md) defines the requirements.

BillingStore adds the confirmed tier, add-ons, paid bounds, and Free anchor.
It exposes retry and ended-banner dates without changing job execution.
BillingSession owns exclusive command and event reconciliation.
It reads, saves, and consumes paid quotes inside the locked workspace.
A `StoredQuote` keeps the reviewed selection, amounts, source, and expiry.
`consume_quote` marks one unused quote and saves its command atomically.
It returns false for an unknown or used quote, or while a command is pending.
Saving a quote prunes expired quotes while no command is pending.
It retains each owned renewal outcome separately from the billing banner.
Each renewal retains whether the provider attempted its payment.
Verified automatic collection stops remain available after a service restart.
Each stop event retains its eligible, superseded, or completed decision.
The subscription-ID lookup resolves a billing workspace before provider reads.
State and event receipts commit atomically.
App removal stores a durable billing cancellation request.
Services consume these boundaries through dyn.
