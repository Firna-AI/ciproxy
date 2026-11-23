//! Saved settings and tenant authority checked under the reservation transaction.

use ciproxy_schema::settings::settings;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::record::StartRecord;
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::QueryResultExt;
use crate::overflow_provider_settings;
use crate::workspaces::lock_workspace;

use super::authority;

pub(super) fn check(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
    json: &str,
    budget: Option<&ReservationBudget>,
) -> Result<ReservationOutcome> {
    lock_workspace(connection, workspace_id).store()?;
    let enabled = settings::table
        .find(workspace_id)
        .select(settings::overflow_enabled)
        .for_update()
        .first::<bool>(connection)
        .store()?;
    if !enabled || !authority::permits_request(connection, workspace_id, record.key_id, json)? {
        return Ok(ReservationOutcome::NotPermitted);
    }
    let limit = overflow_provider_settings::load(connection, workspace_id, record.provider)
        .store()?
        .and_then(|settings| settings.spend_limit_usd_micros);
    if budget.is_some_and(|budget| {
        limit.is_none_or(|limit| {
            let limit = Decimal::new(limit, 6);
            limit <= Decimal::ZERO
                || budget.current_month_usd > limit
                || budget
                    .next_month_usd
                    .is_some_and(|required| required > limit)
        })
    }) {
        return Ok(ReservationOutcome::SpendLimit);
    }
    Ok(ReservationOutcome::Reserved)
}
