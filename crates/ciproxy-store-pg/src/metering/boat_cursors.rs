//! Atomic per-runner Boat cost cursors with anonymous retained cost totals.

use chrono::{DateTime, NaiveDate, Utc};
use ciproxy_schema::metering::{metering_boat_cursors, metering_daily_boat_costs};
use ciproxy_schema::runners::runners;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::metering::history::BoatCost;
use ciproxy_store_interface::metering::totals::{AccountedBoat, DailyBoatCost};
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = metering_boat_cursors, treat_none_as_null = true)]
struct CursorRow {
    runner_id: Uuid,
    workspace_id: Uuid,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    start_usd: Decimal,
    minute_usd: Decimal,
    observed_at: DateTime<Utc>,
}

impl CursorRow {
    fn from_record(workspace_id: Uuid, record: &AccountedBoat) -> Self {
        Self {
            workspace_id,
            runner_id: record.cost.runner_id,
            starts_at: record.cost.starts_at,
            ends_at: record.cost.ends_at,
            start_usd: record.cost.start_usd,
            minute_usd: record.cost.minute_usd,
            observed_at: record.observed_at,
        }
    }

    fn into_record(self) -> AccountedBoat {
        AccountedBoat {
            observed_at: self.observed_at,
            cost: BoatCost {
                runner_id: self.runner_id,
                starts_at: self.starts_at,
                ends_at: self.ends_at,
                start_usd: self.start_usd,
                minute_usd: self.minute_usd,
            },
        }
    }
}

pub(super) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<Vec<AccountedBoat>> {
    Ok(metering_boat_cursors::table
        .filter(metering_boat_cursors::workspace_id.eq(workspace_id))
        .order(metering_boat_cursors::runner_id)
        .select(CursorRow::as_select())
        .load::<CursorRow>(connection)?
        .into_iter()
        .map(CursorRow::into_record)
        .collect())
}

pub(super) fn apply(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    previous: Option<&AccountedBoat>,
    current: &AccountedBoat,
    deltas: &[DailyBoatCost],
) -> QueryResult<bool> {
    let workspace = workspaces::table
        .find(workspace_id)
        .select(workspaces::metering_retained_from)
        .for_update()
        .first::<Option<NaiveDate>>(connection)
        .optional()?;
    let Some(retained_from) = workspace else {
        return Ok(false);
    };
    let runner = runners::table
        .filter(runners::workspace_id.eq(workspace_id))
        .find(current.cost.runner_id)
        .filter(runners::sandbox_id.is_not_null())
        .select(runners::id)
        .for_update()
        .first::<Uuid>(connection)
        .optional()?;
    if runner.is_none() {
        return Ok(false);
    }
    let row = metering_boat_cursors::table
        .filter(metering_boat_cursors::workspace_id.eq(workspace_id))
        .find(current.cost.runner_id)
        .select(CursorRow::as_select())
        .first(connection)
        .optional()?;
    if row.map(CursorRow::into_record).as_ref() != previous {
        return Ok(false);
    }
    for delta in deltas
        .iter()
        .filter(|delta| retained_from.is_none_or(|from| delta.day >= from))
    {
        diesel::insert_into(metering_daily_boat_costs::table)
            .values((
                metering_daily_boat_costs::workspace_id.eq(workspace_id),
                metering_daily_boat_costs::day.eq(delta.day),
                metering_daily_boat_costs::usd.eq(delta.usd),
            ))
            .on_conflict((
                metering_daily_boat_costs::workspace_id,
                metering_daily_boat_costs::day,
            ))
            .do_update()
            .set(metering_daily_boat_costs::usd.eq(metering_daily_boat_costs::usd + delta.usd))
            .execute(connection)?;
    }
    let row = CursorRow::from_record(workspace_id, current);
    diesel::insert_into(metering_boat_cursors::table)
        .values(&row)
        .on_conflict(metering_boat_cursors::runner_id)
        .do_update()
        .set(&row)
        .execute(connection)?;
    Ok(true)
}
