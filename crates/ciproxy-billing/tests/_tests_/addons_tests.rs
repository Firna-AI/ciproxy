//! Quantity saves preserve the reconciled paid tier without choosing one.

use ciproxy_billing::catalog::PriceCode;
use ciproxy_billing::service::view::ChangeOutcome;
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{AddOns, Tier};

use super::database::Database;
use super::fake::{Fake, field};
use super::fake_subscriptions::{ChangeMode, ItemChange, SubscriptionOutcome, SubscriptionRequest};
use super::fixture::{
    access, at, command, seed_paid, service, state, subscription, wanted, workspace,
};

#[test]
fn addon_save_keeps_the_reconciled_business_tier_and_scheduled_period_end() {
    let database = Database::new(workspace());
    seed_paid(&database);
    {
        let mut saved = database
            .store
            .try_lock_billing(workspace())
            .unwrap()
            .unwrap();
        saved
            .save_command(&BillingCommand {
                id: command(),
                started_at: at(),
                payment_expires_at: None,
                kind: CommandKind::Change {
                    selection: wanted(Tier::Business, 2, 1),
                    upgrade: true,
                    quote: None,
                },
            })
            .unwrap();
    }
    let mut current = subscription();
    current.items.data[0].price.lookup_key = PriceCode::Business;
    current.items.data[0].price.id = "price_business".into();
    current.cancel_at_period_end = true;
    let mut updated = current.clone();
    updated.items.data.remove(1);
    updated.items.data[1].quantity = 2;
    let fake = Fake::new(current);
    fake.expect_subscription(
        SubscriptionRequest::Change(
            ChangeMode::Prorated,
            vec![
                ItemChange::Update("si_tier", "price_business", 1),
                ItemChange::Remove("si_starts"),
                ItemChange::Update("si_machine", "price_machine", 2),
            ],
        ),
        SubscriptionOutcome::respond(updated),
    );
    let billing = service(&database, &fake, at());
    assert_eq!(state(&database).tier, Tier::Team);
    let result = billing.change_add_ons(
        access(),
        AddOns {
            start_blocks: 0,
            machines: 2,
        },
    );
    assert!(
        matches!(result, Ok(ChangeOutcome::Applied(_))),
        "expected applied quantities, got {result:?}"
    );
    let saved = state(&database);
    assert_eq!(saved.tier, Tier::Business);
    assert_eq!(
        saved.add_ons,
        AddOns {
            start_blocks: 0,
            machines: 2
        }
    );
    assert_eq!(saved.paid_period_start, Some(at()));
    assert_eq!(
        saved.paid_period_end,
        Some("2026-12-04T12:00:00Z".parse().unwrap())
    );
    assert_eq!(
        saved.cancel_at,
        Some("2026-12-04T12:00:00Z".parse().unwrap())
    );
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    let requests = fake.posts("/v1/subscriptions/sub_recorded");
    assert_eq!(requests.len(), 1);
    assert_eq!(field(&requests[0], "items[0][price]"), "price_business");
    assert_eq!(field(&requests[0], "items[1][deleted]"), "true");
    assert_eq!(field(&requests[0], "items[2][quantity]"), "2");
    assert_eq!(
        field(&requests[0], "proration_behavior"),
        "create_prorations"
    );
}
