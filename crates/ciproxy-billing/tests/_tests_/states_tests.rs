//! Current Stripe state wins over duplicate, old, and same-second snapshots.

use ciproxy_billing::service::view::EventOutcome;
use ciproxy_billing::stripe::wire::invoice::{BillingReason, CancelReason, InvoiceStatus};
use ciproxy_billing::stripe::wire::types::Status;
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};

use super::database::Database;
use super::fake::Fake;
use super::fixture::{access, read_service, seed_paid, signature, state, subscription, workspace};

#[test]
fn retry_dates_use_failed_payment_event_and_old_failure_cannot_restore_recovered_banner() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    let now = "2026-11-07T12:00:00Z".parse().unwrap();
    let service = read_service(&database, &fake, now);
    {
        let mut remote = fake.state.lock().unwrap();
        remote.subscription.status = Status::PastDue;
        let invoice = remote.subscription.latest_invoice.as_mut().unwrap();
        invoice.status = InvoiceStatus::Open;
        invoice.billing_reason = BillingReason::SubscriptionCycle;
        invoice.next_payment_attempt = Some(1794052800);
    }
    let failed = include_bytes!("fixtures/invoice_failed.json");
    assert_eq!(
        service.webhook(&signature(failed, now), failed).unwrap(),
        EventOutcome::Applied
    );
    let retrying = state(&database);
    assert_eq!(
        (
            retrying.tier,
            retrying.add_ons.start_blocks,
            retrying.add_ons.machines
        ),
        (Tier::Team, 2, 1)
    );
    assert_eq!(
        retrying.failed_at,
        Some("2026-11-05T12:00:00Z".parse().unwrap())
    );
    assert_eq!(
        retrying.next_retry_at,
        Some("2026-11-07T12:00:00Z".parse().unwrap())
    );
    assert_eq!(retrying.status, SubscriptionState::PaymentFailed);
    let later_failure = String::from_utf8(failed.to_vec())
        .unwrap()
        .replace("evt_invoice_failed", "evt_invoice_failed_second_attempt")
        .replace("1793880000", "1793966400");
    assert_eq!(
        service
            .webhook(
                &signature(later_failure.as_bytes(), now),
                later_failure.as_bytes()
            )
            .unwrap(),
        EventOutcome::Applied
    );
    assert_eq!(
        state(&database).failed_at,
        Some("2026-11-06T12:00:00Z".parse().unwrap())
    );
    fake.state.lock().unwrap().subscription = subscription();
    let paid = include_bytes!("fixtures/invoice_paid.json");
    assert_eq!(
        service.webhook(&signature(paid, now), paid).unwrap(),
        EventOutcome::Applied
    );
    let stale = String::from_utf8(failed.to_vec())
        .unwrap()
        .replace("evt_invoice_failed", "evt_invoice_failed_later_delivery");
    assert_eq!(
        service
            .webhook(&signature(stale.as_bytes(), now), stale.as_bytes())
            .unwrap(),
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
    assert_eq!(service.status(access()).unwrap().state.tier, Tier::Team);
}

#[test]
fn actual_end_starts_free_month_and_ended_banner_without_changing_workspace_resources() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let fake = Fake::new(subscription());
    let ended = "2027-01-31T12:00:00Z"
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap();
    let service = read_service(&database, &fake, ended);
    {
        let mut remote = fake.state.lock().unwrap();
        remote.subscription.status = Status::Canceled;
        remote.subscription.ended_at = Some(ended.timestamp());
        remote
            .subscription
            .cancellation_details
            .as_mut()
            .unwrap()
            .reason = Some(CancelReason::PaymentFailed);
    }
    let raw = include_bytes!("fixtures/subscription_ended.json");
    assert_eq!(
        service.webhook(&signature(raw, ended), raw).unwrap(),
        EventOutcome::Applied
    );
    let free = state(&database);
    assert_eq!(
        (free.tier, free.add_ons.start_blocks, free.add_ons.machines),
        (Tier::Free, 0, 0)
    );
    assert_eq!(free.free_started_at, ended);
    assert_eq!(
        free.ended_banner_until,
        Some("2027-02-28T12:00:00Z".parse().unwrap())
    );
    assert_eq!(
        (free.paid_period_start, free.paid_period_end, free.failed_at),
        (None, None, None)
    );
    assert_eq!(
        database.store.workspace(workspace()).unwrap().unwrap().name,
        "Billing fixture"
    );
    assert_eq!(
        service.webhook(&signature(raw, ended), raw).unwrap(),
        EventOutcome::Duplicate
    );
    assert_eq!(state(&database), free);
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 0);
}

#[test]
fn unrelated_subscription_event_keeps_unknown_failed_payment_date_absent() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let mut current = subscription();
    current.status = Status::PastDue;
    let latest = current.latest_invoice.as_mut().unwrap();
    latest.billing_reason = BillingReason::SubscriptionUpdate;
    latest.status = InvoiceStatus::Open;
    let fake = Fake::new(current);
    let now = "2026-11-04T12:00:00Z".parse().unwrap();
    let billing = read_service(&database, &fake, now);
    let raw = br#"{"id":"evt_unrelated_subscription_update","created":1793793600,"type":"customer.subscription.updated","data":{"object":{"id":"sub_recorded"}}}"#;
    assert_eq!(
        billing.webhook(&signature(raw, now), raw).unwrap(),
        EventOutcome::Applied
    );
    let result = state(&database);
    assert_eq!(
        (result.status, result.failed_invoice_id, result.failed_at),
        (SubscriptionState::PaymentFailed, None, None)
    );
}
