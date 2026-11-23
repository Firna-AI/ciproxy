//! Scoped cost-history upserts and repeatable-read snapshots.

use chrono::{DateTime, Utc};
use ciproxy_schema::metering::{
    metering_machine_costs, metering_overflow_costs, metering_plan_costs,
};
use ciproxy_schema::runners::runners;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::metering::history::{
    CostHistory, MachineCost, OverflowCost, PlanCost,
};
use diesel::prelude::*;
use diesel::query_dsl::methods;
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};
use crate::overflow_provider::DbOverflowProvider;

use super::history_rows::{MachineCostRow, OverflowCostRow, PlanCostRow};

pub(super) fn save_machine(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    cost: &MachineCost,
) -> Result<()> {
    let values = (
        metering_machine_costs::id.eq(cost.id),
        metering_machine_costs::workspace_id.eq(workspace_id),
        metering_machine_costs::machine_id.eq(cost.machine_id),
        metering_machine_costs::starts_at.eq(cost.starts_at),
        metering_machine_costs::ends_at.eq(cost.ends_at),
        metering_machine_costs::monthly_usd.eq(cost.monthly_usd),
    );
    let count = methods::FilterDsl::filter(
        diesel::insert_into(metering_machine_costs::table)
            .values(values)
            .on_conflict(metering_machine_costs::id)
            .do_update()
            .set(values),
        metering_machine_costs::workspace_id.eq(workspace_id),
    )
    .execute(connection)
    .store()?;
    require_record(count)
}

pub(super) fn save_plan(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    cost: &PlanCost,
) -> Result<()> {
    let values = (
        metering_plan_costs::id.eq(cost.id),
        metering_plan_costs::workspace_id.eq(workspace_id),
        metering_plan_costs::starts_at.eq(cost.starts_at),
        metering_plan_costs::ends_at.eq(cost.ends_at),
        metering_plan_costs::monthly_usd.eq(cost.monthly_usd),
    );
    let count = methods::FilterDsl::filter(
        diesel::insert_into(metering_plan_costs::table)
            .values(values)
            .on_conflict(metering_plan_costs::id)
            .do_update()
            .set(values),
        metering_plan_costs::workspace_id.eq(workspace_id),
    )
    .execute(connection)
    .store()?;
    require_record(count)
}

pub(super) fn save_overflow(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    cost: &OverflowCost,
) -> Result<()> {
    let count = connection
        .transaction(|connection| {
            workspaces::table
                .find(workspace_id)
                .select(workspaces::id)
                .for_key_share()
                .first::<Uuid>(connection)?;
            runners::table
                .filter(runners::workspace_id.eq(workspace_id))
                .find(cost.runner_id)
                .filter(runners::sandbox_id.is_not_null())
                .select(runners::id)
                .for_key_share()
                .first::<Uuid>(connection)?;
            let values = (
                metering_overflow_costs::runner_id.eq(cost.runner_id),
                metering_overflow_costs::workspace_id.eq(workspace_id),
                metering_overflow_costs::provider.eq(DbOverflowProvider(cost.provider)),
                metering_overflow_costs::starts_at.eq(cost.starts_at),
                metering_overflow_costs::ends_at.eq(cost.ends_at),
                metering_overflow_costs::start_usd.eq(cost.start_usd),
                metering_overflow_costs::minute_usd.eq(cost.minute_usd),
            );
            methods::FilterDsl::filter(
                diesel::insert_into(metering_overflow_costs::table)
                    .values(values)
                    .on_conflict(metering_overflow_costs::runner_id)
                    .do_update()
                    .set(values),
                metering_overflow_costs::workspace_id.eq(workspace_id),
            )
            .execute(connection)
        })
        .store()?;
    require_record(count)
}

pub(super) fn load(connection: &mut PgConnection, workspace_id: Uuid) -> QueryResult<CostHistory> {
    let workspace_started_at = workspaces::table
        .find(workspace_id)
        .select(workspaces::metering_started_at)
        .first::<Option<DateTime<Utc>>>(connection)
        .optional()?
        .flatten();
    let machines = metering_machine_costs::table
        .filter(metering_machine_costs::workspace_id.eq(workspace_id))
        .order((
            metering_machine_costs::starts_at,
            metering_machine_costs::id,
        ))
        .select(MachineCostRow::as_select())
        .load(connection)?
        .into_iter()
        .map(MachineCostRow::into_record)
        .collect();
    let plans = metering_plan_costs::table
        .filter(metering_plan_costs::workspace_id.eq(workspace_id))
        .order((metering_plan_costs::starts_at, metering_plan_costs::id))
        .select(PlanCostRow::as_select())
        .load(connection)?
        .into_iter()
        .map(PlanCostRow::into_record)
        .collect();
    Ok(CostHistory {
        workspace_started_at,
        machines,
        plans,
    })
}

pub(super) fn load_overflow(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<Vec<OverflowCost>> {
    Ok(metering_overflow_costs::table
        .filter(metering_overflow_costs::workspace_id.eq(workspace_id))
        .order(metering_overflow_costs::runner_id)
        .select(OverflowCostRow::as_select())
        .load(connection)?
        .into_iter()
        .map(OverflowCostRow::into_record)
        .collect())
}
