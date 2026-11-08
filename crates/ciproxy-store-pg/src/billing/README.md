# Billing queries

These queries implement the billing area of
[ciproxy-store-interface](../../../ciproxy-store-interface/src/billing/README.md).
The [store crate](../../README.md) applies their migrations.

Billing uses a separate live session lock per workspace.
Its subscription-ID query uses the retained unique billing-account identity.
Each command persists before an external Stripe call.
Paid quotes use a workspace-scoped table with their expiry and command.
Consumption locks the workspace row, marks an unused quote, and saves its
command in one transaction.
Saving a quote deletes expired workspace quotes while no command is pending.
The same consumption transaction inserts the `billing_payments` journal row.
Each row keeps one journal as JSON under its command ID, with a pending flag.
A partial index finds the workspaces with pending payment work.
Those rows survive quote pruning and command cleanup.
They cascade with the workspace purge.
The shared read model exposes confirmed coverage and UTC period anchors.
A transaction commits event receipts, state, and command completion together.
A failed state write rolls its receipt back.
Renewal outcomes use separate workspace-scoped invoice records.
They preserve recovery and verified automatic stops after banner fields clear.
An exclusive session saves stop evidence before a provider cancellation request.
The same renewal record retains every stop event's cancellation decision.
Superseded and completed decisions survive later events and restarts.
The last organization removal also persists a cancellation request.
It stays until billing handles it, including unfinished Checkout.
