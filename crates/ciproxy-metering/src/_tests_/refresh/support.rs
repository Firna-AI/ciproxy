//! Unimock state for deterministic refresh interleavings without runtime IO.

use std::sync::{Arc, Mutex};

use ciproxy_store_interface::metering::facts::{UsageJob, WorkspaceUsage};
use ciproxy_store_interface::metering::history::{BoatAccounting, BoatCost};
use ciproxy_store_interface::metering::store::MeteringStoreMock;
use ciproxy_store_interface::metering::totals::{AccountedBoat, AccountedJob};
use rust_decimal::Decimal;
use unimock::{MockFn, Unimock, matching};

use crate::service::{Metering, StoredMetering};

pub(super) struct State {
    pub(super) job: Option<UsageJob>,
    pub(super) job_cursor: Option<AccountedJob>,
    pub(super) cost: BoatCost,
    pub(super) cost_cursor: AccountedBoat,
    pub(super) minutes: Decimal,
    pub(super) usd: Decimal,
    pub(super) competing_write: Option<AccountedBoat>,
}

pub(super) fn service(state: Arc<Mutex<State>>) -> Arc<dyn Metering> {
    let has_job = state.lock().unwrap().job.is_some();
    let jobs = state.clone();
    let costs = state.clone();
    let write_jobs = state.clone();
    let write_costs = state;
    let job_write = MeteringStoreMock::apply_accounting
        .each_call(matching!(_, _, _, _, _))
        .answers_arc(Arc::new(move |_, _, previous, current, changes, _| {
            let mut state = write_jobs.lock().unwrap();
            if state.job_cursor.as_ref() != previous {
                return Ok(false);
            }
            state.minutes += changes
                .iter()
                .map(|change| change.machine_minutes + change.boat_minutes)
                .sum::<Decimal>();
            state.job_cursor = Some(current.clone());
            Ok(true)
        }));
    let common = (
        MeteringStoreMock::workspace_usage
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                let state = jobs.lock().unwrap();
                Ok(WorkspaceUsage {
                    jobs: state.job.iter().cloned().collect(),
                    accounted_jobs: state.job_cursor.iter().cloned().collect(),
                    ..WorkspaceUsage::default()
                })
            })),
        MeteringStoreMock::boat_accounting
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                let mut state = costs.lock().unwrap();
                let snapshot = BoatAccounting {
                    costs: vec![state.cost.clone()],
                    accounted: vec![state.cost_cursor.clone()],
                };
                if let Some(competing) = state.competing_write.take() {
                    state.cost = competing.cost.clone();
                    state.cost_cursor = competing;
                    state.usd = Decimal::new(10, 2);
                }
                Ok(snapshot)
            })),
        MeteringStoreMock::apply_boat_accounting
            .each_call(matching!(_, _, _, _))
            .answers_arc(Arc::new(move |_, _, previous, current, changes| {
                let mut state = write_costs.lock().unwrap();
                if Some(&state.cost_cursor) != previous {
                    return Ok(false);
                }
                state.usd += changes.iter().map(|change| change.usd).sum::<Decimal>();
                state.cost_cursor = current.clone();
                Ok(true)
            })),
        MeteringStoreMock::purge_daily_totals
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(|_, _, _| Ok(0))),
    );
    let store = if has_job {
        Unimock::new((common, job_write))
    } else {
        Unimock::new(common)
    };
    Arc::new(StoredMetering::new(Arc::new(store)))
}
