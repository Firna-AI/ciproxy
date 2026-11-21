//! Typed operator events captured at the hosted billing boundary.

use std::sync::Mutex;

use ciproxy_billing::alerts::{BillingAlert, BillingAlerts};

#[derive(Default)]
pub(super) struct OperatorEvents(pub(super) Mutex<Vec<BillingAlert>>);

impl BillingAlerts for OperatorEvents {
    fn emit(&self, alert: &BillingAlert) {
        self.0.lock().unwrap().push(alert.clone());
    }
}
