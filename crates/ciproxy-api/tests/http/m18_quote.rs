//! Stored authoritative review inputs for HTTP payment routing tests.

use chrono::{DateTime, Duration};
use ciproxy_store_interface::billing::quotes::{
    ChangeItem, QuotePayment, QuoteSource, StoredQuote,
};
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use ciproxy_store_interface::metering::history::Period;
use uuid::Uuid;

use super::m18_stripe::Stripe;
use super::m18_support::{Pages, at};

pub(super) fn quote(pages: &Pages, stripe: &Stripe) -> Uuid {
    let current = stripe.subscription.lock().unwrap().clone();
    let id = Uuid::new_v4();
    let mut items = current
        .items
        .data
        .iter()
        .map(|item| ChangeItem::Set {
            id: Some(item.id.clone()),
            price_id: item.price.id.clone(),
            quantity: item.quantity,
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| match item {
        ChangeItem::Set { id, .. } => id.clone(),
        ChangeItem::Remove { id } => Some(id.clone()),
    });
    let quote = StoredQuote {
        payment: QuotePayment {
            id,
            offset: None,
            starting_balance_cents: Some(0),
            proration_at: at("2026-11-06T12:00:00Z"),
            items: Vec::new(),
        },
        selection: Selection {
            tier: Tier::Business,
            add_ons: AddOns::default(),
        },
        upgrade: true,
        source: QuoteSource {
            subscription_id: current.id,
            customer_id: current.customer,
            selection: Selection {
                tier: Tier::Team,
                add_ons: AddOns {
                    start_blocks: 1,
                    machines: 2,
                },
            },
            period: Period {
                starts_at: DateTime::from_timestamp(current.items.data[0].current_period_start, 0)
                    .unwrap(),
                ends_at: DateTime::from_timestamp(current.items.data[0].current_period_end, 0)
                    .unwrap(),
            },
            items,
            anchor: current.billing_cycle_anchor,
            cancel_at_period_end: current.cancel_at_period_end,
            cancel_at: current.cancel_at,
            anchor_day: current
                .billing_cycle_anchor_config
                .map(|anchor| anchor.day_of_month),
        },
        monthly_cents: 29900,
        today_cents: 22000,
        billing_day: 1,
        expires_at: at("2026-11-06T12:00:00Z") + Duration::minutes(30),
        command_id: None,
    };
    pages
        .fixture
        .database
        .store
        .try_lock_billing(pages.fixture.workspace)
        .unwrap()
        .unwrap()
        .save_quote(&quote)
        .unwrap();
    id
}
