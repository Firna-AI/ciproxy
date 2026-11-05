//! Persisted final collection outcomes drive confirmed immediate cancellation.

use ciproxy_store_interface::billing::renewals::{AutomaticStop, RenewalOutcome};
use ciproxy_store_interface::billing::state::BillingState;
use ciproxy_store_interface::billing::store::BillingSession;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::{BillingReason, CollectionMethod, Invoice, InvoiceStatus};
use crate::stripe::wire::types::Subscription;

use super::invoice_evidence::InvoiceEvidence;
use super::runtime::BillingService;
use super::transition::terminal;

pub(super) fn current_stop(
    current: &Subscription,
    invoice: &Invoice,
    stop: &AutomaticStop,
) -> bool {
    current.collection_method == Some(CollectionMethod::ChargeAutomatically)
        && current.pause_collection.is_none()
        && invoice.billing_reason == BillingReason::SubscriptionCycle
        && invoice.status == InvoiceStatus::Open
        && invoice.collection_method == Some(CollectionMethod::ChargeAutomatically)
        && invoice.amount_remaining.is_some_and(|amount| amount > 0)
        && invoice.attempted == Some(true)
        && invoice.attempt_count == Some(stop.attempt_count)
        && invoice.auto_advance == Some(false)
        && invoice.next_payment_attempt.is_none()
}

impl BillingService {
    pub(super) fn reconcile_subscription(
        &self,
        session: &mut dyn BillingSession,
        old: &BillingState,
        current: Subscription,
        evidence: Option<&InvoiceEvidence>,
    ) -> Result<(Subscription, BillingState)> {
        let state = self.confirmed_state(session, old, &current, evidence)?;
        if terminal(&current) {
            return Ok((current, state));
        }
        let renewals = session.renewals(&current.id).internal()?;
        let Some(renewal) = renewals.iter().find(|renewal| {
            renewal.outcome == RenewalOutcome::Open && renewal.automatic_stop.is_some()
        }) else {
            return Ok((current, state));
        };
        session
            .commit(Some(&state), None, false, false, self.clock.now())
            .internal()?;
        let fresh = self.current_subscription(&state)?;
        if terminal(&fresh) {
            let ended = self.confirmed_state(session, &state, &fresh, None)?;
            return Ok((fresh, ended));
        }
        let invoice = self.owned_invoice(&fresh, &renewal.invoice_id)?;
        if !renewal
            .automatic_stop
            .as_ref()
            .is_some_and(|stop| current_stop(&fresh, &invoice, stop))
        {
            let observed = self.confirmed_state(session, &state, &fresh, None)?;
            return Ok((fresh, observed));
        }
        let ended = self.stripe.cancel_now(&fresh.id, &renewal.invoice_id)?;
        if ended.id != fresh.id
            || ended.customer != fresh.customer
            || ended.metadata.workspace_id != fresh.metadata.workspace_id
        {
            return Err(Error::Ownership);
        }
        if !terminal(&ended) {
            return Err(Error::StripeState);
        }
        let state = self.confirmed_state(session, &state, &ended, None)?;
        Ok((ended, state))
    }
}
