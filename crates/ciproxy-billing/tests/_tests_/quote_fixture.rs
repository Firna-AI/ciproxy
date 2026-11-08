//! Stored reviews for tests of billing workflows below the review boundary.

use chrono::{DateTime, Duration, Utc};
use ciproxy_store_interface::billing::commands::CommandKind;
use ciproxy_store_interface::billing::quotes::{
    ChangeItem, QuotePayment, QuoteSource, StoredQuote,
};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::metering::history::Period;
use uuid::Uuid;

use super::database::Database;
use super::fake::Fake;
use super::fixture::{wanted, workspace};

pub(super) fn quote(database: &Database, fake: &Fake, now: DateTime<Utc>) -> Uuid {
    let mut session = database
        .store
        .try_lock_billing(workspace())
        .unwrap()
        .unwrap();
    let mut command = session.record().unwrap().command;
    if let Some(command) = &command
        && let CommandKind::Change {
            quote: Some(quote), ..
        } = &command.kind
    {
        return quote.id;
    }
    let current = fake.state.lock().unwrap().subscription.clone();
    let id = Uuid::new_v4();
    let payment = QuotePayment {
        id,
        offset: None,
        starting_balance_cents: Some(0),
        proration_at: now,
        items: vec![
            ChangeItem::Remove {
                id: "si_tier".into(),
            },
            ChangeItem::Set {
                id: Some("si_starts".into()),
                price_id: "price_starts".into(),
                quantity: 2,
            },
            ChangeItem::Set {
                id: Some("si_machine".into()),
                price_id: "price_machine".into(),
                quantity: 1,
            },
            ChangeItem::Set {
                id: None,
                price_id: "price_business".into(),
                quantity: 1,
            },
        ],
    };
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
    let command_id = if let Some(command) = &mut command
        && let CommandKind::Change { quote, .. } = &mut command.kind
    {
        *quote = Some(payment.clone());
        session.save_command(command).unwrap();
        Some(command.id)
    } else {
        None
    };
    session
        .save_quote(&StoredQuote {
            payment,
            selection: wanted(Tier::Business, 2, 1),
            upgrade: true,
            source: QuoteSource {
                subscription_id: current.id,
                customer_id: current.customer,
                selection: wanted(Tier::Team, 2, 1),
                period: Period {
                    starts_at: DateTime::from_timestamp(
                        current.items.data[0].current_period_start,
                        0,
                    )
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
            monthly_cents: 33900,
            today_cents: 25000,
            billing_day: 4,
            expires_at: now + Duration::minutes(30),
            command_id,
        })
        .unwrap();
    id
}
