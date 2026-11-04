//! Signed events refresh current Stripe objects under the workspace lock.

use chrono::{DateTime, Utc};

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
        let failure_evidence = match &event.payload {
            Payload::InvoiceFailed(object) => Some(FailedPaymentEvidence {
                invoice_id: object.object.id.clone(),
                failed_at: DateTime::from_timestamp(event.created, 0).ok_or(Error::StripeState)?,
            }),
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
        let state = self.confirmed_state(&record.state, &current, failure_evidence.as_ref())?;
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

/// Failure identity and time built only from the verified failed-payment branch.
pub(super) struct FailedPaymentEvidence {
    invoice_id: String,
    failed_at: DateTime<Utc>,
}

impl FailedPaymentEvidence {
    pub(super) fn invoice_id(&self) -> &str {
        &self.invoice_id
    }

    pub(super) fn failed_at(&self) -> DateTime<Utc> {
        self.failed_at
    }
}
