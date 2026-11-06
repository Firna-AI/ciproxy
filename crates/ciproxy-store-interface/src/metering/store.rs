//! Object-safe workspace-scoped metering persistence.

use chrono::{DateTime, NaiveDate, Utc};
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::metering::facts::WorkspaceUsage;
use crate::metering::history::{BoatAccounting, BoatCost, MachineCost, Period, PlanCost};
use crate::metering::report::SavingsSnapshot;
use crate::metering::totals::{AccountedBoat, AccountedJob, DailyBoatCost, DailyDelta, StartTotal};

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
    /// Record the trusted workspace creation time for zero-cost prehistory.
    fn record_workspace_start(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Record a trusted first registration time without changing machine state.
    fn record_machine_registration(
        &self,
        workspace_id: Uuid,
        machine_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Store one customer's machine-price interval by its stable identifier.
    fn save_machine_cost(&self, workspace_id: Uuid, cost: &MachineCost) -> Result<()>;
    /// Store one confirmed monthly tier and add-on price interval.
    fn save_plan_cost(&self, workspace_id: Uuid, cost: &PlanCost) -> Result<()>;
    /// Store one ciproxy Boat runner's start and runtime price snapshot.
    fn save_boat_cost(&self, workspace_id: Uuid, cost: &BoatCost) -> Result<()>;
    /// Read report registrations, range totals, and monthly costs in one snapshot.
    /// Return AccountingIncomplete when required overflow cost facts are missing.
    fn savings_snapshot(
        &self,
        workspace_id: Uuid,
        from: NaiveDate,
        until: NaiveDate,
    ) -> Result<SavingsSnapshot>;
    /// Save or close a billing period using its original start as the key.
    fn save_metering_period(&self, workspace_id: Uuid, period: &Period) -> Result<()>;
    /// Read stored Free and Stripe period boundaries in start order.
    fn metering_periods(&self, workspace_id: Uuid) -> Result<Vec<Period>>;
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
    /// Read Boat source facts and cost cursors in one coherent snapshot.
    /// Incomplete deliveries contribute only their known bounded billing intervals.
    fn boat_accounting(&self, workspace_id: Uuid) -> Result<BoatAccounting>;
    /// Apply Boat cost corrections and a cursor in one transaction.
    ///
    /// Return false when another refresh changed the previous cursor.
    /// Ignore corrections older than a previously applied retention cutoff.
    fn apply_boat_accounting(
        &self,
        workspace_id: Uuid,
        previous: Option<&AccountedBoat>,
        current: &AccountedBoat,
        deltas: &[DailyBoatCost],
    ) -> Result<bool>;
    /// Delete daily totals before the first retained UTC day.
    /// Keep this cutoff monotonic so stale refreshes cannot restore old dates.
    fn purge_daily_totals(&self, workspace_id: Uuid, before: NaiveDate) -> Result<usize>;
}
