//! Identity-free workspace totals, signed upserts, and retention queries.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use ciproxy_schema::metering::{
    metering_daily_boat_costs, metering_daily_sizes, metering_start_totals,
};
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::metering::totals::{
    DailyDelta, DailyTotal, LocationTotals, SizeTotal, StartTotal,
};
use ciproxy_store_interface::vm_size::VmSize;
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

pub(super) fn apply_usage(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    deltas: &[DailyDelta],
    starts: &[StartTotal],
    retained_from: Option<NaiveDate>,
) -> QueryResult<()> {
    for delta in deltas
        .iter()
        .filter(|delta| retained_from.is_none_or(|from| delta.day >= from))
    {
        diesel::insert_into(metering_daily_sizes::table)
            .values((
                metering_daily_sizes::workspace_id.eq(workspace_id),
                metering_daily_sizes::day.eq(delta.day),
                metering_daily_sizes::vcpus.eq(delta.size.vcpus),
                metering_daily_sizes::memory_gb.eq(delta.size.memory_gb),
                metering_daily_sizes::disk_gb.eq(delta.size.disk_gb),
                metering_daily_sizes::machine_starts.eq(delta.machine_starts),
                metering_daily_sizes::machine_minutes.eq(delta.machine_minutes),
                metering_daily_sizes::boat_starts.eq(delta.boat_starts),
                metering_daily_sizes::boat_minutes.eq(delta.boat_minutes),
            ))
            .on_conflict((
                metering_daily_sizes::workspace_id,
                metering_daily_sizes::day,
                metering_daily_sizes::vcpus,
                metering_daily_sizes::memory_gb,
                metering_daily_sizes::disk_gb,
            ))
            .do_update()
            .set((
                metering_daily_sizes::machine_starts
                    .eq(metering_daily_sizes::machine_starts + delta.machine_starts),
                metering_daily_sizes::machine_minutes
                    .eq(metering_daily_sizes::machine_minutes + delta.machine_minutes),
                metering_daily_sizes::boat_starts
                    .eq(metering_daily_sizes::boat_starts + delta.boat_starts),
                metering_daily_sizes::boat_minutes
                    .eq(metering_daily_sizes::boat_minutes + delta.boat_minutes),
            ))
            .execute(connection)?;
    }
    for start in starts
        .iter()
        .filter(|start| retained_from.is_none_or(|from| start.taken_at.date_naive() >= from))
    {
        diesel::insert_into(metering_start_totals::table)
            .values((
                metering_start_totals::workspace_id.eq(workspace_id),
                metering_start_totals::taken_at.eq(start.taken_at),
                metering_start_totals::starts.eq(start.starts),
            ))
            .on_conflict((
                metering_start_totals::workspace_id,
                metering_start_totals::taken_at,
            ))
            .do_update()
            .set(metering_start_totals::starts.eq(metering_start_totals::starts + start.starts))
            .execute(connection)?;
    }
    Ok(())
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = metering_daily_sizes)]
struct DailyRow {
    day: NaiveDate,
    vcpus: i32,
    memory_gb: i32,
    disk_gb: i32,
    machine_starts: i64,
    machine_minutes: Decimal,
    boat_starts: i64,
    boat_minutes: Decimal,
}

pub(super) fn load(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    from: NaiveDate,
    until: NaiveDate,
) -> QueryResult<Vec<DailyTotal>> {
    let rows = metering_daily_sizes::table
        .filter(metering_daily_sizes::workspace_id.eq(workspace_id))
        .filter(metering_daily_sizes::day.ge(from))
        .filter(metering_daily_sizes::day.lt(until))
        .order((
            metering_daily_sizes::day,
            metering_daily_sizes::vcpus,
            metering_daily_sizes::memory_gb,
            metering_daily_sizes::disk_gb,
        ))
        .select(DailyRow::as_select())
        .load::<DailyRow>(connection)?;
    let mut days: BTreeMap<_, DailyTotal> = BTreeMap::new();
    for row in rows {
        let total = days.entry(row.day).or_insert(DailyTotal {
            day: row.day,
            sizes: Vec::new(),
            boat_usd: Decimal::ZERO,
        });
        total.sizes.push(SizeTotal {
            size: VmSize {
                vcpus: row.vcpus,
                memory_gb: row.memory_gb,
                disk_gb: row.disk_gb,
            },
            machines: LocationTotals {
                starts: row.machine_starts as u64,
                minutes: row.machine_minutes,
            },
            boat: LocationTotals {
                starts: row.boat_starts as u64,
                minutes: row.boat_minutes,
            },
        });
    }
    let costs = metering_daily_boat_costs::table
        .filter(metering_daily_boat_costs::workspace_id.eq(workspace_id))
        .filter(metering_daily_boat_costs::day.ge(from))
        .filter(metering_daily_boat_costs::day.lt(until))
        .select((
            metering_daily_boat_costs::day,
            metering_daily_boat_costs::usd,
        ))
        .load::<(NaiveDate, Decimal)>(connection)?;
    for (day, usd) in costs {
        days.entry(day)
            .or_insert(DailyTotal {
                day,
                sizes: Vec::new(),
                boat_usd: Decimal::ZERO,
            })
            .boat_usd = usd;
    }
    Ok(days.into_values().collect())
}

pub(super) fn purge(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    before: NaiveDate,
) -> QueryResult<usize> {
    let retained = workspaces::table
        .find(workspace_id)
        .select(workspaces::metering_retained_from)
        .for_update()
        .first::<Option<NaiveDate>>(connection)
        .optional()?;
    let Some(retained) = retained else {
        return Ok(0);
    };
    let before = retained.map_or(before, |retained| before.max(retained));
    diesel::update(workspaces::table.find(workspace_id))
        .set(workspaces::metering_retained_from.eq(before))
        .execute(connection)?;
    let sizes = diesel::delete(
        metering_daily_sizes::table
            .filter(metering_daily_sizes::workspace_id.eq(workspace_id))
            .filter(metering_daily_sizes::day.lt(before)),
    )
    .execute(connection)?;
    let costs = diesel::delete(
        metering_daily_boat_costs::table
            .filter(metering_daily_boat_costs::workspace_id.eq(workspace_id))
            .filter(metering_daily_boat_costs::day.lt(before)),
    )
    .execute(connection)?;
    let at: DateTime<Utc> = before.and_time(NaiveTime::MIN).and_utc();
    let starts = diesel::delete(
        metering_start_totals::table
            .filter(metering_start_totals::workspace_id.eq(workspace_id))
            .filter(metering_start_totals::taken_at.lt(at)),
    )
    .execute(connection)?;
    Ok(sizes + costs + starts)
}
