//! Scoped cost-history upserts and repeatable-read snapshots.

use chrono::{DateTime, Utc};
use ciproxy_schema::metering::{metering_boat_costs, metering_machine_costs, metering_plan_costs};
use ciproxy_schema::runners::runners;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::metering::history::{BoatCost, CostHistory, MachineCost, PlanCost};
use diesel::prelude::*;
use diesel::query_dsl::methods;
use uuid::Uuid;

use crate::error::{QueryResultExt, require_record};

use super::history_rows::{BoatCostRow, MachineCostRow, PlanCostRow};

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

pub(super) fn save_boat(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    cost: &BoatCost,
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
                metering_boat_costs::runner_id.eq(cost.runner_id),
                metering_boat_costs::workspace_id.eq(workspace_id),
                metering_boat_costs::starts_at.eq(cost.starts_at),
                metering_boat_costs::ends_at.eq(cost.ends_at),
                metering_boat_costs::start_usd.eq(cost.start_usd),
                metering_boat_costs::minute_usd.eq(cost.minute_usd),
            );
            methods::FilterDsl::filter(
                diesel::insert_into(metering_boat_costs::table)
                    .values(values)
                    .on_conflict(metering_boat_costs::runner_id)
                    .do_update()
                    .set(values),
                metering_boat_costs::workspace_id.eq(workspace_id),
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

pub(super) fn load_boat(
    connection: &mut PgConnection,
    workspace_id: Uuid,
) -> QueryResult<Vec<BoatCost>> {
    Ok(metering_boat_costs::table
        .filter(metering_boat_costs::workspace_id.eq(workspace_id))
        .order(metering_boat_costs::runner_id)
        .select(BoatCostRow::as_select())
        .load(connection)?
        .into_iter()
        .map(BoatCostRow::into_record)
        .collect())
}
