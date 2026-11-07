//! Controlled Stripe IO with production BillingService and PostgreSQL persistence.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ciproxy_billing::catalog::PriceCode;
use ciproxy_billing::stripe::api::{StripeApi, StripeApiMock};
use ciproxy_billing::stripe::wire::types::{
    CheckoutSession, CheckoutStatus, Item, Metadata, PaymentStatus, PendingUpdate, Price,
    Subscription,
};
use ciproxy_store_interface::billing::state::{Selection, Tier};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

pub(super) enum Scenario {
    Checkout,
    Expired,
    Paid,
    AddOns,
}

pub(super) struct Stripe {
    pub(super) session: Mutex<CheckoutSession>,
    pub(super) subscription: Mutex<Subscription>,
    pub(super) checkouts: AtomicUsize,
    pub(super) selections: Mutex<Vec<Selection>>,
    pub(super) payment_required: AtomicBool,
}

impl Stripe {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            session: Mutex::new(CheckoutSession {
                id: "cs_m18".into(),
                status: CheckoutStatus::Open,
                payment_status: PaymentStatus::Unpaid,
                url: Some("https://checkout.stripe.test/m18".into()),
                customer: None,
                subscription: None,
                metadata: Metadata::default(),
            }),
            subscription: Mutex::new(
                serde_json::from_str(include_str!(
                    "../../../ciproxy-billing/tests/_tests_/fixtures/subscription.json"
                ))
                .unwrap(),
            ),
            checkouts: AtomicUsize::new(0),
            selections: Mutex::new(Vec::new()),
            payment_required: AtomicBool::new(false),
        })
    }

    pub(super) fn bind(&self, workspace: Uuid) {
        let mut subscription = self.subscription.lock().unwrap();
        subscription.id = "sub_m18".into();
        subscription.customer = "cus_m18".into();
        subscription.metadata.workspace_id = Some(workspace);
        subscription.metadata.command_id = None;
        subscription.items.data.truncate(1);
        for item in &mut subscription.items.data {
            item.current_period_start = super::m18_support::at("2026-11-01T00:00:00Z").timestamp();
            item.current_period_end = super::m18_support::at("2026-12-01T00:00:00Z").timestamp();
        }
    }

    pub(super) fn provider(self: &Arc<Self>) -> Arc<dyn StripeApi> {
        Arc::new(Unimock::new(()))
    }

    pub(super) fn provider_for(self: &Arc<Self>, scenario: Scenario) -> Arc<dyn StripeApi> {
        let checkout = self.clone();
        let read = self.clone();
        let subscription = self.clone();
        let change = self.clone();
        let cancel = self.clone();
        let checkout = StripeApiMock::checkout
            .each_call(matching!())
            .answers_arc(Arc::new(move |_, workspace, _, command| {
                checkout.checkouts.fetch_add(1, Ordering::SeqCst);
                let mut session = checkout.session.lock().unwrap();
                session.metadata = Metadata {
                    workspace_id: Some(workspace),
                    command_id: Some(command.id),
                };
                checkout.subscription.lock().unwrap().metadata = session.metadata.clone();
                Ok(session.clone())
            }));
        let read = StripeApiMock::checkout_session
            .each_call(matching!())
            .answers_arc(Arc::new(move |_, id| {
                let mut value = read.session.lock().unwrap().clone();
                if id == "cs_foreign" {
                    value.metadata.workspace_id = Some(Uuid::nil());
                }
                Ok(value)
            }));
        let subscription = StripeApiMock::subscription
            .each_call(matching!())
            .answers_arc(Arc::new(move |_, _| {
                Ok(subscription.subscription.lock().unwrap().clone())
            }));
        let change = StripeApiMock::change
            .each_call(matching!())
            .answers_arc(Arc::new(move |_, current, selection, _| {
                change.selections.lock().unwrap().push(selection);
                let mut updated = current.clone();
                if change.payment_required.load(Ordering::SeqCst) {
                    updated.pending_update = Some(PendingUpdate {
                        expires_at: super::m18_support::at("2026-11-07T12:00:00Z").timestamp(),
                    });
                    let invoice = updated.latest_invoice.as_mut().unwrap();
                    invoice.billing_reason =
                        ciproxy_billing::stripe::wire::invoice::BillingReason::SubscriptionUpdate;
                    invoice.status = ciproxy_billing::stripe::wire::invoice::InvoiceStatus::Open;
                } else {
                    replace(&mut updated, selection);
                }
                *change.subscription.lock().unwrap() = updated.clone();
                Ok(updated)
            }));
        let cancel = StripeApiMock::cancel
            .each_call(matching!())
            .answers_arc(Arc::new(move |_, _, _| {
                let mut subscription = cancel.subscription.lock().unwrap();
                subscription.cancel_at_period_end = true;
                subscription.cancel_at = Some(subscription.items.data[0].current_period_end);
                Ok(subscription.clone())
            }));
        let portal = StripeApiMock::portal
            .each_call(matching!())
            .answers(&|_, _, _| Ok("https://billing.stripe.test/m18".into()));
        match scenario {
            Scenario::Checkout => Arc::new(Unimock::new((checkout, read, subscription))),
            Scenario::Expired => Arc::new(Unimock::new((checkout, read))),
            Scenario::Paid => Arc::new(Unimock::new((subscription, change, cancel, portal))),
            Scenario::AddOns => Arc::new(Unimock::new((subscription, change))),
        }
    }

    pub(super) fn complete(&self, paid: bool) {
        let mut session = self.session.lock().unwrap();
        session.status = CheckoutStatus::Complete;
        session.url = None;
        session.payment_status = if paid {
            PaymentStatus::Paid
        } else {
            PaymentStatus::Unpaid
        };
        session.customer = Some("cus_m18".into());
        session.subscription = Some("sub_m18".into());
    }
}

fn replace(subscription: &mut Subscription, selection: Selection) {
    let period = &subscription.items.data[0];
    let codes = [
        (
            if selection.tier == Tier::Business {
                PriceCode::Business
            } else {
                PriceCode::Team
            },
            1,
        ),
        (PriceCode::Starts, selection.add_ons.start_blocks),
        (PriceCode::Machine, selection.add_ons.machines),
    ];
    let items = codes
        .into_iter()
        .filter(|(_, quantity)| *quantity > 0)
        .enumerate()
        .map(|(index, (code, quantity))| Item {
            id: format!("si_{index}"),
            price: Price {
                id: format!("price_{index}"),
                lookup_key: code,
            },
            quantity,
            current_period_start: period.current_period_start,
            current_period_end: period.current_period_end,
        })
        .collect();
    subscription.items.data = items;
}
