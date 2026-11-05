# ciproxy-metering

Workspace metering and savings calculations for the hosted control plane.
Use this crate for counts, advisory plan offers, and savings reports.
Use ciproxy-store-interface for trusted facts and persistent histories.
This crate has no open licence.

## Responsibilities

Keep pricing and calculation rules pure.
Keep clocks, provider calls, and database queries outside the calculations.
Consume persistence through dyn MeteringStore.
Expose runtime orchestration through dyn Metering.

## What This Crate Does

Counts runner-taken jobs across all organizations of one workspace.
Counts current registrations without a machine-health filter.
Keeps Free monthly anchors through short months.
Accepts Stripe's confirmed paid-period boundaries.
Computes forecasts from the UTC dates touched by each period.
The current date counts in the elapsed dates.
Computes advisory soft limits and the cheapest paid tier with add-ons.
An equal-price offer keeps the current paid tier, then prefers Team.
An offer never changes the saved plan or prevents a job.
Payment notices take precedence over a soft-limit notice.

Builds reports for the last thirty dates, this month, and last month.
The initial selection is the last thirty dates, including today.
Starts belong to their taken-at date.
Elapsed fractional minutes follow each UTC day's runtime overlap.
Running jobs stop at the supplied observation time.
Each partial own-machine registration date counts as one cost date.
Stored monthly prices supply machine and tier costs.
The last monthly price on each UTC date applies to the full date.
Each date uses the day count of its calendar month for both monthly costs.
Stored Boat rates supply start charges and elapsed sandbox costs.
A Boat sandbox can cost money without taking a job.
Each savings report reads registrations, range totals, and monthly costs
from one coherent store snapshot.
The report read loads no retained jobs, start counts, or accounting cursors.

Callers supply dated USD price lists for Blacksmith and GitHub.
Matching requires both CPU and memory.
A uniquely smallest fitting size supplies the comparison price.
Incomparable smallest sizes leave that comparison unavailable.
The report leads with the cheaper alternative when both costs are known.
Signed savings remain negative when ciproxy costs more.
Missing machine costs or registration times suppress savings.
Known costs and alternative comparisons remain visible.

This crate supplies no provider price catalog or currency conversion.
It applies no invoice, minute, or display rounding.
Missing monthly price history produces a CostGap.
Other known costs remain visible when a gap prevents a savings figure.

Anonymous daily totals retain actual VM sizes and Boat costs.
Anonymous timestamp counts preserve exact billing-period boundaries.
These aggregates retain no job or organization identifier.
Accounting cursors retain job and runner references only until deletion.
Their signed corrections make refresh idempotent.
Boat source costs and cost cursors come from one coherent store snapshot.
Each new observation preserves the stored cursor's later observation time.
Corrections include every affected UTC date through that time.
Organization deletion keeps the anonymous workspace aggregates.
Refresh removes aggregate dates outside the ninety retained UTC dates.

## Quick Start

Construct the service in the composition root:

```rust
use std::sync::Arc;
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_store_interface::metering::store::MeteringStore;

fn metering(store: Arc<dyn MeteringStore>) -> Arc<dyn Metering> {
    Arc::new(StoredMetering::new(store))
}
```

Trusted fact producers must supply these inputs:

1. At workspace creation, call record_workspace_start.
   Save the initial zero-dollar Free PlanCost and metering period.
2. At first machine registration, call record_machine_registration.
   Save monthly MachineCost intervals when the customer supplies costs.
   Existing remove_machine calls preserve the registration's end time.
3. Supply assigned runners and taken-at times through JobStore.
   Update completed-at times when trusted GitHub facts arrive.
4. Save each ciproxy Boat runner's prices with save_boat_cost.
   Supply its actual billing start, end, start fee, and minute rate.
5. After billing confirmation, save PlanCost and Period histories.
   Close prior intervals with their actual exclusive ends.
6. Call Metering::refresh after facts change and before data purges.
   Refresh also updates running runtime through its supplied time.
   Refresh must precede organization, job, and workspace purges.
7. Retry a refresh that reports stale jobs or sandboxes.
   A competing refresh or deletion can change a cursor.
8. Call usage with the current confirmed period and saved plan.
   Call savings after refresh for current daily totals.
   Supply both dated USD price lists.

Unknown legacy registration and creation times remain unknown.
Runtime wiring, Stripe, machine forms, and catalog configuration belong to
their respective services.

## Development

Run commands from the repository root:

```sh
cargo build --package ciproxy-metering
cargo test --package ciproxy-metering
cargo fmt --all -- --check
cargo clippy --package ciproxy-metering --all-targets -- -D warnings
```

Unit tests use explicit times and hand-worked literal results.
Store-boundary unit tests use unimock.
PostgreSQL integration tests live in ciproxy-store-pg.
Those tests cover competing cursor writes, aggregate changes, and report
snapshots through the production adapter.

### Key Code

- [src/period.rs](src/period.rs) owns Free anchors and period changes.
- [src/usage.rs](src/usage.rs) computes counts, forecasts, and notices.
- [src/pricing.rs](src/pricing.rs) computes paid offers and capacities.
- [src/service.rs](src/service.rs) implements Metering over a dyn store.
- [src/report/runtime.rs](src/report/runtime.rs) computes daily corrections.
- [src/report/boat.rs](src/report/boat.rs) accounts for idle sandbox costs.
- [src/report/catalog.rs](src/report/catalog.rs) matches dated vendor prices.
- [src/report/costs.rs](src/report/costs.rs) computes monthly cost portions.
- [src/report/savings.rs](src/report/savings.rs) builds the savings report.

### Related Docs

- [Metering](../../docs/protocol/metering.md) defines the calculation rules.
- [Billing](../../docs/protocol/billing.md) defines prices and add-ons.
- [Security](../../docs/protocol/security.md#data-retention) defines retention.
- [Workspaces](../../docs/protocol/workspaces.md) defines shared counts.
- [Overflow](../../docs/protocol/overflow.md#spend-limit) defines Boat costs.
- [Architecture](../../docs/architecture.md) describes hosted boundaries.
- [Store interface](../ciproxy-store-interface/README.md) owns persistence.
