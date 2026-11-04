//! Current failed-renewal reads separate from upgrade payment information.

use ciproxy_store_interface::billing::state::BillingState;

use crate::error::{Error, Result};
use crate::stripe::wire::invoice::{BillingReason, Invoice, InvoiceStatus};
use crate::stripe::wire::types::{Status, Subscription};

use super::events::FailedPaymentEvidence;
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
        failure_evidence: Option<&FailedPaymentEvidence>,
    ) -> Result<BillingState> {
        let renewal = if terminal(current) {
            RenewalState::Unknown
        } else {
            self.failed_renewal(old, current, failure_evidence)?
        };
        let mut state = confirmed(old, current, self.clock.now(), &renewal)?;
        if let (Some(evidence), RenewalState::Failed(invoice)) = (failure_evidence, &renewal)
            && evidence.invoice_id() == invoice.id
        {
            let failed_at = evidence.failed_at();
            state.failed_at = Some(state.failed_at.map_or(failed_at, |old| old.max(failed_at)));
        }
        Ok(state)
    }

    fn failed_renewal(
        &self,
        old: &BillingState,
        current: &Subscription,
        failure_evidence: Option<&FailedPaymentEvidence>,
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
        if let Some(evidence) = failure_evidence {
            let invoice = self.owned_invoice(current, evidence.invoice_id())?;
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
