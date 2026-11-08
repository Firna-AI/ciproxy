//! Durable period-end cancellation and last-organization cleanup.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::billing::store::BillingSession;
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::error::{Error, Result, ResultExt};

use super::completion::classify;
use super::runtime::BillingService;
use super::transition::terminal;

impl BillingService {
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
        let superseded = current.cancel_at_period_end
            && record
                .command
                .as_ref()
                .is_some_and(|command| matches!(command.kind, CommandKind::Change { .. }));
        session
            .commit(
                Some(&baseline),
                None,
                completed || superseded,
                removal && (terminal(&current) || current.cancel_at_period_end),
                self.clock.now(),
            )
            .internal()?;
        if terminal(&current) || current.cancel_at_period_end {
            self.observe_payments(session, &current);
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
            .internal()?;
        self.observe_payments(session, &current);
        Ok(())
    }
}
