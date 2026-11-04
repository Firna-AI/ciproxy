//! Literal Stripe snapshots for lost-reply expiry scenarios.

use chrono::{DateTime, Utc};
use ciproxy_billing::error::Error;
use ciproxy_billing::stripe::wire::invoice::{BillingReason, InvoiceStatus};
use ciproxy_billing::stripe::wire::types::{PendingUpdate, Status, Subscription};
use ciproxy_store_interface::billing::state::Tier;
use serde_json::json;

use super::database::Database;
use super::fake::Fake;
use super::fake_subscriptions::SubscriptionOutcome;
use super::fixture::{access, seed_paid, service, subscription, wanted, workspace};
use super::subscription_requests::business_upgrade;

pub(super) fn started() -> DateTime<Utc> {
    "2026-12-04T11:00:00Z".parse().unwrap()
}

pub(super) fn pending() -> Subscription {
    let mut current = subscription();
    current.pending_update = Some(PendingUpdate {
        expires_at: 1796385600,
    });
    let invoice = current.latest_invoice.as_mut().unwrap();
    invoice.id = "in_upgrade".into();
    invoice.created = 1796382000;
    invoice.billing_reason = BillingReason::SubscriptionUpdate;
    invoice.status = InvoiceStatus::Open;
    invoice.hosted_invoice_url = Some("https://invoice.stripe.test/upgrade".into());
    current
}

pub(super) fn renewal() -> Subscription {
    let mut current = subscription();
    current.status = Status::PastDue;
    for item in &mut current.items.data {
        item.current_period_start = 1796385600;
        item.current_period_end = 1799064000;
    }
    let invoice = current.latest_invoice.as_mut().unwrap();
    invoice.id = "in_renewal".into();
    invoice.created = 1796385600;
    invoice.billing_reason = BillingReason::SubscriptionCycle;
    invoice.status = InvoiceStatus::Open;
    invoice.next_payment_attempt = Some(1796558400);
    invoice.hosted_invoice_url = Some("https://invoice.stripe.test/renewal".into());
    current
}

pub(super) fn lost_upgrade() -> (Database, Fake) {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    {
        let mut remote = fake.state.lock().unwrap();
        remote.expect_subscription(business_upgrade(), SubscriptionOutcome::respond(pending()));
        remote.lose_next_post = true;
    }
    let billing = service(&database, &fake, started());
    assert!(matches!(
        billing.change(access(), wanted(Tier::Business, 2, 1)),
        Err(Error::StripeHttp { status: 503 })
    ));
    assert_eq!(
        database
            .store
            .billing(workspace())
            .unwrap()
            .command
            .unwrap()
            .payment_expires_at,
        None
    );
    (database, fake)
}

pub(super) fn event(id: &str, event_type: &str, now: DateTime<Utc>) -> Vec<u8> {
    serde_json::to_vec(&json!({"id":id,"created":now.timestamp(),"type":event_type,
        "data":{"object":{"id":"sub_recorded"}}}))
    .unwrap()
}
