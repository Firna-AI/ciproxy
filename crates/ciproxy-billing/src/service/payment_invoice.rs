//! Exact invoice evidence settles transferred value and emits diagnostic differences.

use chrono::Duration;
use ciproxy_store_interface::billing::payments::{InvoiceAudit, PaymentWork, Settlement};
use ciproxy_store_interface::billing::store::BillingSession;

use crate::alerts::AlertReason;
use crate::error::{Error, Result, ResultExt};
use crate::stripe::wire::invoice::{BillingReason, Invoice, InvoiceStatus, RetrievedInvoice};
use crate::stripe::wire::types::Subscription;

use super::payment_work::owns_command;
use super::quote_source::source;
use super::runtime::BillingService;
use super::transition::{selection, terminal};

fn matching(work: &PaymentWork, invoice: &Invoice) -> bool {
    let marker = invoice
        .parent
        .as_ref()
        .and_then(|parent| parent.subscription_details.as_ref())
        .and_then(|details| details.metadata.as_ref());
    invoice.billing_reason == BillingReason::SubscriptionUpdate
        && marker.is_some_and(|marker| {
            marker.ciproxy_payment_id == Some(work.id)
                && marker.ciproxy_quote_id == Some(work.quote.payment.id)
        })
}

impl BillingService {
    fn payment_invoice(
        &self,
        work: &PaymentWork,
        current: &Subscription,
    ) -> Result<Option<RetrievedInvoice>> {
        let invoice = if let Some(id) = &work.invoice_id {
            Some(self.stripe.invoice(id)?)
        } else if let Some(invoice) = current
            .latest_invoice
            .as_ref()
            .filter(|invoice| matching(work, invoice))
        {
            Some(self.stripe.invoice(&invoice.id)?)
        } else {
            let invoices = self.stripe.payment_invoices(
                &work.quote.source.customer_id,
                &work.quote.source.subscription_id,
            )?;
            let mut matching = invoices
                .into_iter()
                .filter(|invoice| matching(work, &invoice.invoice));
            let invoice = matching.next();
            if matching.next().is_some() {
                return Err(Error::StripeState);
            }
            invoice
        };
        if let Some(invoice) = &invoice
            && (invoice.customer != work.quote.source.customer_id
                || invoice.subscription_id() != Some(work.quote.source.subscription_id.as_str())
                || !matching(work, &invoice.invoice))
        {
            return Err(Error::Ownership);
        }
        Ok(invoice)
    }

    pub(super) fn inspect_payment(
        &self,
        session: &mut dyn BillingSession,
        work: &mut PaymentWork,
        current: &Subscription,
    ) -> Result<()> {
        if current.id != work.quote.source.subscription_id
            || current.customer != work.quote.source.customer_id
            || current.metadata.workspace_id != Some(work.workspace_id)
        {
            return Err(Error::Ownership);
        }
        if matches!(
            work.settlement,
            Settlement::Restored | Settlement::Abandoned
        ) {
            return self.save_payment(session, work);
        }
        if work.settlement == Settlement::Restoring {
            return self.restore_payment(session, work);
        }
        let record = session.record().internal()?;
        let active =
            owns_command(work, record.command.as_ref()) && !record.removal_cancel_requested;
        if (!active || (!terminal(current) && source(current)? != work.quote.source))
            && !work.superseded
        {
            work.superseded = true;
            if work.submitted_at.is_some() && work.settlement != Settlement::Consumed {
                work.settlement = Settlement::Unresolved;
            }
            self.save_payment(session, work)?;
        }
        if work.submitted_at.is_none() {
            if work.superseded || terminal(current) {
                self.restore_payment(session, work)?;
                if owns_command(work, session.record().internal()?.command.as_ref()) {
                    session
                        .commit(None, None, true, false, self.clock.now())
                        .internal()?;
                }
                return self.save_payment(session, work);
            }
            return Ok(());
        }
        let Some(record) = self.payment_invoice(work, current)? else {
            if !work.quote.upgrade
                && selection(current)? == work.quote.selection
                && current.pending_update.is_none()
            {
                work.settlement = Settlement::Consumed;
                work.audit = InvoiceAudit::NoInvoice;
            } else if self.clock.now() - work.started_at >= Duration::hours(24) || work.superseded {
                work.settlement = Settlement::Unresolved;
                if !work.recovery_alerted {
                    self.payment_alert(work, AlertReason::RecoveryPending, None);
                    work.recovery_alerted = true;
                }
            }
            return self.save_payment(session, work);
        };
        let invoice = record.invoice;
        work.invoice_id = Some(invoice.id.clone());
        if work.settlement == Settlement::Consumed && invoice.status != InvoiceStatus::Paid {
            self.payment_alert(work, AlertReason::RecoveryPending, None);
            return self.save_payment(session, work);
        }
        match invoice.status {
            InvoiceStatus::Paid => {
                work.settlement = Settlement::Consumed;
                self.save_payment(session, work)?;
                if let Some(amount) = invoice
                    .amount_due
                    .filter(|_| invoice.currency.as_deref() == Some("usd"))
                {
                    if amount == work.quote.today_cents {
                        work.audit = InvoiceAudit::Matched {
                            amount_cents: amount,
                        };
                    } else {
                        let alerted = matches!(work.audit,InvoiceAudit::Mismatch{amount_cents,alerted:true} if amount_cents == amount);
                        work.audit = InvoiceAudit::Mismatch {
                            amount_cents: amount,
                            alerted,
                        };
                        self.save_payment(session, work)?;
                        if !alerted {
                            self.payment_alert(work, AlertReason::InvoiceMismatch, Some(amount));
                            work.audit = InvoiceAudit::Mismatch {
                                amount_cents: amount,
                                alerted: true,
                            };
                        }
                    }
                }
            }
            InvoiceStatus::Void
                if invoice.amount_paid == Some(0) && work.settlement != Settlement::Consumed =>
            {
                work.settlement = Settlement::Restoring;
                self.save_payment(session, work)?;
                return self.restore_payment(session, work);
            }
            InvoiceStatus::Draft | InvoiceStatus::Open => work.settlement = Settlement::Pending,
            InvoiceStatus::Void | InvoiceStatus::Uncollectible => {
                work.settlement = Settlement::Unresolved;
                if !work.recovery_alerted {
                    self.payment_alert(work, AlertReason::RecoveryPending, None);
                    work.recovery_alerted = true;
                }
            }
        }
        self.save_payment(session, work)
    }
}
