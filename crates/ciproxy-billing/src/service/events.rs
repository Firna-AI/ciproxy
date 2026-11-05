//! Signed events refresh current Stripe objects under the workspace lock.

use uuid::Uuid;

use crate::error::{Error, Result, ResultExt};
use crate::webhook::{Payload, parse};

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
                let Some(target) = self.invoice_target(&object.object.id)? else {
                    return Ok(EventOutcome::Ignored);
                };
                target
            }
            Payload::InvoiceUpdated(update) => {
                let Some(target) = self.invoice_target(&update.object.id)? else {
                    return Ok(EventOutcome::Ignored);
                };
                target
            }
            Payload::Ignore => return Ok(EventOutcome::Ignored),
        };
        let mut session = self.lock(workspace)?;
        if session.event_applied(&event.id).internal()? {
            return Ok(EventOutcome::Duplicate);
        }
        let mut record = session.record().internal()?;
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

    fn invoice_target(&self, invoice_id: &str) -> Result<Option<(Uuid, String, bool)>> {
        let invoice = self.stripe.invoice(invoice_id)?;
        let Some(id) = invoice.subscription_id() else {
            return Ok(None);
        };
        let current = self.stripe.subscription(id)?;
        Ok(Some((
            current.metadata.workspace_id.ok_or(Error::Ownership)?,
            id.to_owned(),
            false,
        )))
    }
}
