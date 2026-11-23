//! Store-interface implementation for metering persistence.

use chrono::NaiveDate;
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::metering::facts::WorkspaceUsage;
use ciproxy_store_interface::metering::history::{
    MachineCost, OverflowAccounting, OverflowCost, PlanCost,
};
use ciproxy_store_interface::metering::report::SavingsSnapshot;
use ciproxy_store_interface::metering::store::MeteringStore;
use ciproxy_store_interface::metering::totals::{
    AccountedJob, AccountedOverflow, DailyDelta, DailyOverflowCost, StartTotal,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::QueryResultExt;
use crate::gateway_transaction::read_snapshot;
use crate::overflow::accounting;

use super::{aggregates, facts, history, job_cursors, overflow_cursors, snapshot};

impl MeteringStore for PgStore {
    fn retire_accounting(
        &self,
        workspace_id: Uuid,
        previous: &AccountedJob,
        deltas: &[DailyDelta],
        starts: &[StartTotal],
    ) -> Result<bool> {
        self.connection()?
            .transaction(|connection| {
                job_cursors::retire(connection, workspace_id, previous, deltas, starts)
            })
            .store()
    }

    fn workspace_usage(&self, workspace_id: Uuid) -> Result<WorkspaceUsage> {
        snapshot::transaction(&mut *self.connection()?)
            .run(|connection| facts::load(connection, workspace_id))
            .store()
    }

    fn save_machine_cost(&self, workspace_id: Uuid, cost: &MachineCost) -> Result<()> {
        history::save_machine(&mut *self.connection()?, workspace_id, cost)
    }

    fn save_plan_cost(&self, workspace_id: Uuid, cost: &PlanCost) -> Result<()> {
        history::save_plan(&mut *self.connection()?, workspace_id, cost)
    }

    fn save_overflow_cost(&self, workspace_id: Uuid, cost: &OverflowCost) -> Result<()> {
        history::save_overflow(&mut *self.connection()?, workspace_id, cost)
    }

    fn savings_snapshot(
        &self,
        workspace_id: Uuid,
        from: NaiveDate,
        until: NaiveDate,
    ) -> Result<SavingsSnapshot> {
        read_snapshot(&mut *self.connection()?, |connection| {
            accounting::require_complete(connection, workspace_id, (from, until))?;
            Ok(SavingsSnapshot {
                machines: facts::registrations(connection, workspace_id).store()?,
                days: aggregates::load(connection, workspace_id, from, until).store()?,
                costs: history::load(connection, workspace_id).store()?,
            })
        })
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

    fn overflow_accounting(&self, workspace_id: Uuid) -> Result<OverflowAccounting> {
        read_snapshot(&mut *self.connection()?, |connection| {
            Ok(OverflowAccounting {
                costs: history::load_overflow(connection, workspace_id).store()?,
                accounted: overflow_cursors::load(connection, workspace_id).store()?,
            })
        })
    }

    fn apply_overflow_accounting(
        &self,
        workspace_id: Uuid,
        previous: Option<&AccountedOverflow>,
        current: &AccountedOverflow,
        deltas: &[DailyOverflowCost],
    ) -> Result<bool> {
        self.connection()?
            .transaction(|connection| {
                overflow_cursors::apply(connection, workspace_id, previous, current, deltas)
            })
            .store()
    }

    fn purge_daily_totals(&self, workspace_id: Uuid, before: NaiveDate) -> Result<usize> {
        self.connection()?
            .transaction(|connection| aggregates::purge(connection, workspace_id, before))
            .store()
    }
}
