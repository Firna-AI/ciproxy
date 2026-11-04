//! Paid item changes and durable period-end cancellation.

use chrono::DateTime;
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{Selection, Tier};
use ciproxy_store_interface::billing::store::BillingSession;
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::error::{Error, Result, ResultExt};

use super::completion::{Completion, classify};
use super::runtime::BillingService;
use super::transition::{selection, terminal};
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
        let baseline = self.confirmed_state(&record.state, &current, None)?;
        let progress = record
            .command
            .as_ref()
            .map(|command| classify(command, &current, self.clock.now()));
        session
            .commit(
                Some(&baseline),
                None,
                progress.is_some_and(Completion::finished),
                false,
                self.clock.now(),
            )
            .internal()?;
        if terminal(&current) {
            return Err(Error::SubscriptionRequired);
        }
        let original_retry = record
            .command
            .as_ref()
            .is_some_and(|command| match command.kind {
                CommandKind::Change { selection, .. } => {
                    selection == wanted && progress == Some(Completion::Applied)
                }
                CommandKind::Cancel | CommandKind::Checkout { .. } => false,
            });
        if original_retry {
            return Ok(ChangeOutcome::Applied(baseline));
        }
        let mut command = match record
            .command
            .as_ref()
            .filter(|_| !progress.is_some_and(Completion::finished))
        {
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
                        upgrade: baseline.tier == Tier::Team && wanted.tier == Tier::Business,
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
                Err(error) => {
                    self.release_rejected(&mut *session, &error)?;
                    return Err(error);
                }
            }
        };
        if let Some(pending) = &current.pending_update {
            command.payment_expires_at = DateTime::from_timestamp(pending.expires_at, 0);
            session.save_command(&command).internal()?;
        }
        let state = self.confirmed_state(&baseline, &current, None)?;
        let progress = classify(&command, &current, self.clock.now());
        session
            .commit(
                Some(&state),
                None,
                progress.finished(),
                false,
                self.clock.now(),
            )
            .internal()?;
        if progress == Completion::Ended {
            return Err(Error::SubscriptionRequired);
        }
        if progress == Completion::Applied {
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
        let current = self.current_subscription(&record.state)?;
        let baseline = self.confirmed_state(&record.state, &current, None)?;
        let completed = record
            .command
            .as_ref()
            .is_some_and(|command| classify(command, &current, self.clock.now()).finished());
        session
            .commit(
                Some(&baseline),
                None,
                completed,
                removal && (terminal(&current) || current.cancel_at_period_end),
                self.clock.now(),
            )
            .internal()?;
        if terminal(&current) || current.cancel_at_period_end {
            return Ok(());
        }
        let command = match record.command.as_ref().filter(|_| !completed) {
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
        let current = match self.stripe.cancel(&current.id, command.id) {
            Ok(current) => current,
            Err(error) => {
                self.release_rejected(session, &error)?;
                return Err(error);
            }
        };
        let state = self.confirmed_state(&baseline, &current, None)?;
        session
            .commit(
                Some(&state),
                None,
                classify(&command, &current, self.clock.now()).finished(),
                removal,
                self.clock.now(),
            )
            .internal()
    }
}
