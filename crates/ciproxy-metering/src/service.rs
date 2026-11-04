//! Object-safe orchestration for incremental daily accounting and reports.

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::metering::history::Period;
use ciproxy_store_interface::metering::store::MeteringStore;
use ciproxy_store_interface::metering::totals::{AccountedBoat, AccountedJob};
use uuid::Uuid;

use crate::pricing::Plan;
use crate::report::boat::boat_cost_deltas;
use crate::report::catalog::PriceList;
use crate::report::costs::MonthlyDivisor;
use crate::report::range::{DateRange, retention_start};
use crate::report::runtime::{accounting_deltas, start_deltas};
use crate::report::savings::{ReportInput, SavingsReport, savings_report};
use crate::usage::{PaymentNotice, Usage, workspace_usage};

/// The result of one refresh without hiding concurrent cursor changes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Refresh {
    /// Jobs whose cursors and aggregate corrections were stored.
    pub updated_jobs: usize,
    /// Jobs changed or deleted concurrently, which callers can refresh again.
    pub stale_jobs: usize,
    /// Boat sandboxes whose cost cursors and corrections were stored.
    pub updated_sandboxes: usize,
    /// Boat cost cursors changed or deleted concurrently.
    pub stale_sandboxes: usize,
    /// Expired anonymous daily rows deleted by this refresh.
    pub purged_rows: usize,
}

/// The metering behavior consumed by runtime services through dyn.
pub trait Metering: Send + Sync {
    /// Read current counts and advisory soft limits for one workspace.
    fn usage(
        &self,
        workspace_id: Uuid,
        period: Period,
        plan: Plan,
        at: DateTime<Utc>,
        payment: PaymentNotice,
    ) -> Result<Usage>;
    /// Refresh job-derived daily totals and purge dates outside ninety UTC days.
    fn refresh(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<Refresh>;
    /// Read retained totals and histories for a pure workspace savings report.
    fn savings(
        &self,
        workspace_id: Uuid,
        range: DateRange,
        at: DateTime<Utc>,
        lists: &[PriceList; 2],
        divisor: MonthlyDivisor,
    ) -> Result<SavingsReport>;
}

/// Store-backed metering, constructed only by the composition root.
pub struct StoredMetering {
    store: Arc<dyn MeteringStore>,
}

impl StoredMetering {
    /// Inject the tenant-scoped persistence boundary.
    pub fn new(store: Arc<dyn MeteringStore>) -> Self {
        Self { store }
    }
}

impl Metering for StoredMetering {
    fn usage(
        &self,
        workspace_id: Uuid,
        period: Period,
        plan: Plan,
        at: DateTime<Utc>,
        payment: PaymentNotice,
    ) -> Result<Usage> {
        Ok(workspace_usage(
            &self.store.workspace_usage(workspace_id)?,
            period,
            plan,
            at,
            payment,
        ))
    }

    fn refresh(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<Refresh> {
        let Some(from) = retention_start(at) else {
            return Ok(Refresh::default());
        };
        let Some(until) = at.date_naive().succ_opt() else {
            return Ok(Refresh::default());
        };
        let facts = self.store.workspace_usage(workspace_id)?;
        let previous: BTreeMap<_, _> = facts
            .accounted_jobs
            .into_iter()
            .map(|job| (job.job.id, job))
            .collect();
        let mut outcome = Refresh::default();
        for job in facts.jobs {
            let current = AccountedJob {
                job,
                observed_at: at,
            };
            let before = previous.get(&current.job.id);
            let changes = accounting_deltas(before, &current, DateRange { from, until });
            let starts = start_deltas(before, &current, DateRange { from, until });
            if changes.is_empty() && starts.is_empty() && before.is_some() {
                continue;
            }
            if self
                .store
                .apply_accounting(workspace_id, before, &current, &changes, &starts)?
            {
                outcome.updated_jobs += 1;
            } else {
                outcome.stale_jobs += 1;
            }
        }
        let costs = self.store.cost_history(workspace_id)?;
        let previous: BTreeMap<_, _> = self
            .store
            .accounted_boat_costs(workspace_id)?
            .into_iter()
            .map(|cost| (cost.cost.runner_id, cost))
            .collect();
        for cost in costs.boat {
            let current = AccountedBoat {
                cost,
                observed_at: at,
            };
            let before = previous.get(&current.cost.runner_id);
            let changes = boat_cost_deltas(before, &current, DateRange { from, until });
            if changes.is_empty() && before.is_some() {
                continue;
            }
            if self
                .store
                .apply_boat_accounting(workspace_id, before, &current, &changes)?
            {
                outcome.updated_sandboxes += 1;
            } else {
                outcome.stale_sandboxes += 1;
            }
        }
        outcome.purged_rows = self.store.purge_daily_totals(workspace_id, from)?;
        Ok(outcome)
    }

    fn savings(
        &self,
        workspace_id: Uuid,
        range: DateRange,
        at: DateTime<Utc>,
        lists: &[PriceList; 2],
        divisor: MonthlyDivisor,
    ) -> Result<SavingsReport> {
        let facts = self.store.workspace_usage(workspace_id)?;
        let days = self
            .store
            .daily_totals(workspace_id, range.from, range.until)?;
        let costs = self.store.cost_history(workspace_id)?;
        Ok(savings_report(ReportInput {
            range,
            at,
            days: &days,
            machines: &facts.machines,
            costs: &costs,
            price_lists: lists,
            monthly_divisor: divisor,
        }))
    }
}

#[cfg(test)]
#[path = "_tests_/service_tests.rs"]
mod service_tests;
