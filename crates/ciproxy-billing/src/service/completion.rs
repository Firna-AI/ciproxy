//! Shared completion rules for durable paid commands and rejected mutations.

use chrono::{DateTime, Duration, Utc};
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::BillingState;
use ciproxy_store_interface::billing::store::BillingSession;

use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::{BillingReason, InvoiceStatus};
use crate::stripe::wire::types::Subscription;

use super::runtime::BillingService;
use super::transition::{selection, terminal};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Completion {
    Pending,
    Applied,
    Expired,
    Ended,
}

impl Completion {
    pub(super) fn finished(self) -> bool {
        self != Self::Pending
    }
}

/// Classify only a fresh Stripe read while the billing session is held.
pub(super) fn classify(
    command: &BillingCommand,
    current: &Subscription,
    now: DateTime<Utc>,
) -> Completion {
    match command.kind {
        CommandKind::Checkout { .. } => Completion::Pending,
        CommandKind::Change {
            selection: wanted, ..
        } => {
            if terminal(current) {
                return Completion::Ended;
            }
            if current.pending_update.is_some() {
                return Completion::Pending;
            }
            if selection(current).is_ok_and(|actual| actual == wanted) {
                return Completion::Applied;
            }
            let expired = command.payment_expires_at.is_some_and(|at| at <= now)
                || now.signed_duration_since(command.started_at) >= Duration::hours(24)
                || current.latest_invoice.as_ref().is_some_and(|invoice| {
                    invoice.billing_reason == BillingReason::SubscriptionUpdate
                        && invoice.created >= command.started_at.timestamp()
                        && matches!(
                            invoice.status,
                            InvoiceStatus::Void | InvoiceStatus::Uncollectible
                        )
                });
            if expired {
                Completion::Expired
            } else {
                Completion::Pending
            }
        }
        CommandKind::Cancel => {
            if terminal(current) {
                Completion::Ended
            } else if current.cancel_at_period_end {
                Completion::Applied
            } else {
                Completion::Pending
            }
        }
    }
}

impl BillingService {
    pub(super) fn capture_pending_expiry(
        &self,
        session: &mut dyn BillingSession,
        state: &BillingState,
        command: &mut BillingCommand,
        current: &Subscription,
    ) -> Result<()> {
        match command.kind {
            CommandKind::Change { .. } => {}
            CommandKind::Checkout { .. } | CommandKind::Cancel => return Ok(()),
        }
        if state.subscription_id.as_deref() != Some(current.id.as_str())
            || current.metadata.workspace_id != Some(state.workspace_id)
        {
            return Ok(());
        }
        let Some(pending) = &current.pending_update else {
            return Ok(());
        };
        let expiry = DateTime::from_timestamp(pending.expires_at, 0).ok_or(Error::StripeState)?;
        if command.payment_expires_at != Some(expiry) {
            command.payment_expires_at = Some(expiry);
            session.save_command(command).internal()?;
        }
        Ok(())
    }

    pub(super) fn release_rejected(
        &self,
        session: &mut dyn BillingSession,
        error: &Error,
    ) -> Result<()> {
        if matches!(
            error,
            Error::StripeRejected { .. } | Error::StripeValidationRejected { .. }
        ) {
            session
                .commit(None, None, true, false, self.clock.now())
                .internal()?;
        }
        Ok(())
    }
}
