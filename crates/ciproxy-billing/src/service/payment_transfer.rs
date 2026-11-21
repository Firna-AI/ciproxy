//! Retire counted sources before payment and restore only proven uncharged value.

use chrono::Duration;
use ciproxy_store_interface::billing::commands::BillingCommand;
use ciproxy_store_interface::billing::payments::{
    InvoiceAudit, PaymentWork, Restoration, Settlement,
};
use ciproxy_store_interface::billing::quotes::{PendingSource, StoredQuote};
use ciproxy_store_interface::billing::store::BillingSession;

use crate::alerts::AlertReason;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::pending::InvoiceItem;
use crate::stripe::wire::types::Subscription;

use super::payment_work::{new_work, owns_command};
use super::quote_source::source;
use super::runtime::BillingService;

pub(super) fn same_source(source: &PendingSource, item: &InvoiceItem) -> bool {
    item.id == source.id
        && item.customer == source.customer_id
        && item.currency == "usd"
        && item.amount == source.amount_cents
        && item.subscription_id() == source.subscription_id.as_deref()
        && item.invoice.is_none()
}

impl BillingService {
    fn same_balance(&self, quote: &StoredQuote) -> Result<bool> {
        match quote.payment.starting_balance_cents {
            Some(balance) => {
                Ok(self.stripe.customer_balance(&quote.source.customer_id)? == balance)
            }
            None => Ok(false),
        }
    }

    pub(super) fn validate_offset(&self, quote: &StoredQuote) -> Result<()> {
        if !self.same_balance(quote)? {
            return Err(Error::ReviewRequired);
        }
        if let Some(offset) = &quote.payment.offset {
            let pending = self.stripe.pending_items(&quote.source.customer_id)?;
            if offset
                .sources
                .iter()
                .any(|source| !pending.iter().any(|item| same_source(source, item)))
            {
                return Err(Error::ReviewRequired);
            }
        }
        Ok(())
    }

    pub(super) fn prepare_payment(
        &self,
        session: &mut dyn BillingSession,
        quote: &StoredQuote,
        command: &BillingCommand,
        current: &Subscription,
    ) -> Result<bool> {
        if quote.payment.starting_balance_cents.is_none() {
            return Err(Error::PaymentConfirming);
        }
        let mut work = match session.payment(command.id).internal()? {
            Some(work) => work,
            None => {
                let work = new_work(
                    session.record().internal()?.state.workspace_id,
                    quote,
                    command,
                );
                session.save_payment(&work).internal()?;
                work
            }
        };
        let record = session.record().internal()?;
        if !owns_command(&work, record.command.as_ref()) {
            return Err(Error::ReviewRequired);
        }
        if record.removal_cancel_requested {
            if work.submitted_at.is_none() {
                self.restore_payment(session, &mut work)?;
                session
                    .commit(None, None, true, false, self.clock.now())
                    .internal()?;
                self.save_payment(session, &mut work)?;
                return Err(Error::ReviewRequired);
            }
            self.inspect_payment(session, &mut work, current)?;
            return Ok(false);
        }
        if work.submitted_at.is_some() {
            self.inspect_payment(session, &mut work, current)?;
            if matches!(
                work.settlement,
                Settlement::Restored | Settlement::Restoring | Settlement::Abandoned
            ) {
                if matches!(
                    work.settlement,
                    Settlement::Restored | Settlement::Abandoned
                ) {
                    session
                        .commit(None, None, true, false, self.clock.now())
                        .internal()?;
                    self.save_payment(session, &mut work)?;
                }
                return Err(Error::ReviewRequired);
            }
            if work.invoice_id.is_some() {
                return Ok(false);
            }
            if work.superseded
                || source(current)? != work.quote.source
                || self.clock.now() >= work.quote.source.period.ends_at
                || !self.same_balance(&work.quote)?
            {
                work.settlement = Settlement::Unresolved;
                work.superseded = true;
                if !work.recovery_alerted {
                    self.payment_alert(&work, AlertReason::RecoveryPending, None);
                    work.recovery_alerted = true;
                }
                self.save_payment(session, &mut work)?;
                return Err(Error::PaymentConfirming);
            }
            if self.clock.now() - work.started_at >= Duration::hours(24) {
                return Err(Error::PaymentConfirming);
            }
        }
        if matches!(
            work.settlement,
            Settlement::Restored | Settlement::Restoring | Settlement::Abandoned
        ) {
            if matches!(
                work.settlement,
                Settlement::Restored | Settlement::Abandoned
            ) {
                session
                    .commit(None, None, true, false, self.clock.now())
                    .internal()?;
                self.save_payment(session, &mut work)?;
            }
            return Err(Error::ReviewRequired);
        }
        if work.submitted_at.is_none()
            && (source(current)? != work.quote.source
                || self.clock.now() >= work.quote.source.period.ends_at
                || current.pending_update.is_some())
        {
            self.restore_payment(session, &mut work)?;
            session
                .commit(None, None, true, false, self.clock.now())
                .internal()?;
            self.save_payment(session, &mut work)?;
            return Err(Error::ReviewRequired);
        }
        if !work.marked {
            self.stripe
                .mark_payment(&current.id, quote.payment.id, command.id)?;
            work.marked = true;
            self.save_payment(session, &mut work)?;
        }
        let sources = work
            .quote
            .payment
            .offset
            .as_ref()
            .map(|offset| offset.sources.clone())
            .unwrap_or_default();
        for source in sources {
            if work.retired.contains(&source.id) {
                continue;
            }
            let item = self.stripe.invoice_item(&source.id)?;
            if item.is_none() && work.retiring.as_ref() == Some(&source.id) {
                work.retired.push(source.id);
                work.retiring = None;
                self.save_payment(session, &mut work)?;
                continue;
            }
            if !item.as_ref().is_some_and(|item| same_source(&source, item)) {
                self.restore_payment(session, &mut work)?;
                session
                    .commit(None, None, true, false, self.clock.now())
                    .internal()?;
                self.save_payment(session, &mut work)?;
                return Err(Error::ReviewRequired);
            }
            work.retiring = Some(source.id.clone());
            self.save_payment(session, &mut work)?;
            self.stripe.delete_invoice_item(&source.id)?;
            work.retired.push(source.id);
            work.retiring = None;
            self.save_payment(session, &mut work)?;
        }
        if work.submitted_at.is_none() {
            let state = session.record().internal()?.state;
            let fresh = self.current_subscription(session, &state)?;
            if source(&fresh)? != work.quote.source
                || self.clock.now() >= work.quote.source.period.ends_at
                || fresh.pending_update.is_some()
                || !self.same_balance(&work.quote)?
                || session.record().internal()?.removal_cancel_requested
            {
                self.restore_payment(session, &mut work)?;
                session
                    .commit(None, None, true, false, self.clock.now())
                    .internal()?;
                self.save_payment(session, &mut work)?;
                return Err(Error::ReviewRequired);
            }
        }
        let intent = session.record().internal()?;
        if intent.removal_cancel_requested || !owns_command(&work, intent.command.as_ref()) {
            if work.submitted_at.is_some() {
                return Ok(false);
            }
            self.restore_payment(session, &mut work)?;
            if owns_command(&work, intent.command.as_ref()) {
                session
                    .commit(None, None, true, false, self.clock.now())
                    .internal()?;
            }
            self.save_payment(session, &mut work)?;
            return Err(Error::ReviewRequired);
        }
        work.submitted_at.get_or_insert(self.clock.now());
        work.settlement = Settlement::Pending;
        self.save_payment(session, &mut work)?;
        Ok(true)
    }

    pub(super) fn restore_payment(
        &self,
        session: &mut dyn BillingSession,
        work: &mut PaymentWork,
    ) -> Result<()> {
        if matches!(
            work.settlement,
            Settlement::Restored | Settlement::Abandoned
        ) {
            return self.save_payment(session, work);
        }
        if work.submitted_at.is_some() && work.settlement != Settlement::Restoring {
            return Err(Error::StripeState);
        }
        if let Some(id) = work.retiring.clone() {
            if self.stripe.invoice_item(&id)?.is_none() && !work.retired.contains(&id) {
                work.retired.push(id);
            }
            work.retiring = None;
            self.save_payment(session, work)?;
        }
        if work.restoration.is_none() {
            let amount: i128 = work
                .quote
                .payment
                .offset
                .as_ref()
                .into_iter()
                .flat_map(|offset| &offset.sources)
                .filter(|source| work.retired.contains(&source.id))
                .map(|source| i128::from(source.effective_cents))
                .sum();
            work.restoration = Some(Restoration {
                amount_cents: amount.try_into().internal()?,
                started_at: self.clock.now(),
                transaction_id: None,
            });
            work.settlement = Settlement::Restoring;
            self.save_payment(session, work)?;
        }
        let restore = work.restoration.as_ref().ok_or(Error::StripeState)?;
        if restore.amount_cents != 0 && restore.transaction_id.is_none() {
            let id = self.stripe.restore_balance(
                &work.quote.source.customer_id,
                work.id,
                restore.amount_cents,
            )?;
            if let Some(restore) = work.restoration.as_mut() {
                restore.transaction_id = Some(id);
            }
        }
        work.settlement = Settlement::Restored;
        work.audit = InvoiceAudit::NoInvoice;
        self.save_payment(session, work)
    }
}
