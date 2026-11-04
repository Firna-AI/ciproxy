//! Paid item changes and durable period-end cancellation.

use chrono::DateTime;
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{Selection, Tier};
use ciproxy_store_interface::billing::store::BillingSession;
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::InvoiceStatus;

use super::runtime::BillingService;
use super::transition::{confirmed, selection};
use super::view::ChangeOutcome;

impl BillingService {
    pub(super) fn change(&self, access: BillingAccess, wanted: Selection) -> Result<ChangeOutcome> {
        let workspace = access.require_payer()?;
        if wanted.tier == Tier::Free {
            return Err(Error::PaidTierRequired);
        }
        let mut session = self.lock(workspace)?;
        let record = session.record().internal()?;
        if record.state.tier == Tier::Free {
            return Err(Error::SubscriptionRequired);
        }
        let current = self.current_subscription(&record.state)?;
        let already_applied = record.command.as_ref().is_some_and(|command| {
            matches!(command.kind, CommandKind::Change { selection: wanted, .. }
                if selection(&current).is_ok_and(|actual| actual == wanted))
                && current.pending_update.is_none()
        });
        let expired = !already_applied
            && record.command.as_ref().is_some_and(|command| {
                matches!(command.kind, CommandKind::Change { .. })
                    && current.pending_update.is_none()
                    && (command
                        .payment_expires_at
                        .is_some_and(|at| at <= self.clock.now())
                        || current.latest_invoice.as_ref().is_some_and(|invoice| {
                            invoice.created >= command.started_at.timestamp()
                                && matches!(
                                    invoice.status,
                                    InvoiceStatus::Void | InvoiceStatus::Uncollectible
                                )
                        }))
            });
        let mut command = match record.command.as_ref().filter(|_| !expired) {
            Some(command) if matches!(command.kind, CommandKind::Change { selection, .. } if selection == wanted) => {
                command.clone()
            }
            Some(_) => return Err(Error::PaymentConfirming),
            None => {
                if selection(&current)? == wanted {
                    return Err(Error::CurrentTier);
                }
                let command = BillingCommand {
                    id: self.clock.command_id(),
                    started_at: self.clock.now(),
                    payment_expires_at: None,
                    kind: CommandKind::Change {
                        selection: wanted,
                        upgrade: record.state.tier == Tier::Team && wanted.tier == Tier::Business,
                    },
                };
                session.save_command(&command).internal()?;
                command
            }
        };
        let current = if selection(&current)? == wanted || current.pending_update.is_some() {
            current
        } else {
            match self.stripe.change(&current, wanted, &command) {
                Ok(current) => current,
                Err(Error::StripeRejected { status }) => {
                    session
                        .commit(None, None, true, false, self.clock.now())
                        .internal()?;
                    return Err(Error::StripeRejected { status });
                }
                Err(error) => return Err(error),
            }
        };
        if let Some(pending) = &current.pending_update {
            command.payment_expires_at = DateTime::from_timestamp(pending.expires_at, 0);
            session.save_command(&command).internal()?;
        }
        let state = confirmed(&record.state, &current, self.clock.now())?;
        let applied = current.pending_update.is_none() && selection(&current)? == wanted;
        session
            .commit(Some(&state), None, applied, false, self.clock.now())
            .internal()?;
        if applied {
            return Ok(ChangeOutcome::Applied(state));
        }
        Ok(ChangeOutcome::PaymentRequired {
            payment_url: current
                .latest_invoice
                .and_then(|invoice| invoice.hosted_invoice_url),
        })
    }

    pub(super) fn move_to_free(&self, access: BillingAccess) -> Result<()> {
        let workspace = access.require_payer()?;
        let mut session = self.lock(workspace)?;
        self.cancel_subscription(&mut *session, false)
    }

    pub(super) fn reconcile_removal(&self, workspace: Uuid) -> Result<()> {
        let mut session = self.lock(workspace)?;
        self.cancel_removed(&mut *session)
    }

    pub(super) fn cancel_removed(&self, session: &mut dyn BillingSession) -> Result<()> {
        let record = session.record().internal()?;
        if !record.removal_cancel_requested {
            return Ok(());
        }
        if record.state.tier == Tier::Free {
            let pending_checkout = record
                .command
                .as_ref()
                .is_some_and(|command| matches!(command.kind, CommandKind::Checkout { .. }));
            if !pending_checkout {
                session
                    .commit(None, None, false, true, self.clock.now())
                    .internal()?;
            }
            return Ok(());
        }
        self.cancel_subscription(session, true)
    }

    fn cancel_subscription(&self, session: &mut dyn BillingSession, removal: bool) -> Result<()> {
        let record = session.record().internal()?;
        let id = record
            .state
            .subscription_id
            .as_deref()
            .ok_or(Error::SubscriptionRequired)?;
        let current = self.current_subscription(&record.state)?;
        if current.cancel_at_period_end {
            let state = confirmed(&record.state, &current, self.clock.now())?;
            session
                .commit(
                    Some(&state),
                    None,
                    matches!(
                        record.command.as_ref().map(|command| &command.kind),
                        Some(CommandKind::Cancel)
                    ),
                    removal,
                    self.clock.now(),
                )
                .internal()?;
            return Ok(());
        }
        let command = match &record.command {
            Some(command) if matches!(command.kind, CommandKind::Cancel) => command.clone(),
            Some(command) if matches!(command.kind, CommandKind::Checkout { .. }) => {
                return Err(Error::PaymentConfirming);
            }
            _ => {
                let command = BillingCommand {
                    id: self.clock.command_id(),
                    started_at: self.clock.now(),
                    payment_expires_at: None,
                    kind: CommandKind::Cancel,
                };
                session.save_command(&command).internal()?;
                command
            }
        };
        let current = self.stripe.cancel(id, command.id)?;
        let state = confirmed(&record.state, &current, self.clock.now())?;
        session
            .commit(Some(&state), None, true, removal, self.clock.now())
            .internal()
    }
}
