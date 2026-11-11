//! Durable quote snapshots and best-effort payment evidence recovery.

use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::payments::{InvoiceAudit, PaymentWork, Settlement};
use ciproxy_store_interface::billing::quotes::StoredQuote;
use ciproxy_store_interface::billing::store::BillingSession;
use uuid::Uuid;

use crate::alerts::{AlertReason, BillingAlert};
use crate::error::{Result, ResultExt};
use crate::stripe::wire::types::Subscription;

use super::runtime::BillingService;

pub(super) fn new_work(
    workspace: Uuid,
    quote: &StoredQuote,
    command: &BillingCommand,
) -> PaymentWork {
    PaymentWork {
        id: command.id,
        workspace_id: workspace,
        quote: quote.clone(),
        started_at: command.started_at,
        marked: false,
        retiring: None,
        retired: Vec::new(),
        submitted_at: None,
        invoice_id: None,
        settlement: Settlement::Preparing,
        restoration: None,
        audit: InvoiceAudit::Waiting,
        pending: true,
        recovery_alerted: false,
        rejection: None,
        superseded: false,
    }
}

pub(super) fn owns_command(work: &PaymentWork, command: Option<&BillingCommand>) -> bool {
    command.is_some_and(|command| command.id == work.id
        && matches!(&command.kind,CommandKind::Change{quote:Some(quote),..} if quote.id == work.quote.payment.id))
}

pub(super) fn money_held(work: &PaymentWork) -> bool {
    (!work.retired.is_empty() || work.retiring.is_some())
        && !matches!(
            work.settlement,
            Settlement::Consumed | Settlement::Restored | Settlement::Abandoned
        )
}

impl BillingService {
    pub(super) fn save_payment(
        &self,
        session: &mut dyn BillingSession,
        work: &mut PaymentWork,
    ) -> Result<()> {
        let financial_done = matches!(
            work.settlement,
            Settlement::Consumed | Settlement::Restored | Settlement::Abandoned
        );
        let audit_done = matches!(
            work.audit,
            InvoiceAudit::Matched { .. }
                | InvoiceAudit::NoInvoice
                | InvoiceAudit::Mismatch { alerted: true, .. }
        );
        let command = session.record().internal()?.command;
        work.pending = !financial_done || !audit_done || owns_command(work, command.as_ref());
        session.save_payment(work).internal()
    }

    pub(super) fn payment_alert(
        &self,
        work: &PaymentWork,
        reason: AlertReason,
        actual_cents: Option<u64>,
    ) {
        self.alerts.emit(&BillingAlert {
            workspace_id: work.workspace_id,
            quote_id: work.quote.payment.id,
            command_id: work.id,
            invoice_id: work.invoice_id.clone(),
            expected_cents: work.quote.today_cents,
            actual_cents,
            reason,
        });
    }

    pub(super) fn observe_payments(
        &self,
        session: &mut dyn BillingSession,
        current: &Subscription,
    ) {
        let works = match session.payments() {
            Ok(works) => works,
            Err(_) => {
                tracing::error!(target:"ciproxy_billing::operator", code="payment_work_read_failed", "billing recovery will retry");
                return;
            }
        };
        for mut work in works
            .into_iter()
            .filter(|work| work.quote.source.subscription_id == current.id)
        {
            if self.inspect_payment(session, &mut work, current).is_err() {
                self.payment_alert(&work, AlertReason::RecoveryPending, None);
            }
        }
    }
}
