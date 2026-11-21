//! Plan-price boundaries written inside the confirmed billing transaction.

use chrono::{DateTime, Utc};
use ciproxy_schema::metering::metering_plan_costs;
use ciproxy_store_interface::billing::state::{BillingState, Tier};
use ciproxy_store_interface::error::Result;
use diesel::dsl::{max, sql};
use diesel::prelude::*;
use diesel::sql_types;
use rust_decimal::Decimal;

use crate::error::QueryResultExt;

pub(super) fn save(
    connection: &mut PgConnection,
    previous: &BillingState,
    current: &BillingState,
    at: DateTime<Utc>,
) -> Result<()> {
    let monthly_cents = current.selection().monthly_cents();
    if previous.selection().monthly_cents() == monthly_cents {
        return Ok(());
    }
    let effective_at = if current.tier == Tier::Free {
        current.free_started_at
    } else if previous.tier == Tier::Free {
        current.paid_period_start.unwrap_or(at)
    } else {
        at
    };
    let open_start = metering_plan_costs::table
        .filter(metering_plan_costs::workspace_id.eq(current.workspace_id))
        .filter(metering_plan_costs::ends_at.is_null())
        .select(max(metering_plan_costs::starts_at))
        .first::<Option<DateTime<Utc>>>(connection)
        .store()?;
    let effective_at = open_start.map_or(effective_at, |start| effective_at.max(start));
    diesel::update(
        metering_plan_costs::table
            .filter(metering_plan_costs::workspace_id.eq(current.workspace_id))
            .filter(metering_plan_costs::ends_at.is_null()),
    )
    .set(metering_plan_costs::ends_at.eq(effective_at))
    .execute(connection)
    .store()?;
    diesel::insert_into(metering_plan_costs::table)
        .values((
            metering_plan_costs::id.eq(sql::<sql_types::Uuid>("gen_random_uuid()")),
            metering_plan_costs::workspace_id.eq(current.workspace_id),
            metering_plan_costs::starts_at.eq(effective_at),
            metering_plan_costs::monthly_usd.eq(Decimal::from(monthly_cents) / Decimal::from(100)),
        ))
        .execute(connection)
        .store()?;
    Ok(())
}
