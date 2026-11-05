//! Persisted final collection outcomes drive confirmed immediate cancellation.

use ciproxy_store_interface::billing::renewals::RenewalOutcome;
use ciproxy_store_interface::billing::state::BillingState;
use ciproxy_store_interface::billing::store::BillingSession;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::types::Subscription;

use super::collection_decisions::eligible;
use super::invoice_evidence::InvoiceEvidence;
use super::runtime::BillingService;
use super::transition::terminal;

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
            renewal.outcome == RenewalOutcome::Open && eligible(&renewal.collection).is_some()
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
        let refreshed =
            self.observe_invoice(session, Some(renewal), &state, &fresh, &invoice, None)?;
        if eligible(&refreshed.collection).is_none() {
            let state = self.confirmed_state(session, &state, &fresh, None)?;
            return Ok((fresh, state));
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
