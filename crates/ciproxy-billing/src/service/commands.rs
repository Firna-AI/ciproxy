//! Paid item changes and durable period-end cancellation.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use ciproxy_store_interface::billing::store::BillingSession;
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::{BillingReason, InvoiceStatus};

use super::completion::{Completion, classify};
use super::runtime::BillingService;
use super::transition::{selection, terminal};
use super::view::ChangeOutcome;

#[derive(Clone, Copy)]
enum ChangeIntent {
    Tier(Selection),
    AddOns(AddOns),
}

impl ChangeIntent {
    fn resolve(self, confirmed_tier: Tier) -> Selection {
        match self {
            Self::Tier(selection) => selection,
            Self::AddOns(add_ons) => Selection {
                tier: confirmed_tier,
                add_ons,
            },
        }
    }
}

impl BillingService {
    pub(super) fn change(&self, access: BillingAccess, wanted: Selection) -> Result<ChangeOutcome> {
        self.change_items(access, ChangeIntent::Tier(wanted))
    }

    pub(super) fn change_add_ons(
        &self,
        access: BillingAccess,
        add_ons: AddOns,
    ) -> Result<ChangeOutcome> {
        self.change_items(access, ChangeIntent::AddOns(add_ons))
    }

    fn change_items(&self, access: BillingAccess, intent: ChangeIntent) -> Result<ChangeOutcome> {
        let workspace = access.require_payer()?;
        if matches!(
            intent,
            ChangeIntent::Tier(Selection {
                tier: Tier::Free,
                ..
            })
        ) {
            return Err(Error::PaidTierRequired);
        }
        let mut session = self.lock(workspace)?;
        let mut record = session.record().internal()?;
        if record.state.tier == Tier::Free {
            return Err(match intent {
                ChangeIntent::Tier(_) => Error::SubscriptionRequired,
                ChangeIntent::AddOns(_) => Error::PaidTierRequired,
            });
        }
        let current = self.current_subscription(&mut *session, &record.state)?;
        let (current, baseline) =
            self.reconcile_subscription(&mut *session, &record.state, current, None)?;
        if let Some(command) = record.command.as_mut() {
            self.capture_pending_expiry(&mut *session, &baseline, command, &current)?;
        }
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
        let wanted = intent.resolve(baseline.tier);
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
                if current.pending_update.is_some() {
                    return Err(Error::PaymentConfirming);
                }
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
        let (current, state) =
            self.reconcile_subscription(&mut *session, &baseline, current, None)?;
        self.capture_pending_expiry(&mut *session, &state, &mut command, &current)?;
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
                .filter(|invoice| {
                    invoice.billing_reason == BillingReason::SubscriptionUpdate
                        && invoice.status == InvoiceStatus::Open
                })
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
        let mut record = session.record().internal()?;
        let current = self.current_subscription(session, &record.state)?;
        let (current, baseline) =
            self.reconcile_subscription(session, &record.state, current, None)?;
        if let Some(command) = record.command.as_mut() {
            self.capture_pending_expiry(session, &baseline, command, &current)?;
        }
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
        let mut command = match record.command.as_ref().filter(|_| !completed) {
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
        let (current, state) = self.reconcile_subscription(session, &baseline, current, None)?;
        self.capture_pending_expiry(session, &state, &mut command, &current)?;
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
