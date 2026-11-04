//! Current pending changes protect their command and their payment context.

use ciproxy_billing::catalog::PriceCode;
use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::ChangeOutcome;
use ciproxy_store_interface::billing::state::Tier;

use super::database::Database;
use super::expiry_support::{lost_upgrade, pending, started};
use super::fake::Fake;
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::fixture::{
    access, read_service, seed_paid, service, state, subscription, wanted, workspace,
};
use super::subscription_requests::business_upgrade;

#[test]
fn current_pending_update_survives_age_then_applied_selection_wins_over_age() {
    let (database, fake) = lost_upgrade();
    let saved = database
        .store
        .billing(workspace())
        .unwrap()
        .command
        .unwrap();
    let later = "2026-12-06T11:00:00Z".parse().unwrap();
    let billing = read_service(&database, &fake, later);
    assert!(matches!(
        billing.change(access(), wanted(Tier::Team, 0, 0)),
        Err(Error::PaymentConfirming)
    ));
    assert_eq!(
        database
            .store
            .billing(workspace())
            .unwrap()
            .command
            .map(|command| command.id),
        Some(saved.id)
    );
    assert_eq!(
        billing
            .change(access(), wanted(Tier::Business, 2, 1))
            .unwrap(),
        ChangeOutcome::PaymentRequired {
            payment_url: Some("https://invoice.stripe.test/upgrade".into())
        }
    );
    let mut applied = subscription();
    applied.items.data[0].price.lookup_key = PriceCode::Business;
    applied.items.data[0].price.id = "price_business".into();
    fake.state.lock().unwrap().subscription = applied;
    assert!(matches!(
        billing.change(access(), wanted(Tier::Business, 2, 1)),
        Ok(ChangeOutcome::Applied(_))
    ));
    assert_eq!(state(&database).tier, Tier::Business);
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 1);
}

#[test]
fn canceling_a_pending_upgrade_keeps_other_selections_out_of_its_payment_context() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    let billing = service(&database, &fake, started());
    fake.expect_subscription(business_upgrade(), SubscriptionOutcome::respond(pending()));
    assert!(matches!(
        billing.change(access(), wanted(Tier::Business, 2, 1)),
        Ok(ChangeOutcome::PaymentRequired { .. })
    ));
    let mut canceling = pending();
    canceling.cancel_at_period_end = true;
    fake.expect_subscription(
        SubscriptionRequest::CancelAtPeriodEnd,
        SubscriptionOutcome::respond(canceling),
    );
    billing.move_to_free(access()).unwrap();
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    let result = billing.change(access(), wanted(Tier::Team, 0, 0));
    assert!(matches!(result, Err(Error::PaymentConfirming)));
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 2);
}
