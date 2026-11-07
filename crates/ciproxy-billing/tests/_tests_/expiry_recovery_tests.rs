//! Observed and unobserved pending upgrades recover after a lost reply.

use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::{ChangeOutcome, CheckoutOutcome, EventOutcome};
use ciproxy_billing::stripe::wire::types::{CheckoutSession, CheckoutStatus, PaymentStatus};
use ciproxy_store_interface::billing::state::{AddOns, SubscriptionState, Tier};

use super::expiry_support::{event, lost_upgrade, renewal, started};
use super::fake_subscriptions::SubscriptionOutcome;
use super::fixture::{access, read_service, service, signature, state, wanted, workspace};
use super::subscription_requests::{business_upgrade, remove_all_addons};

#[derive(Debug)]
enum Observer {
    Webhook,
    ConflictingChange,
    CheckoutReturn,
}

#[test]
fn every_observer_saves_lost_upgrade_expiry_before_a_renewal_replaces_latest_invoice() {
    for observer in [
        Observer::Webhook,
        Observer::ConflictingChange,
        Observer::CheckoutReturn,
    ] {
        let (database, fake) = lost_upgrade();
        let reader = read_service(&database, &fake, started());
        let observed = match observer {
            Observer::Webhook => {
                let raw = event(
                    "evt_pending_seen",
                    "customer.subscription.updated",
                    started(),
                );
                matches!(
                    reader.webhook(&signature(&raw, started()), &raw),
                    Ok(EventOutcome::Applied)
                )
            }
            Observer::ConflictingChange => {
                matches!(
                    reader.change_add_ons(
                        access(),
                        AddOns {
                            start_blocks: 0,
                            machines: 0
                        }
                    ),
                    Err(Error::PaymentConfirming)
                )
            }
            Observer::CheckoutReturn => {
                let mut checkout: CheckoutSession =
                    serde_json::from_str(include_str!("fixtures/checkout.json")).unwrap();
                checkout.status = CheckoutStatus::Complete;
                checkout.payment_status = PaymentStatus::Paid;
                checkout.customer = Some("cus_recorded".into());
                checkout.subscription = Some("sub_recorded".into());
                fake.state.lock().unwrap().checkout = Some(checkout);
                matches!(
                    reader.reconcile_checkout(access(), "cs_recorded"),
                    Ok(CheckoutOutcome::Confirmed(_))
                )
            }
        };
        assert!(observed, "{observer:?}");
        assert_eq!(
            database
                .store
                .billing(workspace())
                .unwrap()
                .command
                .unwrap()
                .payment_expires_at,
            Some("2026-12-04T12:00:00Z".parse().unwrap()),
            "{observer:?}"
        );
        fake.state.lock().unwrap().subscription = renewal();
        let expired = "2026-12-04T12:00:00Z".parse().unwrap();
        let billing = service(&database, &fake, expired);
        let raw = event(
            "evt_expired_at_renewal",
            "customer.subscription.pending_update_expired",
            expired,
        );
        assert_eq!(
            billing.webhook(&signature(&raw, expired), &raw).unwrap(),
            EventOutcome::Applied
        );
        assert_eq!(database.store.billing(workspace()).unwrap().command, None);
        let mut updated = renewal();
        updated.items.data.truncate(1);
        fake.expect_subscription(remove_all_addons(), SubscriptionOutcome::respond(updated));
        assert!(matches!(
            billing.change_add_ons(
                access(),
                AddOns {
                    start_blocks: 0,
                    machines: 0
                }
            ),
            Ok(ChangeOutcome::Applied(_))
        ));
        assert_eq!(
            (state(&database).tier, state(&database).status),
            (Tier::Team, SubscriptionState::PaymentFailed)
        );
        let requests = fake.posts("/v1/subscriptions/sub_recorded");
        assert_eq!(requests.len(), 2);
        assert_ne!(requests[0].key, requests[1].key);
    }
}

#[test]
fn unobserved_upgrade_releases_at_one_day_without_showing_the_renewal_payment_page() {
    let (database, fake) = lost_upgrade();
    fake.state.lock().unwrap().subscription = renewal();
    let before = "2026-12-05T10:59:59Z".parse().unwrap();
    let reader = read_service(&database, &fake, before);
    assert!(matches!(
        reader.change_add_ons(
            access(),
            AddOns {
                start_blocks: 0,
                machines: 0
            }
        ),
        Err(Error::PaymentConfirming)
    ));
    let saved = database
        .store
        .billing(workspace())
        .unwrap()
        .command
        .unwrap();
    assert_eq!(saved.payment_expires_at, None);
    fake.expect_subscription(business_upgrade(), SubscriptionOutcome::respond(renewal()));
    assert_eq!(
        reader
            .change(access(), wanted(Tier::Business, 2, 1))
            .unwrap(),
        ChangeOutcome::PaymentRequired { payment_url: None }
    );
    assert_eq!(
        database
            .store
            .billing(workspace())
            .unwrap()
            .command
            .unwrap()
            .id,
        saved.id
    );
    let boundary = "2026-12-05T11:00:00Z".parse().unwrap();
    let billing = service(&database, &fake, boundary);
    let mut updated = renewal();
    updated.items.data.truncate(1);
    fake.expect_subscription(remove_all_addons(), SubscriptionOutcome::respond(updated));
    assert!(matches!(
        billing.change_add_ons(
            access(),
            AddOns {
                start_blocks: 0,
                machines: 0
            }
        ),
        Ok(ChangeOutcome::Applied(_))
    ));
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    let requests = fake.posts("/v1/subscriptions/sub_recorded");
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].key, requests[1].key);
    assert_ne!(requests[1].key, requests[2].key);
    assert_eq!(
        state(&database).failed_invoice_id,
        Some("in_renewal".into())
    );
    assert_eq!(
        state(&database).next_retry_at,
        Some("2026-12-06T12:00:00Z".parse().unwrap())
    );
}
