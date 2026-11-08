//! Paid item changes with authoritative quotes and durable Stripe retries.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::quotes::StoredQuote;
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use uuid::Uuid;

use crate::access::BillingAccess;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::{BillingReason, InvoiceStatus};

use super::completion::{Completion, classify};
use super::quote_source::source;
use super::runtime::BillingService;
use super::transition::{selection, terminal};
use super::view::ChangeOutcome;

#[derive(Clone, Copy)]
enum ChangeIntent {
    Quote(Uuid),
    AddOns(AddOns),
}

impl BillingService {
    pub(super) fn change(&self, access: BillingAccess, quote_id: Uuid) -> Result<ChangeOutcome> {
        self.change_items(access, ChangeIntent::Quote(quote_id))
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
        let mut session = self.lock(workspace)?;
        let mut record = session.record().internal()?;
        let quote = match intent {
            ChangeIntent::Quote(id) => {
                let quote = session.quote(id).internal()?.ok_or(Error::ReviewRequired)?;
                if let Some(id) = quote.command_id {
                    if !record
                        .command
                        .as_ref()
                        .is_some_and(|command| command.id == id && owns_quote(command, &quote))
                    {
                        return Err(Error::ReviewRequired);
                    }
                } else if self.clock.now() >= quote.expires_at {
                    return Err(Error::ReviewRequired);
                }
                Some(quote)
            }
            ChangeIntent::AddOns(_) => None,
        };
        if record.state.tier == Tier::Free {
            return Err(if quote.is_some() {
                Error::ReviewRequired
            } else {
                Error::PaidTierRequired
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
            return Err(if quote.is_some() {
                Error::ReviewRequired
            } else {
                Error::SubscriptionRequired
            });
        }
        let wanted = match intent {
            ChangeIntent::Quote(_) => quote.as_ref().ok_or(Error::ReviewRequired)?.selection,
            ChangeIntent::AddOns(add_ons) => Selection {
                tier: baseline.tier,
                add_ons,
            },
        };
        if quote
            .as_ref()
            .is_some_and(|quote| quote.command_id.is_some())
        {
            if progress == Some(Completion::Applied) {
                return Ok(ChangeOutcome::Applied(baseline));
            }
            if progress.is_some_and(Completion::finished) {
                return Err(Error::ReviewRequired);
            }
        } else if quote.is_none()
            && record.command.as_ref().is_some_and(|command| {
                matches!(command.kind, CommandKind::Change {selection, ..} if selection == wanted)
                    && progress == Some(Completion::Applied)
            })
        {
            return Ok(ChangeOutcome::Applied(baseline));
        }
        let mut command = match record
            .command
            .as_ref()
            .filter(|_| !progress.is_some_and(Completion::finished))
        {
            Some(command)
                if match &quote {
                    Some(quote) => owns_quote(command, quote),
                    None => {
                        matches!(command.kind, CommandKind::Change {selection, quote: None, ..} if selection == wanted)
                    }
                } =>
            {
                command.clone()
            }
            Some(_) => return Err(Error::PaymentConfirming),
            None => {
                if current.pending_update.is_some() {
                    return Err(Error::PaymentConfirming);
                }
                if let Some(quote) = &quote {
                    if quote.source != source(&current)? || self.clock.now() >= quote.expires_at {
                        return Err(Error::ReviewRequired);
                    }
                } else if selection(&current)? == wanted {
                    return Err(Error::CurrentTier);
                }
                let command = BillingCommand {
                    id: self.clock.command_id(),
                    started_at: self.clock.now(),
                    payment_expires_at: None,
                    kind: CommandKind::Change {
                        selection: wanted,
                        upgrade: quote.as_ref().map_or(
                            baseline.tier == Tier::Team && wanted.tier == Tier::Business,
                            |quote| quote.upgrade,
                        ),
                        quote: quote.as_ref().map(|quote| quote.payment.clone()),
                    },
                };
                if let Some(quote) = &quote {
                    if !session
                        .consume_quote(quote.payment.id, &command)
                        .internal()?
                    {
                        return Err(Error::ReviewRequired);
                    }
                } else {
                    session.save_command(&command).internal()?;
                }
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
}

fn owns_quote(command: &BillingCommand, quote: &StoredQuote) -> bool {
    matches!(&command.kind, CommandKind::Change {quote: Some(payment), ..} if payment.id == quote.payment.id)
}
