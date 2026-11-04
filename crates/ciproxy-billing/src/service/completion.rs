//! Shared completion rules for durable paid commands and rejected mutations.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
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
    pub(super) fn release_rejected(
        &self,
        session: &mut dyn BillingSession,
        error: &Error,
    ) -> Result<()> {
        if matches!(error, Error::StripeRejected { .. }) {
            session
                .commit(None, None, true, false, self.clock.now())
                .internal()?;
        }
        Ok(())
    }
}
