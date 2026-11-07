//! Invoice identity and collection provenance from verified signed events.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::collection::AutomaticStop;

use crate::error::{Error, Result};
use crate::stripe::wire::invoice::{BillingReason, CollectionMethod, InvoiceStatus};
use crate::stripe::wire::types::Subscription;
use crate::webhook::{Event, Payload};

pub(super) struct InvoiceEvidence {
    pub(super) invoice_id: String,
    pub(super) failed_at: Option<DateTime<Utc>>,
    collection_change: Option<SignedCollectionChange>,
}

struct SignedCollectionChange {
    customer: Option<String>,
    subscription: Option<String>,
    at: DateTime<Utc>,
    automatic_stop: Option<AutomaticStop>,
}

pub(super) enum CollectionChange {
    Automatic(AutomaticStop),
    Other(DateTime<Utc>),
}

impl InvoiceEvidence {
    pub(super) fn from_event(event: &Event) -> Result<Option<Self>> {
        if !matches!(
            event.payload,
            Payload::Invoice { .. }
                | Payload::InvoiceFailed { .. }
                | Payload::InvoiceUpdated { .. }
        ) {
            return Ok(None);
        }
        let at = DateTime::from_timestamp(event.created, 0).ok_or(Error::StripeState)?;
        let evidence = match &event.payload {
            Payload::Invoice { data: object } | Payload::InvoiceFailed { data: object } => Self {
                invoice_id: object.object.id.clone(),
                failed_at: matches!(event.payload, Payload::InvoiceFailed { .. }).then_some(at),
                collection_change: None,
            },
            Payload::InvoiceUpdated { data: update } => {
                let invoice = &update.object;
                let changed = update
                    .previous_attributes
                    .as_ref()
                    .and_then(|previous| previous.auto_advance.zip(invoice.auto_advance));
                let automatic = event
                    .request
                    .as_ref()
                    .is_some_and(|request| request.id.is_none())
                    && changed == Some((true, false))
                    && invoice.status == Some(InvoiceStatus::Open)
                    && invoice.billing_reason == Some(BillingReason::SubscriptionCycle)
                    && invoice.collection_method == Some(CollectionMethod::ChargeAutomatically)
                    && invoice.attempted == Some(true)
                    && invoice.amount_remaining.is_some_and(|amount| amount > 0)
                    && invoice.next_payment_attempt.is_none();
                Self {
                    invoice_id: invoice.id.clone(),
                    failed_at: None,
                    collection_change: changed.filter(|(before, after)| before != after).map(
                        |_| SignedCollectionChange {
                            customer: invoice.customer.clone(),
                            subscription: invoice
                                .parent
                                .as_ref()
                                .and_then(|parent| parent.subscription_details.as_ref())
                                .map(|details| details.subscription.clone()),
                            at,
                            automatic_stop: invoice
                                .attempt_count
                                .filter(|count| *count > 0)
                                .filter(|_| automatic)
                                .map(|attempt_count| AutomaticStop {
                                    event_id: event.id.clone(),
                                    stopped_at: at,
                                    attempt_count,
                                }),
                        },
                    ),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(evidence))
    }

    pub(super) fn collection_change(&self, current: &Subscription) -> Option<CollectionChange> {
        let change = self.collection_change.as_ref()?;
        if change.customer.as_deref() != Some(&current.customer)
            || change.subscription.as_deref() != Some(&current.id)
        {
            return None;
        }
        Some(match &change.automatic_stop {
            Some(stop) => CollectionChange::Automatic(stop.clone()),
            None => CollectionChange::Other(change.at),
        })
    }
}
