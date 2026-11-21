//! Recover financial and diagnostic work independently of transient commands.

use ciproxy_store_interface::billing::commands::CommandKind;
use uuid::Uuid;

use crate::access::{BillingAccess, Role};
use crate::alerts::AlertReason;
use crate::error::{Error, Result, ResultExt};

use super::completion::classify;
use super::runtime::BillingService;

impl BillingService {
    pub(super) fn reconcile_payments(&self, workspace: Uuid) -> Result<()> {
        let resume = {
            let mut session = self.lock(workspace)?;
            let works = session.payments().internal()?;
            for mut work in works {
                let current = match self.stripe.subscription(&work.quote.source.subscription_id) {
                    Ok(current) => current,
                    Err(_) => {
                        self.payment_alert(&work, AlertReason::RecoveryPending, None);
                        continue;
                    }
                };
                if self
                    .inspect_payment(&mut *session, &mut work, &current)
                    .is_err()
                {
                    self.payment_alert(&work, AlertReason::RecoveryPending, None);
                }
                let record = session.record().internal()?;
                if record.state.subscription_id.as_deref() == Some(current.id.as_str()) {
                    let (_, state) = self.reconcile_subscription(
                        &mut *session,
                        &record.state,
                        current.clone(),
                        None,
                    )?;
                    let completed = record.command.as_ref().is_some_and(|command| {
                        classify(command, &current, self.clock.now()).finished()
                    });
                    session
                        .commit(Some(&state), None, completed, false, self.clock.now())
                        .internal()?;
                    self.observe_payments(&mut *session, &current);
                }
            }
            let record = session.record().internal()?;
            record
                .command
                .filter(|_| !record.removal_cancel_requested)
                .and_then(|command| match command.kind {
                    CommandKind::Change {
                        quote: Some(quote), ..
                    } => Some(quote.id),
                    _ => None,
                })
        };
        if let Some(quote) = resume {
            match self.change(
                BillingAccess {
                    workspace_id: workspace,
                    role: Role::Admin,
                },
                quote,
            ) {
                Ok(_)
                | Err(Error::ReviewRequired | Error::PaymentConfirming | Error::CurrentTier) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
