//! One transition owner for per-event cancellation authority.

use ciproxy_store_interface::billing::collection::{
    AutomaticStop, CancellationDecision, CancellationState, CollectionDecisions,
};

use crate::stripe::wire::invoice::{BillingReason, CollectionMethod, Invoice, InvoiceStatus};
use crate::stripe::wire::types::{Status, Subscription};

use super::invoice_evidence::CollectionChange;

pub(super) enum CollectionObservation<'a> {
    Fresh {
        current: &'a Subscription,
        invoice: &'a Invoice,
        change: Option<CollectionChange>,
    },
    Subscription {
        current: &'a Subscription,
        invoice_id: &'a str,
    },
    Invoice(&'a Invoice),
}

/// Terminal decisions never regain cancellation authority, including on replay.
pub(super) fn merge(decisions: &mut CollectionDecisions, observation: CollectionObservation<'_>) {
    match observation {
        CollectionObservation::Fresh {
            current,
            invoice,
            change,
        } => {
            if let Some(change) = change {
                signed_change(decisions, change);
            }
            for decision in &mut decisions.decisions {
                if decision.state == CancellationState::Eligible
                    && !current_stop(current, invoice, &decision.stop)
                {
                    decision.state = CancellationState::Superseded;
                }
            }
        }
        CollectionObservation::Subscription {
            current,
            invoice_id,
        } => {
            for decision in &mut decisions.decisions {
                if decision.state == CancellationState::Eligible {
                    if matches!(current.status, Status::Canceled | Status::IncompleteExpired) {
                        decision.state = CancellationState::Completed;
                    } else if !subscription_matches(current, invoice_id, &decision.stop) {
                        decision.state = CancellationState::Superseded;
                    }
                }
            }
        }
        CollectionObservation::Invoice(invoice) => {
            for decision in &mut decisions.decisions {
                if decision.state == CancellationState::Eligible
                    && !invoice_matches(invoice, &decision.stop)
                {
                    decision.state = CancellationState::Superseded;
                }
            }
        }
    }
}

pub(super) fn eligible(decisions: &CollectionDecisions) -> Option<&AutomaticStop> {
    decisions
        .decisions
        .iter()
        .find(|decision| decision.state == CancellationState::Eligible)
        .map(|decision| &decision.stop)
}

fn signed_change(decisions: &mut CollectionDecisions, change: CollectionChange) {
    let at = match &change {
        CollectionChange::Automatic(stop) => stop.stopped_at,
        CollectionChange::Other(at) => *at,
    };
    if let CollectionChange::Automatic(stop) = &change
        && decisions
            .decisions
            .iter()
            .any(|decision| decision.stop.event_id == stop.event_id)
    {
        return;
    }
    let newer = decisions.changed_at.is_none_or(|old| match change {
        CollectionChange::Automatic(_) => at > old,
        CollectionChange::Other(_) => at >= old,
    });
    if newer {
        decisions.changed_at = Some(at);
        for decision in &mut decisions.decisions {
            if decision.state == CancellationState::Eligible {
                decision.state = CancellationState::Superseded;
            }
        }
    }
    if let CollectionChange::Automatic(stop) = change {
        decisions.decisions.push(CancellationDecision {
            stop,
            state: if newer {
                CancellationState::Eligible
            } else {
                CancellationState::Superseded
            },
        });
    }
}

fn current_stop(current: &Subscription, invoice: &Invoice, stop: &AutomaticStop) -> bool {
    subscription_matches(current, &invoice.id, stop) && invoice_matches(invoice, stop)
}

fn subscription_matches(current: &Subscription, invoice_id: &str, stop: &AutomaticStop) -> bool {
    current.collection_method == Some(CollectionMethod::ChargeAutomatically)
        && current.pause_collection.is_none()
        && current
            .latest_invoice
            .as_ref()
            .filter(|invoice| invoice.id == invoice_id)
            .is_none_or(|invoice| invoice_matches(invoice, stop))
}

fn invoice_matches(invoice: &Invoice, stop: &AutomaticStop) -> bool {
    invoice.billing_reason == BillingReason::SubscriptionCycle
        && invoice.status == InvoiceStatus::Open
        && invoice.collection_method == Some(CollectionMethod::ChargeAutomatically)
        && invoice.amount_remaining.is_some_and(|amount| amount > 0)
        && invoice.attempted == Some(true)
        && invoice.attempt_count == Some(stop.attempt_count)
        && invoice.auto_advance == Some(false)
        && invoice.next_payment_attempt.is_none()
}
