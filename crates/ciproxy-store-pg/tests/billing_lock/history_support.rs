//! Literal billing selections and public reads of persisted plan costs.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::state::{AddOns, BillingState, SubscriptionState, Tier};
use ciproxy_store_interface::metering::history::PlanCost;
use ciproxy_store_interface::workspaces::Workspace;
use rust_decimal::Decimal;

use super::database::Database;

pub(super) fn paid(state: &BillingState, at: DateTime<Utc>) -> BillingState {
    let mut paid = state.clone();
    paid.tier = Tier::Team;
    paid.add_ons = AddOns {
        start_blocks: 2,
        machines: 1,
    };
    paid.status = SubscriptionState::Active;
    paid.customer_id = Some("cus_history".into());
    paid.subscription_id = Some("sub_history".into());
    paid.paid_period_start = Some(at);
    paid.paid_period_end = Some(at + chrono::Duration::days(30));
    paid
}

pub(super) fn plans(database: &Database, workspace: &Workspace) -> Vec<PlanCost> {
    let at = database
        .store
        .billing(workspace.id)
        .unwrap()
        .state
        .free_started_at;
    database
        .store
        .savings_snapshot(
            workspace.id,
            at.date_naive(),
            at.date_naive().succ_opt().unwrap(),
        )
        .unwrap()
        .costs
        .plans
}

pub(super) fn intervals(
    database: &Database,
    workspace: &Workspace,
) -> Vec<(DateTime<Utc>, Option<DateTime<Utc>>, Decimal)> {
    plans(database, workspace)
        .into_iter()
        .map(|cost| (cost.starts_at, cost.ends_at, cost.monthly_usd))
        .collect()
}
