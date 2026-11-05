//! Durable invoice outcomes separate from upgrade and banner information.

use chrono::DateTime;
use ciproxy_store_interface::billing::renewals::{Renewal, RenewalOutcome};
use ciproxy_store_interface::billing::state::{BillingState, Tier};
use ciproxy_store_interface::billing::store::BillingSession;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::{BillingReason, Invoice, InvoiceStatus};
use crate::stripe::wire::types::Subscription;

use super::collection_stop::current_stop;
use super::invoice_evidence::InvoiceEvidence;
use super::runtime::BillingService;
use super::transition::{confirmed, terminal};

impl BillingService {
    pub(super) fn confirmed_state(
        &self,
        session: &mut dyn BillingSession,
        old: &BillingState,
        current: &Subscription,
        evidence: Option<&InvoiceEvidence>,
    ) -> Result<BillingState> {
        if old.tier != Tier::Free && old.subscription_id.as_ref() != Some(&current.id) {
            return Err(Error::Ownership);
        }
        confirmed(old, current, self.clock.now(), None)?;
        let renewal = if terminal(current) {
            None
        } else {
            self.renewal_state(session, old, current, evidence)?
        };
        confirmed(old, current, self.clock.now(), renewal.as_ref())
    }

    fn renewal_state(
        &self,
        session: &mut dyn BillingSession,
        old: &BillingState,
        current: &Subscription,
        evidence: Option<&InvoiceEvidence>,
    ) -> Result<Option<Renewal>> {
        let mut renewals = session.renewals(&current.id).internal()?;
        let mut ids = renewals
            .iter()
            .map(|renewal| renewal.invoice_id.clone())
            .collect::<Vec<_>>();
        if old.subscription_id.as_ref() == Some(&current.id)
            && let Some(id) = &old.failed_invoice_id
        {
            ids.push(id.clone());
        }
        if let Some(evidence) = evidence {
            ids.push(evidence.invoice_id.clone());
        }
        if let Some(latest) = &current.latest_invoice
            && latest.billing_reason == BillingReason::SubscriptionCycle
        {
            ids.push(latest.id.clone());
        }
        ids.sort();
        ids.dedup();
        for id in ids {
            if renewals.iter().any(|renewal| {
                renewal.invoice_id == id
                    && matches!(renewal.outcome, RenewalOutcome::Paid | RenewalOutcome::Void)
            }) {
                continue;
            }
            let invoice = self.owned_invoice(current, &id)?;
            let previous = renewals.iter().find(|renewal| renewal.invoice_id == id);
            if invoice.billing_reason != BillingReason::SubscriptionCycle && previous.is_none() {
                continue;
            }
            let mut renewal = observed(previous, old, current, &invoice, evidence)?;
            if renewal
                .automatic_stop
                .as_ref()
                .is_some_and(|stop| !current_stop(current, &invoice, stop))
            {
                renewal.automatic_stop = None;
            }
            if previous != Some(&renewal) {
                session.save_renewal(&renewal).internal()?;
            }
            renewals.retain(|renewal| renewal.invoice_id != id);
            renewals.push(renewal);
        }
        let failed = renewals
            .iter()
            .filter(|renewal| {
                renewal.outcome == RenewalOutcome::Open
                    && renewal.amount_remaining.is_none_or(|amount| amount > 0)
            })
            .max_by_key(|renewal| {
                (
                    old.failed_invoice_id.as_ref() == Some(&renewal.invoice_id),
                    renewal.failed_at,
                    &renewal.invoice_id,
                )
            });
        Ok(failed.cloned())
    }

    pub(super) fn owned_invoice(&self, current: &Subscription, id: &str) -> Result<Invoice> {
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

fn observed(
    previous: Option<&Renewal>,
    old: &BillingState,
    current: &Subscription,
    invoice: &Invoice,
    evidence: Option<&InvoiceEvidence>,
) -> Result<Renewal> {
    let mut renewal = previous.cloned().unwrap_or_else(|| Renewal {
        customer_id: current.customer.clone(),
        subscription_id: current.id.clone(),
        invoice_id: invoice.id.clone(),
        outcome: RenewalOutcome::Draft,
        amount_remaining: None,
        failed_at: (old.failed_invoice_id.as_ref() == Some(&invoice.id))
            .then_some(old.failed_at)
            .flatten(),
        next_retry_at: None,
        automatic_stop: None,
        collection_changed_at: None,
    });
    renewal.outcome = match invoice.status {
        InvoiceStatus::Draft => RenewalOutcome::Draft,
        InvoiceStatus::Open => RenewalOutcome::Open,
        InvoiceStatus::Paid => RenewalOutcome::Paid,
        InvoiceStatus::Void => RenewalOutcome::Void,
        InvoiceStatus::Uncollectible => RenewalOutcome::Uncollectible,
    };
    renewal.amount_remaining = invoice.amount_remaining;
    renewal.next_retry_at = invoice
        .next_payment_attempt
        .map(|seconds| DateTime::from_timestamp(seconds, 0).ok_or(Error::StripeState))
        .transpose()?;
    if let Some(evidence) = evidence.filter(|evidence| evidence.invoice_id == invoice.id) {
        if let Some(failed_at) = evidence.failed_at {
            renewal.failed_at = Some(
                renewal
                    .failed_at
                    .map_or(failed_at, |old| old.max(failed_at)),
            );
        }
        if let Some((at, stop)) = evidence.collection_change(current)
            && renewal.collection_changed_at.is_none_or(|old| at >= old)
        {
            renewal.collection_changed_at = Some(at);
            renewal.automatic_stop = stop;
        }
    }
    Ok(renewal)
}
