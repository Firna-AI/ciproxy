//! Signed events refresh current Stripe objects under the workspace lock.

use crate::error::{Error, Result, ResultExt};
use crate::webhook::parse;

use super::completion::classify;
use super::invoice_evidence::InvoiceEvidence;
use super::runtime::BillingService;
use super::view::EventOutcome;

impl BillingService {
    pub(super) fn webhook(&self, signature: &str, raw: &[u8]) -> Result<EventOutcome> {
        self.signing_secret
            .verify(signature, raw, self.clock.now().timestamp())?;
        let event = parse(raw)?;
        let evidence = InvoiceEvidence::from_event(&event)?;
        let Some(route) = self.route_event(&event)? else {
            return Ok(EventOutcome::Ignored);
        };
        let target = route.target;
        let mut session = self.lock(route.workspace)?;
        if session.event_applied(&event.id).internal()? {
            return Ok(EventOutcome::Duplicate);
        }
        let mut record = session.record().internal()?;
        if route.checkout {
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
        if record.state.subscription_id.as_deref() != Some(&target) {
            session
                .commit(None, Some(&event.id), false, false, self.clock.now())
                .internal()?;
            return Ok(EventOutcome::Ignored);
        }
        let current = self.read_subscription(&mut *session, &record.state, &target)?;
        let (current, state) =
            self.reconcile_subscription(&mut *session, &record.state, current, evidence.as_ref())?;
        if let Some(command) = record.command.as_mut() {
            self.capture_pending_expiry(&mut *session, &state, command, &current)?;
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
