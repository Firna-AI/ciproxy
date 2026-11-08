//! Controlled refresh outcomes keep production retention behind accounting.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use ciproxy_metering::pricing::Plan;
use ciproxy_metering::report::catalog::PriceList;
use ciproxy_metering::report::range::DateRange;
use ciproxy_metering::report::savings::SavingsReport;
use ciproxy_metering::service::{Metering, Refresh, StoredMetering};
use ciproxy_metering::usage::{PaymentNotice, Usage};
use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::metering::history::Period;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub(super) enum Fault {
    Error,
    Panic,
    StaleJob,
    StaleSandbox,
}

pub(super) struct RefreshFault {
    pub(super) service: StoredMetering,
    pub(super) failed: Uuid,
    pub(super) fault: Fault,
    pub(super) calls: Arc<Mutex<Vec<Uuid>>>,
}

impl Metering for RefreshFault {
    fn refresh(&self, workspace: Uuid, at: DateTime<Utc>) -> Result<Refresh> {
        self.calls.lock().unwrap().push(workspace);
        if workspace != self.failed {
            return self.service.refresh(workspace, at);
        }
        match self.fault {
            Fault::Error => Err(Error::NotFound),
            Fault::Panic => panic!("controlled workspace refresh panic"),
            Fault::StaleJob => Ok(Refresh {
                stale_jobs: 1,
                ..Refresh::default()
            }),
            Fault::StaleSandbox => Ok(Refresh {
                stale_sandboxes: 1,
                ..Refresh::default()
            }),
        }
    }

    fn usage(
        &self,
        workspace: Uuid,
        period: Period,
        plan: Plan,
        at: DateTime<Utc>,
        payment: PaymentNotice,
    ) -> Result<Usage> {
        self.service.usage(workspace, period, plan, at, payment)
    }

    fn savings(
        &self,
        workspace: Uuid,
        range: DateRange,
        at: DateTime<Utc>,
        lists: &[PriceList; 2],
    ) -> Result<SavingsReport> {
        self.service.savings(workspace, range, at, lists)
    }
}
