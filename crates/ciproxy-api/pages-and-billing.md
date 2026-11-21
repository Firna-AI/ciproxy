# Jobs, Savings, and billing composition

This guide belongs to the [ciproxy-api README](README.md). That README names
the entry points under Key Code.

The primary hosted composition root supplies `PageServices`. Keep the existing
`ApiState` construction. Mount the page router and raw Stripe hook together:

```rust
use std::sync::Arc;
use axum::Router;
use ciproxy_api::pages::{PageServices, router_with_pages, stripe_router};
use ciproxy_api::state::ApiState;

fn dashboard(state: ApiState, pages: Arc<PageServices>) -> Router {
    Router::new()
        .nest("/api", router_with_pages(state.clone(), pages.clone()))
        .merge(stripe_router(state, pages.billing.clone()))
}
```

Use the existing `BillingService`, `StoredMetering`, and `Overflow` service.
Supply dated Blacksmith and GitHub USD price lists from trusted configuration.
Set `PageServices.boat_root_sizing` to the exact `OverflowConfig.root_sizing`
value before moving that configuration into the overflow service. The live
queue reuses this trusted sizing and the scaler's public size selection. It
adds no separate root allocation formula. Missing sizing stays unavailable.
An absent catalog returns `unavailable` for Savings. An absent overflow service
leaves live Boat observations explicitly unavailable. The existing
`router(state)` remains available to callers without these collaborators.

Supply `ciproxy_billing::service::paid_review::PaidReview` for paid changes.
The existing `BillingService` implements it with authoritative Stripe invoice
previews, and it stores each quote. A caller without it gets `unavailable` for
paid review. Free Checkout keeps `BillingApi::review`, its period, a null
`quote`, and the selection body of POST `/checkout`.

A paid change takes two requests. GET
`/workspaces/{workspace_id}/checkout/review` returns Stripe's amounts, the
current Stripe period end, and `quote`. That period stays current while stored
renewal state lags. POST `/workspaces/{workspace_id}/billing/change` then sends
only `quote_id`, with no selection, amount, or time. The server reads the
reviewed selection and proration time from that quote. Today's charge can
include earlier credits and charges of the period, and the payment carries them
too. Start a new payment before `quote.expires_at`. Retry a lost reply or an
`unavailable` error with the same `quote_id`. A started payment then continues,
also after that time.

HTTP 409 with Problem `code` `review_required` means that the quote cannot
start a payment.
[Paid moves](../../docs/protocol/billing-paid-moves.md#the-quote) gives the
cases. Reload the billing state first, because the payment of that
quote can be complete. Then fetch a new review, and ask the customer to confirm
it. On the change route, HTTP 409 with `code` `conflict` means that another
billing operation is unfinished. PUT
`/workspaces/{workspace_id}/billing/add-ons` needs no quote.

The control plane uses this composition and loads the Stripe signing secret.
Register `/hooks/stripe` in Stripe on the dashboard host. Run the checks in
[the live guide](scripts/README.md#jobs-savings-and-billing).
These live operations need the deployment owner's credential paths.

Jobs read all organizations of the authorized workspace. Both numeric Actions
jobs and exact scale-set attempts contribute rows. Mapped aliases keep one row.
The shared attempt shape fills unknown Actions facts while known Actions facts
stay authoritative. Result words remain verbatim from their trusted source.
The current plan limits finished-job visibility to seven, thirty, or ninety
days without deleting retained metadata. Queued jobs report the current label
request. Running and finished jobs report the retained execution request.
Their requested vCPUs, memory GB, and free-disk GB are null when that record is
absent. Label names remain available. Actual runner resources stay distinct
from requested free job disk. Missing real finish times remain unknown.
Trusted taken evidence stays true while a real start time is unknown. Period
accounting still needs the actual time.

Live waiting rows, waiting-label notices, and inactive banners include current
workspace members only. Removed organizations keep running jobs and history.
Only suspension of a current member makes the complete waiting count unknown.
Owner-removal notices link to the configured App install page. Intentional
take-outs add no removal notice.

The live response includes the fit of each exact label with waiting jobs.
Every role sees this data without access to the Admin-only Labels routes.
Admin receives the edit capability. Other roles receive an ask-Admin notice.
Busy slots and temporary provider refusals remain separate from physical fit.
An unknown spend bound keeps its original start hold and unavailable counted
spend. The overflow owner's resource-only accessor preserves the known catalog.
The response preserves unknown facts and the permitted largest-size fallback.

Every page includes one shared banner. Existing pages can read the banner route.
The Organizations tab requests its explicit page scope for inactive reasons.
Money crosses the HTTP boundary as integer-cent strings. Savings rounds only
the final displayed amount, with half cents rounded away from zero. Metering
keeps exact costs and runtime throughout the calculations.
