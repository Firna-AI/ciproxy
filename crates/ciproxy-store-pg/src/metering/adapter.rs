//! Store-interface implementation for metering persistence.

use chrono::{DateTime, NaiveDate, Utc};
use ciproxy_schema::machines::machines;
use ciproxy_schema::metering::metering_periods;
use ciproxy_schema::workspaces::workspaces;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::metering::facts::WorkspaceUsage;
use ciproxy_store_interface::metering::history::{
    BoatAccounting, BoatCost, MachineCost, Period, PlanCost,
};
use ciproxy_store_interface::metering::report::SavingsSnapshot;
use ciproxy_store_interface::metering::store::MeteringStore;
use ciproxy_store_interface::metering::totals::{
    AccountedBoat, AccountedJob, DailyBoatCost, DailyDelta, StartTotal,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::read_snapshot;
use crate::overflow::accounting;

use super::{aggregates, boat_cursors, facts, history, job_cursors, snapshot};

impl MeteringStore for PgStore {
    fn workspace_usage(&self, workspace_id: Uuid) -> Result<WorkspaceUsage> {
        snapshot::transaction(&mut *self.connection()?)
            .run(|connection| facts::load(connection, workspace_id))
            .store()
    }

    fn record_workspace_start(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<()> {
        diesel::update(
            workspaces::table
                .find(workspace_id)
                .filter(workspaces::metering_started_at.is_null()),
        )
        .set(workspaces::metering_started_at.eq(at))
        .execute(&mut *self.connection()?)
        .store()?;
        Ok(())
    }

    fn record_machine_registration(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()> {
        diesel::update(
            machines::table
                .filter(machines::workspace_id.eq(workspace_id))
                .find(machine_id)
                .filter(machines::registered_at.is_null()),
        )
        .set(machines::registered_at.eq(at))
        .execute(&mut *self.connection()?)
        .store()?;
        Ok(())
    }

    fn save_machine_cost(&self, workspace_id: Uuid, cost: &MachineCost) -> Result<()> {
        history::save_machine(&mut *self.connection()?, workspace_id, cost)
    }

    fn save_plan_cost(&self, workspace_id: Uuid, cost: &PlanCost) -> Result<()> {
        history::save_plan(&mut *self.connection()?, workspace_id, cost)
    }

    fn save_boat_cost(&self, workspace_id: Uuid, cost: &BoatCost) -> Result<()> {
        history::save_boat(&mut *self.connection()?, workspace_id, cost)
    }

    fn savings_snapshot(
        &self,
        workspace_id: Uuid,
        from: NaiveDate,
        until: NaiveDate,
    ) -> Result<SavingsSnapshot> {
        read_snapshot(&mut *self.connection()?, |connection| {
            accounting::require_complete(connection, workspace_id, Some((from, until)))?;
            Ok(SavingsSnapshot {
                machines: facts::registrations(connection, workspace_id).store()?,
                days: aggregates::load(connection, workspace_id, from, until).store()?,
                costs: history::load(connection, workspace_id).store()?,
            })
        })
    }

    fn save_metering_period(&self, workspace_id: Uuid, period: &Period) -> Result<()> {
        let count = diesel::insert_into(metering_periods::table)
            .values((
                metering_periods::workspace_id.eq(workspace_id),
                metering_periods::starts_at.eq(period.starts_at),
                metering_periods::ends_at.eq(period.ends_at),
            ))
            .on_conflict((metering_periods::workspace_id, metering_periods::starts_at))
            .do_update()
            .set(metering_periods::ends_at.eq(period.ends_at))
            .execute(&mut *self.connection()?)
            .store()?;
        require_record(count)
    }

    fn metering_periods(&self, workspace_id: Uuid) -> Result<Vec<Period>> {
        let rows = metering_periods::table
            .filter(metering_periods::workspace_id.eq(workspace_id))
            .order(metering_periods::starts_at)
            .select((metering_periods::starts_at, metering_periods::ends_at))
            .load::<(DateTime<Utc>, DateTime<Utc>)>(&mut *self.connection()?)
            .store()?;
        Ok(rows
            .into_iter()
            .map(|(starts_at, ends_at)| Period { starts_at, ends_at })
            .collect())
    }

    fn apply_accounting(
        &self,
        workspace_id: Uuid,
        previous: Option<&AccountedJob>,
        current: &AccountedJob,
        deltas: &[DailyDelta],
        starts: &[StartTotal],
    ) -> Result<bool> {
        self.connection()?
            .transaction(|connection| {
                job_cursors::apply(connection, workspace_id, previous, current, deltas, starts)
            })
            .store()
    }

    fn boat_accounting(&self, workspace_id: Uuid) -> Result<BoatAccounting> {
        read_snapshot(&mut *self.connection()?, |connection| {
            accounting::require_complete(connection, workspace_id, None)?;
            Ok(BoatAccounting {
                costs: history::load_boat(connection, workspace_id).store()?,
                accounted: boat_cursors::load(connection, workspace_id).store()?,
            })
        })
    }

    fn apply_boat_accounting(
        &self,
        workspace_id: Uuid,
        previous: Option<&AccountedBoat>,
        current: &AccountedBoat,
        deltas: &[DailyBoatCost],
    ) -> Result<bool> {
        self.connection()?
            .transaction(|connection| {
                boat_cursors::apply(connection, workspace_id, previous, current, deltas)
            })
            .store()
    }

    fn purge_daily_totals(&self, workspace_id: Uuid, before: NaiveDate) -> Result<usize> {
        self.connection()?
            .transaction(|connection| aggregates::purge(connection, workspace_id, before))
            .store()
    }
}
