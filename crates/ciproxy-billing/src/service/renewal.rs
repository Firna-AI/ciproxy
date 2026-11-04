//! Current failed-renewal reads separate from upgrade payment information.

use ciproxy_store_interface::billing::state::BillingState;

use crate::error::{Error, Result};
use crate::stripe::wire::invoice::{BillingReason, Invoice, InvoiceStatus};
use crate::stripe::wire::types::{Status, Subscription};

use super::runtime::BillingService;
use super::transition::{confirmed, terminal};

pub(super) enum RenewalState {
    Unknown,
    Failed(Invoice),
    Recovered,
}

fn unresolved(invoice: &Invoice) -> bool {
    invoice.billing_reason == BillingReason::SubscriptionCycle
        && invoice.status != InvoiceStatus::Paid
}

impl BillingService {
    pub(super) fn confirmed_state(
        &self,
        old: &BillingState,
        current: &Subscription,
        failure_hint: Option<&str>,
    ) -> Result<BillingState> {
        let renewal = if terminal(current) {
            RenewalState::Unknown
        } else {
            self.failed_renewal(old, current, failure_hint)?
        };
        confirmed(old, current, self.clock.now(), &renewal)
    }

    fn failed_renewal(
        &self,
        old: &BillingState,
        current: &Subscription,
        failure_hint: Option<&str>,
    ) -> Result<RenewalState> {
        let mut recovered = false;
        let known_id = if old.subscription_id.as_ref() == Some(&current.id) {
            old.failed_invoice_id.as_deref()
        } else {
            None
        };
        if let Some(id) = known_id {
            let invoice = self.owned_invoice(current, id)?;
            if unresolved(&invoice) {
                return Ok(RenewalState::Failed(invoice));
            }
            recovered |= invoice.billing_reason == BillingReason::SubscriptionCycle;
        }
        if let Some(id) = failure_hint {
            let invoice = self.owned_invoice(current, id)?;
            if unresolved(&invoice) {
                return Ok(RenewalState::Failed(invoice));
            }
            recovered |= invoice.billing_reason == BillingReason::SubscriptionCycle;
        }
        if matches!(current.status, Status::PastDue | Status::Unpaid)
            && let Some(latest) = current.latest_invoice.as_ref()
            && latest.billing_reason == BillingReason::SubscriptionCycle
        {
            let invoice = self.owned_invoice(current, &latest.id)?;
            if unresolved(&invoice) {
                return Ok(RenewalState::Failed(invoice));
            }
            recovered = true;
        }
        Ok(if recovered {
            RenewalState::Recovered
        } else {
            RenewalState::Unknown
        })
    }

    fn owned_invoice(&self, current: &Subscription, id: &str) -> Result<Invoice> {
        let record = self.stripe.invoice(id)?;
        if record.invoice.id != id
            || record.customer != current.customer
            || record.subscription_id() != Some(current.id.as_str())
        {
            return Err(Error::Ownership);
        }
        Ok(record.invoice)
    }
}
