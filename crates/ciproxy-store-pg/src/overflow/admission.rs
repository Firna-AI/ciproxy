//! Saved settings and tenant authority checked under the reservation transaction.

use ciproxy_schema::settings::settings;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::record::StartRecord;
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::QueryResultExt;

use super::authority;

pub(super) fn check(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    record: &StartRecord,
    json: &str,
    budget: Option<&ReservationBudget>,
) -> Result<ReservationOutcome> {
    workspaces::table
        .find(workspace_id)
        .select(workspaces::id)
        .for_update()
        .first::<Uuid>(connection)
        .store()?;
    let (enabled, limit) = settings::table
        .find(workspace_id)
        .select((
            settings::overflow_enabled,
            settings::boat_spend_limit_usd_micros,
        ))
        .for_update()
        .first::<(bool, Option<i64>)>(connection)
        .store()?;
    if !enabled || !authority::permits_request(connection, workspace_id, record.key_id, json)? {
        return Ok(ReservationOutcome::NotPermitted);
    }
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
