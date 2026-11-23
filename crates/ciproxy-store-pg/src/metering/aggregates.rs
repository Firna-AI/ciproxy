//! Identity-free workspace totals, signed upserts, and retention queries.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, NaiveTime, SecondsFormat, Utc};
use ciproxy_protocol::resources::VmSize;
use ciproxy_schema::metering::{
    metering_daily_overflow_costs, metering_daily_sizes, metering_start_totals,
};
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::metering::totals::{
    DailyDelta, DailyTotal, LocationTotals, SizeTotal, StartTotal,
};
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::exact_size::convert;
use crate::overflow_provider::DbOverflowProvider;

use super::place::DbJobPlace;

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
                metering_daily_sizes::vcpus.eq(convert::<i32, _>(delta.size.vcpu)?),
                metering_daily_sizes::memory_bytes.eq(convert::<i64, _>(delta.size.memory_bytes)?),
                metering_daily_sizes::disk_bytes.eq(convert::<i64, _>(delta.size.disk_bytes)?),
                metering_daily_sizes::place.eq(DbJobPlace(delta.place)),
                metering_daily_sizes::starts.eq(delta.starts),
                metering_daily_sizes::minutes.eq(delta.minutes),
            ))
            .on_conflict((
                metering_daily_sizes::workspace_id,
                metering_daily_sizes::day,
                metering_daily_sizes::vcpus,
                metering_daily_sizes::memory_bytes,
                metering_daily_sizes::disk_bytes,
                metering_daily_sizes::place,
            ))
            .do_update()
            .set((
                metering_daily_sizes::starts.eq(metering_daily_sizes::starts + delta.starts),
                metering_daily_sizes::minutes.eq(metering_daily_sizes::minutes + delta.minutes),
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
                metering_start_totals::taken_at
                    .eq(start.taken_at.to_rfc3339_opts(SecondsFormat::Nanos, true)),
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
    memory_bytes: i64,
    disk_bytes: i64,
    place: DbJobPlace,
    starts: i64,
    minutes: Decimal,
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
            metering_daily_sizes::memory_bytes,
            metering_daily_sizes::disk_bytes,
        ))
        .select(DailyRow::as_select())
        .load::<DailyRow>(connection)?;
    let mut days: BTreeMap<_, DailyTotal> = BTreeMap::new();
    for row in rows {
        let total = days
            .entry(row.day)
            .or_insert_with(|| DailyTotal::new(row.day));
        let size = VmSize {
            vcpu: convert(row.vcpus)?,
            memory_bytes: convert(row.memory_bytes)?,
            disk_bytes: convert(row.disk_bytes)?,
        };
        if total
            .sizes
            .last()
            .is_none_or(|previous| previous.size != size)
        {
            total.sizes.push(SizeTotal::new(size));
        }
        if let Some(total) = total.sizes.last_mut() {
            *total.usage_mut(row.place.0) = LocationTotals {
                starts: row.starts as u64,
                minutes: row.minutes,
            };
        }
    }
    let costs = metering_daily_overflow_costs::table
        .filter(metering_daily_overflow_costs::workspace_id.eq(workspace_id))
        .filter(metering_daily_overflow_costs::day.ge(from))
        .filter(metering_daily_overflow_costs::day.lt(until))
        .select((
            metering_daily_overflow_costs::day,
            metering_daily_overflow_costs::provider,
            metering_daily_overflow_costs::usd,
        ))
        .load::<(NaiveDate, DbOverflowProvider, Decimal)>(connection)?;
    for (day, provider, usd) in costs {
        days.entry(day)
            .or_insert_with(|| DailyTotal::new(day))
            .overflow_usd
            .insert(provider.0, usd);
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
        metering_daily_overflow_costs::table
            .filter(metering_daily_overflow_costs::workspace_id.eq(workspace_id))
            .filter(metering_daily_overflow_costs::day.lt(before)),
    )
    .execute(connection)?;
    let at: DateTime<Utc> = before.and_time(NaiveTime::MIN).and_utc();
    let starts = diesel::delete(
        metering_start_totals::table
            .filter(metering_start_totals::workspace_id.eq(workspace_id))
            .filter(
                metering_start_totals::taken_at.lt(at.to_rfc3339_opts(SecondsFormat::Nanos, true)),
            ),
    )
    .execute(connection)?;
    Ok(sizes + costs + starts)
}
