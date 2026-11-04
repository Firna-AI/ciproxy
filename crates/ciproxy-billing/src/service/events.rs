//! Signed events refresh current Stripe objects under the workspace lock.

use chrono::DateTime;
use ciproxy_store_interface::billing::state::SubscriptionState;

use crate::error::{Error, Result, ResultExt};
use crate::webhook::{Payload, parse};

use super::completion::classify;
use super::runtime::BillingService;
use super::view::EventOutcome;

impl BillingService {
    pub(super) fn webhook(&self, signature: &str, raw: &[u8]) -> Result<EventOutcome> {
        self.signing_secret
            .verify(signature, raw, self.clock.now().timestamp())?;
        let event = parse(raw)?;
        let failure_invoice = match &event.payload {
            Payload::InvoiceFailed(object) => Some(object.object.id.clone()),
            _ => None,
        };
        let (workspace, target, checkout) = match event.payload {
            Payload::Checkout(object) => {
                let current = self.stripe.checkout_session(&object.object.id)?;
                (
                    current.metadata.workspace_id.ok_or(Error::Ownership)?,
                    current.id,
                    true,
                )
            }
            Payload::Subscription(object) => {
                let current = self.stripe.subscription(&object.object.id)?;
                (
                    current.metadata.workspace_id.ok_or(Error::Ownership)?,
                    current.id,
                    false,
                )
            }
            Payload::Invoice(object) | Payload::InvoiceFailed(object) => {
                let invoice = self.stripe.invoice(&object.object.id)?;
                let Some(id) = invoice.subscription_id() else {
                    return Ok(EventOutcome::Ignored);
                };
                let current = self.stripe.subscription(id)?;
                (
                    current.metadata.workspace_id.ok_or(Error::Ownership)?,
                    id.to_owned(),
                    false,
                )
            }
            Payload::Ignore => return Ok(EventOutcome::Ignored),
        };
        let mut session = self.lock(workspace)?;
        if session.event_applied(&event.id).internal()? {
            return Ok(EventOutcome::Duplicate);
        }
        let record = session.record().internal()?;
        if checkout {
            let current = self.stripe.checkout_session(&target)?;
            match self.apply_checkout(&mut *session, &record, &current, Some(&event.id)) {
                Ok(_) => return Ok(EventOutcome::Applied),
                Err(Error::Ownership) => {
                    session
                        .commit(None, Some(&event.id), false, false, self.clock.now())
                        .internal()?;
                    return Ok(EventOutcome::Ignored);
                }
                Err(error) => return Err(error),
            }
        }
        let current = self.stripe.subscription(&target)?;
        if record.state.subscription_id.as_deref() != Some(&current.id) {
            session
                .commit(None, Some(&event.id), false, false, self.clock.now())
                .internal()?;
            return Ok(EventOutcome::Ignored);
        }
        let mut state =
            self.confirmed_state(&record.state, &current, failure_invoice.as_deref())?;
        if state.status == SubscriptionState::PaymentFailed
            && failure_invoice.as_ref() == state.failed_invoice_id.as_ref()
        {
            let failure_at =
                DateTime::from_timestamp(event.created, 0).ok_or(Error::StripeState)?;
            state.failed_at = Some(
                state
                    .failed_at
                    .map_or(failure_at, |old| old.max(failure_at)),
            );
        }
        let complete = record
            .command
            .as_ref()
            .is_some_and(|command| classify(command, &current, self.clock.now()).finished());
        session
            .commit(
                Some(&state),
                Some(&event.id),
                complete,
                false,
                self.clock.now(),
            )
            .internal()?;
        if record.removal_cancel_requested {
            self.cancel_removed(&mut *session)?;
        }
        Ok(EventOutcome::Applied)
    }
}
