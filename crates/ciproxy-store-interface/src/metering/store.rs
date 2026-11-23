//! Object-safe workspace-scoped metering persistence.

use chrono::NaiveDate;
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::metering::facts::WorkspaceUsage;
use crate::metering::history::{MachineCost, OverflowAccounting, OverflowCost, PlanCost};
use crate::metering::report::SavingsSnapshot;
use crate::metering::totals::{
    AccountedJob, AccountedOverflow, DailyDelta, DailyOverflowCost, StartTotal,
};

/// Queries and writes used by metering and its trusted fact producers.
#[unimock(api = MeteringStoreMock)]
pub trait MeteringStore: Send + Sync {
    /// Remove an explicitly aliased scale cursor and apply its negative corrections.
    fn retire_accounting(
        &self,
        workspace_id: Uuid,
        previous: &AccountedJob,
        deltas: &[DailyDelta],
        starts: &[StartTotal],
    ) -> Result<bool>;
    /// Read all retained started jobs and registrations in one workspace.
    fn workspace_usage(&self, workspace_id: Uuid) -> Result<WorkspaceUsage>;
    /// Store one customer's machine-price interval by its stable identifier.
    fn save_machine_cost(&self, workspace_id: Uuid, cost: &MachineCost) -> Result<()>;
    /// Store one confirmed monthly tier and add-on price interval.
    fn save_plan_cost(&self, workspace_id: Uuid, cost: &PlanCost) -> Result<()>;
    /// Store one overflow runner's start and runtime price snapshot.
    fn save_overflow_cost(&self, workspace_id: Uuid, cost: &OverflowCost) -> Result<()>;
    /// Read report registrations, range totals, and monthly costs in one snapshot.
    /// Return AccountingIncomplete when required overflow cost facts are missing.
    fn savings_snapshot(
        &self,
        workspace_id: Uuid,
        from: NaiveDate,
        until: NaiveDate,
    ) -> Result<SavingsSnapshot>;
    /// Apply supplied aggregate corrections and a cursor in one transaction.
    ///
    /// Return false when another refresh changed the previous cursor.
    /// A job deleted before this write also returns false.
    /// Ignore corrections older than a previously applied retention cutoff.
    fn apply_accounting(
        &self,
        workspace_id: Uuid,
        previous: Option<&AccountedJob>,
        current: &AccountedJob,
        deltas: &[DailyDelta],
        starts: &[StartTotal],
    ) -> Result<bool>;
    /// Read overflow source facts and cost cursors in one coherent snapshot.
    /// Incomplete deliveries contribute only their known bounded billing intervals.
    fn overflow_accounting(&self, workspace_id: Uuid) -> Result<OverflowAccounting>;
    /// Apply overflow cost corrections and a cursor in one transaction.
    ///
    /// Return false when another refresh changed the previous cursor.
    /// Ignore corrections older than a previously applied retention cutoff.
    fn apply_overflow_accounting(
        &self,
        workspace_id: Uuid,
        previous: Option<&AccountedOverflow>,
        current: &AccountedOverflow,
        deltas: &[DailyOverflowCost],
    ) -> Result<bool>;
    /// Delete daily totals before the first retained UTC day.
    /// Keep this cutoff monotonic so stale refreshes cannot restore old dates.
    fn purge_daily_totals(&self, workspace_id: Uuid, before: NaiveDate) -> Result<usize>;
}
