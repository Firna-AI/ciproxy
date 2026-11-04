//! Checkout confirmation, retry, and concurrent ownership contracts.

use chrono::{DateTime, Utc};
use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::{CheckoutOutcome, EventOutcome};
use ciproxy_billing::stripe::wire::types::{CheckoutStatus, PaymentStatus};
use ciproxy_store_interface::billing::state::Tier;

use super::database::Database;
use super::fake::{Fake, field};
use super::fixture::{
    access, at, read_service, seed_checkout, service, signature, state, subscription, wanted,
    workspace,
};

#[test]
fn checkout_lost_reply_and_concurrent_callers_keep_one_session_until_confirmation() {
    let mut database = Database::new(workspace());
    database.set_created_at("2026-10-31T12:00:00Z".parse().unwrap());
    let fake = Fake::new(subscription());
    let service = service(&database, &fake, at());
    let initial = service.status(access()).unwrap().state;
    assert_eq!(initial.tier, Tier::Free);
    assert_eq!(initial.customer_id, None);
    assert_eq!(
        initial.free_started_at,
        "2026-10-31T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    fake.state.lock().unwrap().lose_next_post = true;
    assert!(matches!(
        service.open_checkout(access(), wanted(Tier::Team, 2, 1)),
        Err(Error::StripeHttp { status: 503 })
    ));
    let first_command = database
        .store
        .billing(workspace())
        .unwrap()
        .command
        .unwrap();
    let lock = database
        .store
        .try_lock_billing(workspace())
        .unwrap()
        .unwrap();
    assert!(matches!(
        service.open_checkout(access(), wanted(Tier::Business, 0, 0)),
        Err(Error::Busy)
    ));
    drop(lock);
    assert_eq!(
        service
            .open_checkout(access(), wanted(Tier::Business, 0, 0))
            .unwrap(),
        CheckoutOutcome::Open {
            url: "https://checkout.stripe.test/cs_recorded".into()
        }
    );
    let post = fake.posts("/v1/checkout/sessions");
    assert_eq!(post.len(), 1);
    assert_eq!(field(&post[0], "payment_method_types[0]"), "card");
    assert_eq!(field(&post[0], "automatic_tax[enabled]"), "false");
    assert_eq!(field(&post[0], "line_items[0][price]"), "price_team");
    assert_eq!(field(&post[0], "line_items[1][quantity]"), "2");
    assert_eq!(field(&post[0], "line_items[2][quantity]"), "1");
    assert_eq!(
        post[0].key,
        Some(format!("ciproxy:checkout:{}", first_command.id))
    );
    {
        let mut remote = fake.state.lock().unwrap();
        let checkout = remote.checkout.as_mut().unwrap();
        checkout.status = CheckoutStatus::Complete;
        checkout.url = None;
    }
    assert_eq!(
        service
            .open_checkout(access(), wanted(Tier::Business, 0, 0))
            .unwrap(),
        CheckoutOutcome::Confirming
    );
    assert_eq!(state(&database).tier, Tier::Free);
    fake.state.lock().unwrap().checkout.as_mut().unwrap().status = CheckoutStatus::Expired;
    assert_eq!(
        service.reconcile_checkout(access(), "cs_recorded").unwrap(),
        CheckoutOutcome::Unfinished
    );
    assert_eq!(state(&database).tier, Tier::Free);
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    for status in [409, 429] {
        fake.state.lock().unwrap().reject_next_mutation = Some(status);
        let result = service.open_checkout(access(), wanted(Tier::Team, u32::MAX, 1));
        assert!(matches!(result, Err(Error::StripeHttp { status: actual }) if actual == status));
        assert!(
            database
                .store
                .billing(workspace())
                .unwrap()
                .command
                .is_some()
        );
    }
    fake.state.lock().unwrap().reject_next_mutation = Some(400);
    assert!(matches!(
        service.open_checkout(access(), wanted(Tier::Team, u32::MAX, 1)),
        Err(Error::StripeRejected { status: 400 })
    ));
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    assert!(matches!(
        service.open_checkout(access(), wanted(Tier::Business, 0, 0)),
        Ok(CheckoutOutcome::Open { .. })
    ));
    let posts = fake.posts("/v1/checkout/sessions");
    assert_eq!(
        field(posts.last().unwrap(), "line_items[0][price]"),
        "price_business"
    );
}

#[test]
fn return_page_and_duplicate_signed_event_confirm_once_without_resetting_periods() {
    let database = Database::new(workspace());
    let fake = Fake::new(subscription());
    seed_checkout(&database, &fake);
    let service = read_service(&database, &fake, at());
    let outcome = service.reconcile_checkout(access(), "cs_recorded").unwrap();
    assert!(matches!(outcome, CheckoutOutcome::Confirmed(_)));
    let paid = state(&database);
    assert_eq!(
        (paid.tier, paid.add_ons.start_blocks, paid.add_ons.machines),
        (Tier::Team, 2, 1)
    );
    assert_eq!(paid.paid_period_start, Some(at()));
    assert_eq!(
        paid.paid_period_end,
        Some("2026-12-04T12:00:00Z".parse().unwrap())
    );
    let raw = include_bytes!("fixtures/checkout_completed.json");
    assert_eq!(
        service.webhook(&signature(raw, at()), raw).unwrap(),
        EventOutcome::Applied
    );
    assert_eq!(
        service.webhook(&signature(raw, at()), raw).unwrap(),
        EventOutcome::Duplicate
    );
    assert_eq!(state(&database), paid);
    assert_eq!(fake.posts("/v1/checkout/sessions").len(), 0);
    fake.state
        .lock()
        .unwrap()
        .checkout
        .as_mut()
        .unwrap()
        .payment_status = PaymentStatus::Unpaid;
    assert_eq!(
        service.reconcile_checkout(access(), "cs_recorded").unwrap(),
        CheckoutOutcome::Confirming
    );
    assert_eq!(state(&database), paid);
}
