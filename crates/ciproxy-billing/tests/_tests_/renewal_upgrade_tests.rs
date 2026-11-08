//! A renewal failure stays separate from a later upgrade invoice.

use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::{ChangeOutcome, EventOutcome};
use ciproxy_billing::stripe::wire::invoice::{BillingReason, InvoiceStatus};
use ciproxy_billing::stripe::wire::types::{PendingUpdate, Status};
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};

use super::database::Database;
use super::fake::Fake;
use super::fake_subscriptions::SubscriptionOutcome;
use super::fixture::{
    access, at, read_service, seed_paid, service, signature, state, subscription, workspace,
};
use super::quote_fixture::quote;
use super::subscription_requests::business_upgrade;

#[test]
fn known_renewal_failure_survives_upgrade_expiry_until_actual_recovery() {
    renewal_and_upgrade(false, SubscriptionState::PaymentFailed);
}

#[test]
fn delayed_renewal_failure_learns_older_invoice_after_busy_and_upgrade() {
    renewal_and_upgrade(true, SubscriptionState::Active);
}

fn renewal_and_upgrade(delayed: bool, after_upgrade: SubscriptionState) {
    let database = Database::new(workspace());
    seed_paid(&database);
    let mut renewal = subscription().latest_invoice.unwrap();
    renewal.billing_reason = BillingReason::SubscriptionCycle;
    renewal.status = InvoiceStatus::Open;
    renewal.next_payment_attempt = Some(1794052800);
    let mut current = subscription();
    if delayed {
        current.latest_invoice.as_mut().unwrap().id = "in_initial".into();
    } else {
        current.status = Status::PastDue;
        current.latest_invoice = Some(renewal.clone());
    }
    let fake = Fake::new(current.clone());
    fake.state.lock().unwrap().invoices.push(renewal.clone());
    let billing = service(&database, &fake, at());
    let event_reader = read_service(&database, &fake, at());
    let failed = include_bytes!("fixtures/invoice_failed.json");
    let first = if delayed {
        let _lock = database
            .store
            .try_lock_billing(workspace())
            .unwrap()
            .unwrap();
        event_reader.webhook(&signature(failed, at()), failed)
    } else {
        event_reader.webhook(&signature(failed, at()), failed)
    };
    assert_eq!(
        first.err().map(|error| matches!(error, Error::Busy)),
        if delayed { Some(true) } else { None }
    );
    current.status = Status::PastDue;
    current.latest_invoice = Some(renewal);
    current.pending_update = Some(PendingUpdate {
        expires_at: 1793966400,
    });
    let upgrade = current.latest_invoice.as_mut().unwrap();
    upgrade.id = "in_upgrade".into();
    upgrade.billing_reason = BillingReason::SubscriptionUpdate;
    upgrade.next_payment_attempt = None;
    upgrade.hosted_invoice_url = Some("https://invoice.stripe.test/upgrade".into());
    {
        let mut remote = fake.state.lock().unwrap();
        remote.invoices[0].next_payment_attempt = Some(1794139200);
        remote.expect_subscription(
            business_upgrade().at(1793793600),
            SubscriptionOutcome::respond(current.clone()),
        );
    }
    assert_eq!(
        billing
            .change(access(), quote(&database, &fake, super::fixture::at()))
            .unwrap(),
        ChangeOutcome::PaymentRequired {
            payment_url: Some("https://invoice.stripe.test/upgrade".into())
        }
    );
    assert_eq!(state(&database).status, after_upgrade);
    let expected_failure = if delayed {
        None
    } else {
        Some("2026-11-05T12:00:00Z".parse().unwrap())
    };
    assert_eq!(state(&database).failed_at, expected_failure);
    let retry = String::from_utf8(failed.to_vec())
        .unwrap()
        .replace("evt_invoice_failed", "evt_renewal_redelivery");
    assert_eq!(
        event_reader
            .webhook(&signature(retry.as_bytes(), at()), retry.as_bytes())
            .unwrap(),
        EventOutcome::Applied
    );
    assert_eq!(
        (state(&database).tier, state(&database).status),
        (Tier::Team, SubscriptionState::PaymentFailed)
    );
    assert_eq!(
        state(&database).failed_invoice_id,
        Some("in_recorded".into())
    );
    assert_eq!(
        state(&database).failed_at,
        Some("2026-11-05T12:00:00Z".parse().unwrap())
    );
    assert_eq!(
        state(&database).next_retry_at,
        Some("2026-11-08T12:00:00Z".parse().unwrap())
    );
    current.pending_update = None;
    current.latest_invoice.as_mut().unwrap().status = InvoiceStatus::Void;
    {
        let mut remote = fake.state.lock().unwrap();
        remote.subscription = current;
        remote.invoices[0].next_payment_attempt = Some(1794225600);
    }
    let later = "2026-11-07T12:00:00Z".parse().unwrap();
    let events = read_service(&database, &fake, later);
    let update = br#"{"id":"evt_upgrade_expired","created":1794052800,"type":"customer.subscription.updated","data":{"object":{"id":"sub_recorded"}}}"#;
    assert_eq!(
        events.webhook(&signature(update, later), update).unwrap(),
        EventOutcome::Applied
    );
    assert_eq!(
        (state(&database).status, state(&database).next_retry_at),
        (
            SubscriptionState::PaymentFailed,
            Some("2026-11-09T12:00:00Z".parse().unwrap())
        )
    );
    assert_eq!(
        state(&database).failed_at,
        Some("2026-11-05T12:00:00Z".parse().unwrap())
    );
    assert_eq!(database.store.billing(workspace()).unwrap().command, None);
    fake.state.lock().unwrap().invoices[0].status = InvoiceStatus::Paid;
    let paid = include_bytes!("fixtures/invoice_paid.json");
    assert_eq!(
        events.webhook(&signature(paid, later), paid).unwrap(),
        EventOutcome::Applied
    );
    assert_eq!(
        (
            state(&database).status,
            state(&database).failed_at,
            state(&database).next_retry_at
        ),
        (SubscriptionState::Active, None, None)
    );
    assert_eq!(state(&database).tier, Tier::Team);
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 1);
}
