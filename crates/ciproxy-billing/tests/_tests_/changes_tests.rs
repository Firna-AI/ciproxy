//! Proration, payment-gated upgrades, and exact retry recovery.

use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::ChangeOutcome;
use ciproxy_billing::stripe::wire::invoice::{BillingReason, InvoiceStatus};
use ciproxy_billing::stripe::wire::types::PendingUpdate;
use ciproxy_store_interface::billing::state::{AddOns, SubscriptionState, Tier};

use super::database::Database;
use super::fake::{Fake, field};
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::fixture::{access, at, seed_paid, service, state, subscription, wanted, workspace};
use super::subscription_requests::{
    add_one_start_block, business_upgrade, rejected_large_selection, remove_all_addons,
    remove_remaining_machine, remove_starts,
};

#[test]
fn upgrade_waits_for_payment_then_applies_and_expired_payment_can_retry() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    let service = service(&database, &fake, at());
    let mut pending = subscription();
    pending.pending_update = Some(PendingUpdate {
        expires_at: 1793793700,
    });
    let invoice = pending.latest_invoice.as_mut().unwrap();
    invoice.billing_reason = BillingReason::SubscriptionUpdate;
    invoice.status = InvoiceStatus::Open;
    fake.expect_subscription(business_upgrade(), SubscriptionOutcome::respond(pending));
    assert_eq!(
        service
            .change(access(), wanted(Tier::Business, 2, 1))
            .unwrap(),
        ChangeOutcome::PaymentRequired {
            payment_url: Some("https://invoice.stripe.test/pay".into())
        }
    );
    assert_eq!(
        (
            state(&database).tier,
            state(&database).status,
            state(&database).failed_at
        ),
        (Tier::Team, SubscriptionState::Active, None)
    );
    let post = fake.posts("/v1/subscriptions/sub_recorded");
    assert_eq!(field(&post[0], "proration_behavior"), "always_invoice");
    assert_eq!(field(&post[0], "payment_behavior"), "pending_if_incomplete");
    assert_eq!(
        service
            .change(access(), wanted(Tier::Business, 2, 1))
            .unwrap(),
        ChangeOutcome::PaymentRequired {
            payment_url: Some("https://invoice.stripe.test/pay".into())
        }
    );
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 1);
    let expired_at = "2026-11-05T12:00:00Z".parse().unwrap();
    let retry = super::fixture::service(&database, &fake, expired_at);
    let mut expired = subscription();
    expired.latest_invoice.as_mut().unwrap().status = InvoiceStatus::Void;
    let mut business = subscription();
    business.items.data[0].price.id = "price_business".into();
    business.items.data[0].price.lookup_key = ciproxy_billing::catalog::PriceCode::Business;
    {
        let mut remote = fake.state.lock().unwrap();
        remote.subscription = expired;
        remote.expect_subscription(business_upgrade(), SubscriptionOutcome::respond(business));
    }
    let outcome = retry
        .change(access(), wanted(Tier::Business, 2, 1))
        .unwrap();
    assert!(matches!(outcome, ChangeOutcome::Applied(_)));
    assert_eq!(state(&database).tier, Tier::Business);
    assert_eq!(state(&database).paid_period_start, Some(at()));
    assert_eq!(
        state(&database).paid_period_end,
        Some("2026-12-04T12:00:00Z".parse().unwrap())
    );
    let posts = fake.posts("/v1/subscriptions/sub_recorded");
    assert_ne!(posts[0].key, posts[1].key);
}

#[test]
fn saved_addon_removal_prorates_at_once_and_lost_reply_does_not_replay_a_changed_form() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    let service = service(&database, &fake, at());
    let mut without_starts = subscription();
    without_starts.items.data.remove(1);
    {
        let mut remote = fake.state.lock().unwrap();
        remote.expect_subscription(
            remove_starts(),
            SubscriptionOutcome::respond(without_starts),
        );
        remote.lose_next_post = true;
    }
    assert!(matches!(
        service.change_add_ons(
            access(),
            AddOns {
                start_blocks: 0,
                machines: 1
            }
        ),
        Err(Error::StripeHttp { status: 503 })
    ));
    assert_eq!(state(&database).add_ons.start_blocks, 2);
    let mut second_change = subscription();
    second_change.items.data.truncate(1);
    fake.expect_subscription(
        remove_remaining_machine(),
        SubscriptionOutcome::respond(second_change),
    );
    assert!(matches!(
        service.change_add_ons(
            access(),
            AddOns {
                start_blocks: 0,
                machines: 0
            }
        ),
        Ok(ChangeOutcome::Applied(_))
    ));
    assert_eq!(state(&database).add_ons.start_blocks, 0);
    let posts = fake.posts("/v1/subscriptions/sub_recorded");
    assert_eq!(posts.len(), 2);
    assert_eq!(field(&posts[0], "proration_behavior"), "create_prorations");
    assert_eq!(field(&posts[0], "items[1][deleted]"), "true");
    assert_eq!(field(&posts[1], "items[1][deleted]"), "true");
    assert_eq!(state(&database).add_ons.machines, 0);
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    assert!(matches!(
        service.change_add_ons(
            access(),
            AddOns {
                start_blocks: 0,
                machines: 0
            }
        ),
        Err(Error::CurrentTier)
    ));
    for status in [409, 429] {
        fake.expect_subscription(
            rejected_large_selection(),
            SubscriptionOutcome::Reject(status),
        );
        let result = service.change_add_ons(
            access(),
            AddOns {
                start_blocks: u32::MAX,
                machines: 1,
            },
        );
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
    fake.expect_subscription(rejected_large_selection(), SubscriptionOutcome::Reject(400));
    assert!(matches!(
        service.change_add_ons(
            access(),
            AddOns {
                start_blocks: u32::MAX,
                machines: 1
            }
        ),
        Err(Error::StripeRejected { status: 400 })
    ));
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    let mut corrected = subscription();
    corrected.items.data.truncate(2);
    corrected.items.data[1].quantity = 1;
    fake.expect_subscription(
        add_one_start_block(),
        SubscriptionOutcome::respond(corrected),
    );
    assert!(matches!(
        service.change_add_ons(
            access(),
            AddOns {
                start_blocks: 1,
                machines: 0
            }
        ),
        Ok(ChangeOutcome::Applied(_))
    ));
}

#[test]
fn cancel_lost_reply_clears_claim_and_portal_undo_allows_later_paid_change() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    let service = service(&database, &fake, at());
    fake.expect_subscription(
        SubscriptionRequest::CancelAtPeriodEnd,
        SubscriptionOutcome::Reject(400),
    );
    assert!(matches!(
        service.move_to_free(access()),
        Err(Error::StripeRejected { status: 400 })
    ));
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    let mut canceled = subscription();
    canceled.cancel_at_period_end = true;
    {
        let mut remote = fake.state.lock().unwrap();
        remote.expect_subscription(
            SubscriptionRequest::CancelAtPeriodEnd,
            SubscriptionOutcome::respond(canceled),
        );
        remote.lose_next_post = true;
    }
    assert!(matches!(
        service.move_to_free(access()),
        Err(Error::StripeHttp { status: 503 })
    ));
    service.move_to_free(access()).unwrap();
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    assert_eq!(
        (state(&database).tier, state(&database).add_ons.start_blocks),
        (Tier::Team, 2)
    );
    assert_eq!(
        state(&database).cancel_at,
        Some("2026-12-04T12:00:00Z".parse().unwrap())
    );
    let mut no_addons = subscription();
    no_addons.items.data.truncate(1);
    {
        let mut remote = fake.state.lock().unwrap();
        remote.subscription = subscription();
        remote.expect_subscription(remove_all_addons(), SubscriptionOutcome::respond(no_addons));
    }
    assert!(matches!(
        service
            .change_add_ons(
                access(),
                AddOns {
                    start_blocks: 0,
                    machines: 0
                }
            )
            .unwrap(),
        ChangeOutcome::Applied(_)
    ));
    assert_eq!(state(&database).cancel_at, None);
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 3);
}

#[test]
fn successful_pending_upgrade_after_deadline_syncs_without_second_charge() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    let billing = service(&database, &fake, at());
    let mut pending = subscription();
    pending.pending_update = Some(PendingUpdate {
        expires_at: 1793793700,
    });
    fake.expect_subscription(business_upgrade(), SubscriptionOutcome::respond(pending));
    assert!(matches!(
        billing.change(access(), wanted(Tier::Business, 2, 1)),
        Ok(ChangeOutcome::PaymentRequired { .. })
    ));
    let mut paid = subscription();
    paid.items.data[0].price.lookup_key = ciproxy_billing::catalog::PriceCode::Business;
    paid.items.data[0].price.id = "price_business".into();
    fake.state.lock().unwrap().subscription = paid;
    let later =
        super::fixture::read_service(&database, &fake, "2026-11-05T12:00:00Z".parse().unwrap());
    let outcome = later.change(access(), wanted(Tier::Business, 2, 1));
    assert!(matches!(outcome, Ok(ChangeOutcome::Applied(_))));
    assert_eq!(state(&database).tier, Tier::Business);
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 1);
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
}
