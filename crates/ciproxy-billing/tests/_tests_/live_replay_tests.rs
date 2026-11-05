//! Real recorded recovery and automatic-stop regressions through signed webhooks.

use chrono::{DateTime, Utc};
use ciproxy_billing::service::view::EventOutcome;
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};

use super::fake::Fake;
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::live_replay::{Recording, deliver, install, saved};

#[test]
fn recorded_paid_renewal_stays_recovered_after_later_signed_event_and_restart() {
    let recording = Recording::recovery();
    let mut database = recording.seed();
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    let renewal_id = recording.events[0]["data"]["object"]["id"]
        .as_str()
        .unwrap();
    let before = recording.snapshot(id, "past_due");
    let fake = Fake::new(serde_json::from_value(before.clone()).unwrap());
    let mut unattempted = recording.snapshot(renewal_id, "open");
    unattempted["attempted"] = serde_json::json!(false);
    unattempted["attempt_count"] = serde_json::json!(0);
    unattempted["next_payment_attempt"] = serde_json::Value::Null;
    let mut active = before.clone();
    active["status"] = serde_json::json!("active");
    active["latest_invoice"] = unattempted.clone();
    install(&fake, active, &[unattempted.clone()]);
    let update = serde_json::json!({
        "id":"evt_derived_unattempted_renewal",
        "created":1793844851,
        "type":"invoice.updated",
        "request":{"id":"req_derived_finalize"},
        "data":{"object":unattempted,"previous_attributes":{"status":"draft"}}
    });
    assert_eq!(deliver(&database, &fake, &update), EventOutcome::Applied);
    let awaiting_charge = saved(&database, &recording);
    assert_eq!(
        (
            awaiting_charge.status,
            awaiting_charge.failed_invoice_id,
            awaiting_charge.failed_at,
            awaiting_charge.next_retry_at
        ),
        (SubscriptionState::Active, None, None, None),
        "an owned open renewal must stay active before its first charge attempt"
    );
    database.reconnect_store();
    let renewals = database
        .store
        .try_lock_billing(recording.initial_state.workspace_id)
        .unwrap()
        .unwrap()
        .renewals(id)
        .unwrap();
    assert_eq!(renewals.len(), 1);
    assert_eq!(renewals[0].attempted, Some(false));
    install(&fake, before, &[recording.snapshot(renewal_id, "open")]);
    assert_eq!(
        deliver(&database, &fake, &recording.events[0]),
        EventOutcome::Applied
    );
    assert_eq!(
        saved(&database, &recording).status,
        SubscriptionState::PaymentFailed
    );
    let after = recording
        .reads
        .values()
        .find(|body| body["id"] == id && body["pending_update"].is_object())
        .unwrap()
        .clone();
    install(&fake, after, &[recording.snapshot(renewal_id, "paid")]);
    assert_eq!(
        deliver(&database, &fake, &recording.events[1]),
        EventOutcome::Applied
    );
    assert_eq!(
        saved(&database, &recording).status,
        SubscriptionState::Active
    );
    assert_eq!(
        deliver(&database, &fake, &recording.events[2]),
        EventOutcome::Applied
    );
    let state = saved(&database, &recording);
    assert_eq!(
        (
            state.status,
            state.failed_at,
            state.failed_invoice_id,
            state.next_retry_at
        ),
        (SubscriptionState::Active, None, None, None),
        "a later signed invoice event must preserve the paid renewal recovery"
    );
    assert_eq!(
        deliver(&database, &fake, &recording.events[2]),
        EventOutcome::Duplicate
    );
    let mut voided = recording
        .reads
        .values()
        .find(|body| body["id"] == id && body["pending_update"].is_object())
        .unwrap()
        .clone();
    voided["pending_update"] = serde_json::Value::Null;
    voided["latest_invoice"]["status"] = serde_json::json!("void");
    install(&fake, voided, &[]);
    let later = serde_json::json!({"id":"evt_derived_void_upgrade_later", "created":1793844877, "type":"customer.subscription.updated", "data":{"object":{"id":id}}});
    assert_eq!(deliver(&database, &fake, &later), EventOutcome::Applied);
    assert_eq!(
        saved(&database, &recording).status,
        SubscriptionState::Active
    );
}

#[test]
fn recorded_automatic_collection_stop_ends_subscription_after_stripe_confirmation() {
    let recording = Recording::final_failure();
    let database = recording.seed();
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    let renewal_id = recording.events[2]["data"]["object"]["id"]
        .as_str()
        .unwrap();
    let before = recording.snapshot(id, "past_due");
    let fake = Fake::new(serde_json::from_value(before.clone()).unwrap());
    let retrying = recording
        .reads
        .values()
        .find(|body| body["id"] == renewal_id && body["attempt_count"] == 8)
        .unwrap()
        .clone();
    install(&fake, before, &[retrying]);
    assert_eq!(
        deliver(&database, &fake, &recording.events[0]),
        EventOutcome::Applied
    );
    assert_eq!(
        saved(&database, &recording).status,
        SubscriptionState::PaymentFailed
    );
    let active = recording
        .reads
        .values()
        .find(|body| body["id"] == id && body["pending_update"].is_object())
        .unwrap()
        .clone();
    let stopped = recording
        .reads
        .values()
        .find(|body| body["id"] == renewal_id && body["auto_advance"] == false)
        .unwrap()
        .clone();
    install(&fake, active, &[stopped]);
    fake.expect_subscription(
        SubscriptionRequest::CancelImmediately(id.into()),
        SubscriptionOutcome::respond(
            serde_json::from_value(recording.snapshot(id, "canceled")).unwrap(),
        ),
    );
    assert_eq!(
        deliver(&database, &fake, &recording.events[1]),
        EventOutcome::Applied
    );
    assert_eq!(saved(&database, &recording).tier, Tier::Team);
    assert_eq!(
        deliver(&database, &fake, &recording.events[2]),
        EventOutcome::Applied
    );
    let state = saved(&database, &recording);
    assert_eq!(
        (state.tier, state.status),
        (Tier::Free, SubscriptionState::Ended),
        "a verified automatic collection stop must end the owned unpaid renewal"
    );
    assert_eq!(
        state.free_started_at,
        "2026-12-19T04:12:02Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        state.ended_banner_until,
        Some("2027-01-19T04:12:02Z".parse().unwrap())
    );
    for later in &recording.events[3..] {
        assert_eq!(deliver(&database, &fake, later), EventOutcome::Applied);
        assert_eq!(saved(&database, &recording), state);
    }
    assert_eq!(
        deliver(&database, &fake, &recording.events[2]),
        EventOutcome::Duplicate
    );
}

#[test]
fn paying_one_recorded_renewal_keeps_another_owned_unpaid_renewal_visible() {
    let recording = Recording::recovery();
    let database = recording.seed();
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    let renewal_id = recording.events[0]["data"]["object"]["id"]
        .as_str()
        .unwrap();
    let before = recording.snapshot(id, "past_due");
    let fake = Fake::new(serde_json::from_value(before.clone()).unwrap());
    let first = recording.snapshot(renewal_id, "open");
    let mut second = first.clone();
    second["id"] = serde_json::json!("in_derived_other_renewal");
    let mut second_failure = recording.events[0].clone();
    second_failure["id"] = serde_json::json!("evt_derived_other_renewal_failure");
    second_failure["data"]["object"]["id"] = second["id"].clone();
    install(&fake, before.clone(), &[first.clone(), second.clone()]);
    deliver(&database, &fake, &recording.events[0]);
    deliver(&database, &fake, &second_failure);
    let paid = recording.snapshot(renewal_id, "paid");
    let mut after = before;
    after["latest_invoice"] = paid.clone();
    install(&fake, after, &[paid, second]);
    deliver(&database, &fake, &recording.events[1]);
    let state = saved(&database, &recording);
    assert_eq!(
        (state.status, state.failed_invoice_id, state.failed_at),
        (
            SubscriptionState::PaymentFailed,
            Some("in_derived_other_renewal".into()),
            Some("2026-11-05T02:14:12Z".parse().unwrap())
        ),
        "another successful renewal must not hide the owned open unpaid invoice"
    );
}
