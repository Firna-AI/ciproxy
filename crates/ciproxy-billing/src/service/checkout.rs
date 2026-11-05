//! Single-Checkout creation and shared return-page/event reconciliation.

use ciproxy_store_interface::billing::commands::{BillingCommand, CheckoutProgress, CommandKind};
use ciproxy_store_interface::billing::state::{Selection, Tier};
use ciproxy_store_interface::billing::store::{BillingRecord, BillingSession};

use crate::access::BillingAccess;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::types::{CheckoutSession, CheckoutStatus, PaymentStatus};

use super::runtime::BillingService;
use super::view::CheckoutOutcome;

impl BillingService {
    pub(super) fn open_checkout(
        &self,
        access: BillingAccess,
        selection: Selection,
    ) -> Result<CheckoutOutcome> {
        let workspace = access.require_payer()?;
        if selection.tier == Tier::Free {
            return Err(Error::PaidTierRequired);
        }
        let mut session = self.lock(workspace)?;
        let record = session.record().internal()?;
        if record.state.tier != Tier::Free {
            return Err(Error::CurrentTier);
        }
        if let Some(checkout) = &record.checkout {
            let current = self.stripe.checkout_session(&checkout.session_id)?;
            return self.apply_checkout(&mut *session, &record, &current, None);
        }
        let command = match &record.command {
            Some(command) if matches!(command.kind, CommandKind::Checkout { .. }) => {
                command.clone()
            }
            Some(_) => return Err(Error::PaymentConfirming),
            None => {
                let command = BillingCommand {
                    id: self.clock.command_id(),
                    started_at: self.clock.now(),
                    payment_expires_at: None,
                    kind: CommandKind::Checkout {
                        selection,
                        success_url: self.redirects.success_url.clone(),
                        cancel_url: self.redirects.cancel_url.clone(),
                    },
                };
                session.save_command(&command).internal()?;
                command
            }
        };
        let current =
            match self
                .stripe
                .checkout(workspace, record.state.customer_id.as_deref(), &command)
            {
                Ok(current) => current,
                Err(error) => {
                    self.release_rejected(&mut *session, &error)?;
                    return Err(error);
                }
            };
        session
            .save_checkout(&CheckoutProgress {
                command_id: command.id,
                session_id: current.id.clone(),
                url: current.url.clone(),
            })
            .internal()?;
        let record = session.record().internal()?;
        self.apply_checkout(&mut *session, &record, &current, None)
    }

    pub(super) fn reconcile_checkout(
        &self,
        access: BillingAccess,
        id: &str,
    ) -> Result<CheckoutOutcome> {
        access.require_payer()?;
        let mut session = self.lock(access.workspace_id)?;
        let record = session.record().internal()?;
        let current = self.stripe.checkout_session(id)?;
        self.apply_checkout(&mut *session, &record, &current, None)
    }

    pub(super) fn apply_checkout(
        &self,
        session: &mut dyn BillingSession,
        record: &BillingRecord,
        current: &CheckoutSession,
        event_id: Option<&str>,
    ) -> Result<CheckoutOutcome> {
        let matching_command = record.command.as_ref().is_some_and(|command| {
            matches!(command.kind, CommandKind::Checkout { .. })
                && current.metadata.command_id == Some(command.id)
        });
        let already_confirmed =
            current.subscription.is_some() && current.subscription == record.state.subscription_id;
        if current.metadata.workspace_id != Some(record.state.workspace_id)
            || (!matching_command && !already_confirmed)
        {
            return Err(Error::Ownership);
        }
        let now = self.clock.now();
        if current.status == CheckoutStatus::Expired {
            session
                .commit(None, event_id, matching_command, false, now)
                .internal()?;
            return Ok(CheckoutOutcome::Unfinished);
        }
        if current.status != CheckoutStatus::Complete
            || current.payment_status != PaymentStatus::Paid
        {
            session
                .commit(None, event_id, false, false, now)
                .internal()?;
            return Ok(match (current.status, &current.url) {
                (CheckoutStatus::Open, Some(url)) => CheckoutOutcome::Open { url: url.clone() },
                _ => CheckoutOutcome::Confirming,
            });
        }
        let id = current.subscription.as_deref().ok_or(Error::StripeState)?;
        let subscription = self.stripe.subscription(id)?;
        if subscription.id != id || current.customer.as_ref() != Some(&subscription.customer) {
            return Err(Error::Ownership);
        }
        let (subscription, state) =
            self.reconcile_subscription(session, &record.state, subscription, None)?;
        if let Some(mut command) = record.command.clone() {
            self.capture_pending_expiry(session, &state, &mut command, &subscription)?;
        }
        let changed = if state != record.state {
            Some(&state)
        } else {
            None
        };
        session
            .commit(changed, event_id, matching_command, false, now)
            .internal()?;
        if record.removal_cancel_requested {
            self.cancel_removed(session)?;
        }
        Ok(CheckoutOutcome::Confirmed(
            session.record().internal()?.state,
        ))
    }
}
