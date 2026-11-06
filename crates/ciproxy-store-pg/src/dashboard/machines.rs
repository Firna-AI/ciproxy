//! Machine preference writes and exact monthly cost history.

use chrono::{DateTime, Utc};
use ciproxy_schema::machines::machines;
use ciproxy_schema::metering::metering_machine_costs;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::metering::history::MachineCost;
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;

#[derive(Queryable, Selectable)]
#[diesel(table_name = metering_machine_costs)]
struct CostRow {
    id: Uuid,
    machine_id: Uuid,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    monthly_usd: Decimal,
}

pub(super) fn costs(store: &PgStore, workspace: Uuid) -> Result<Vec<MachineCost>> {
    Ok(metering_machine_costs::table
        .filter(metering_machine_costs::workspace_id.eq(workspace))
        .order(metering_machine_costs::starts_at)
        .select(CostRow::as_select())
        .load::<CostRow>(&mut *store.connection()?)
        .store()?
        .into_iter()
        .map(|row| MachineCost {
            id: row.id,
            machine_id: row.machine_id,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            monthly_usd: row.monthly_usd,
        })
        .collect())
}

pub(super) fn save(store: &PgStore, workspace: Uuid, warm: i32, cost: &MachineCost) -> Result<()> {
    store
        .connection()?
        .transaction(|connection| {
            let machine = machines::table
                .find(cost.machine_id)
                .filter(machines::workspace_id.eq(workspace))
                .filter(machines::removed_at.is_null());
            machine
                .select(machines::id)
                .for_update()
                .first::<Uuid>(connection)?;
            diesel::update(machine)
                .set(machines::warm_runners.eq(warm))
                .execute(connection)?;
            diesel::update(
                metering_machine_costs::table
                    .filter(metering_machine_costs::workspace_id.eq(workspace))
                    .filter(metering_machine_costs::machine_id.eq(cost.machine_id))
                    .filter(metering_machine_costs::ends_at.is_null()),
            )
            .set(metering_machine_costs::ends_at.eq(cost.starts_at))
            .execute(connection)?;
            diesel::insert_into(metering_machine_costs::table)
                .values((
                    metering_machine_costs::id.eq(cost.id),
                    metering_machine_costs::workspace_id.eq(workspace),
                    metering_machine_costs::machine_id.eq(cost.machine_id),
                    metering_machine_costs::starts_at.eq(cost.starts_at),
                    metering_machine_costs::monthly_usd.eq(cost.monthly_usd),
                ))
                .execute(connection)?;
            QueryResult::Ok(())
        })
        .store()
}
