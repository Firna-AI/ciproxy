# Paid moves

This guide belongs to the [ciproxy-billing README](README.md).
[Paid moves](../../docs/protocol/billing-paid-moves.md) states the
requirements.

`QuotePayment` can also freeze a signed offset with its source snapshots.
The offset holds only the earlier pending items that its preview counted.
The quote also freezes the customer balance that the preview counted.
Before the first payment, a changed balance or source returns ReviewRequired.
Active retries keep the frozen amounts and do not reprice.
The payment adds the signed offset as one inline add_invoice_items price.
That one-time price uses an existing catalog product and is not discountable.
Stripe computes the charge from it, the frozen items, and the proration time.
Quote consumption creates a `PaymentWork` journal atomically with the command.
It keeps the immutable quote and offset facts and the state of each step.
It survives quote pruning and command cleanup.
The original pending items are removed before the paid mutation.
So a renewal invoice cannot count them twice.
A metadata-only update first stamps the quote and command identifiers.
The paid-change invoice carries them in its subscription metadata.
An exact paid invoice settles the transfer.
Definitive proof of an unpaid or never-sent payment restores the removed value.
Only a typed parameter-validation rejection proves that a sent update never ran.
The restore adds that signed value once to the Stripe customer balance.
A later invoice then uses it.
Uncertain outcomes keep the transfer pending and emit operator events.
A 24-hour limit or an ended subscription alone never proves an unpaid amount.
Audit-only pending work never holds settled money or blocks a new review.
After each paid change, billing compares the owned invoice with the quote.
It records each comparison.
A downgrade without an immediate invoice records that absence explicitly.
A renewal invoice is never compared as the invoice of a paid change.
BillingService reports each difference to its `BillingAlerts` collaborator.
`HostedBillingAlerts` emits an infallible typed error-level operator event.
The event holds no secret or card data, and it promises no email or page.
A difference never refuses or rolls back a confirmed payment or tier.
BillingApi::reconcile_payments resumes the pending payment work of a
workspace, also after a restart.
It recovers pending transfer, comparison, and event emission work.
Journals follow the workspace retention.
The workspace purge deletes them, also unfinished ones.
