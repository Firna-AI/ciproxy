//! A fresh collection resume permanently retires the old cancellation proof.

use std::slice;

use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::EventOutcome;
use ciproxy_store_interface::billing::collection::CancellationState;
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};

use serde_json::json;

use super::database::Database;
use super::fake::Fake;
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::live_replay::{Recording, deliver, install, saved, send};

#[test]
fn observed_resume_retires_a_failed_cancellation_before_manual_events_arrive() {
    let recording = Recording::final_failure();
    let mut database = recording.seed();
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    let stop = &recording.events[2];
    let invoice = stop["data"]["object"].clone();
    let current = recording
        .reads
        .values()
        .find(|body| body["id"] == id && body["pending_update"].is_object())
        .unwrap()
        .clone();
    let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
    install(&fake, current.clone(), slice::from_ref(&invoice));
    fake.expect_subscription(
        SubscriptionRequest::CancelImmediately(id.into()),
        SubscriptionOutcome::Reject(503),
    );
    assert!(matches!(
        send(&database, &fake, stop),
        Err(Error::StripeHttp { status: 503 })
    ));
    let mut resumed = invoice.clone();
    resumed["auto_advance"] = json!(true);
    install(&fake, current.clone(), &[resumed]);
    let observed = json!({"id":"evt_derived_resumed_collection_observed", "created":1793845030, "type":"customer.subscription.updated", "data":{"object":{"id":id}}});
    assert_eq!(deliver(&database, &fake, &observed), EventOutcome::Applied);
    assert_eq!(
        decisions(&database, &recording),
        vec![(
            "evt_1UYGEFAGa5h8DTS3z2H40Eaa".into(),
            CancellationState::Superseded
        )]
    );
    database.reconnect_store();
    install(&fake, current.clone(), slice::from_ref(&invoice));
    assert_eq!(deliver(&database, &fake, stop), EventOutcome::Applied);
    assert_eq!(saved(&database, &recording).tier, Tier::Team);
    assert_eq!(
        fake.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|request| request.method == "DELETE")
            .count(),
        1,
        "retrying the retired automatic event must not cancel a later manual stop"
    );
    assert_eq!(
        decisions(&database, &recording),
        vec![(
            "evt_1UYGEFAGa5h8DTS3z2H40Eaa".into(),
            CancellationState::Superseded
        )]
    );
    let mut later_invoice = invoice;
    later_invoice["attempt_count"] = json!(10);
    later_invoice["auto_advance"] = json!(true);
    install(&fake, current.clone(), &[later_invoice.clone()]);
    let later_resume = json!({"id":"evt_derived_later_collection_resumed", "created":1793845040, "type":"customer.subscription.updated", "data":{"object":{"id":id}}});
    assert_eq!(
        deliver(&database, &fake, &later_resume),
        EventOutcome::Applied
    );
    later_invoice["auto_advance"] = json!(false);
    let mut later_stop = stop.clone();
    later_stop["id"] = json!("evt_derived_distinct_automatic_stop");
    later_stop["created"] = json!(1793845050);
    later_stop["data"]["object"] = later_invoice.clone();
    install(&fake, current, &[later_invoice]);
    fake.expect_subscription(
        SubscriptionRequest::CancelImmediately(id.into()),
        SubscriptionOutcome::respond(
            serde_json::from_value(recording.snapshot(id, "canceled")).unwrap(),
        ),
    );
    fake.state.lock().unwrap().lose_next_delete = true;
    assert!(matches!(
        send(&database, &fake, &later_stop),
        Err(Error::StripeHttp { status: 503 })
    ));
    assert_eq!(
        decisions(&database, &recording),
        vec![
            (
                "evt_1UYGEFAGa5h8DTS3z2H40Eaa".into(),
                CancellationState::Superseded
            ),
            (
                "evt_derived_distinct_automatic_stop".into(),
                CancellationState::Eligible
            ),
        ]
    );
    database.reconnect_store();
    assert_eq!(
        deliver(&database, &fake, &recording.events[3]),
        EventOutcome::Applied
    );
    assert_eq!(
        (
            saved(&database, &recording).tier,
            saved(&database, &recording).status
        ),
        (Tier::Free, SubscriptionState::Ended)
    );
    let completed = vec![
        (
            "evt_1UYGEFAGa5h8DTS3z2H40Eaa".into(),
            CancellationState::Superseded,
        ),
        (
            "evt_derived_distinct_automatic_stop".into(),
            CancellationState::Completed,
        ),
    ];
    assert_eq!(decisions(&database, &recording), completed);
    assert_eq!(
        deliver(&database, &fake, &later_stop),
        EventOutcome::Applied
    );
    assert_eq!(
        deliver(&database, &fake, &later_stop),
        EventOutcome::Duplicate
    );
    assert_eq!(deliver(&database, &fake, stop), EventOutcome::Duplicate);
    assert_eq!(decisions(&database, &recording), completed);
    assert_eq!(
        fake.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|request| request.method == "DELETE")
            .count(),
        2
    );
}

fn decisions(database: &Database, recording: &Recording) -> Vec<(String, CancellationState)> {
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    database
        .store
        .try_lock_billing(recording.initial_state.workspace_id)
        .unwrap()
        .unwrap()
        .renewals(id)
        .unwrap()[0]
        .collection
        .decisions
        .iter()
        .map(|decision| (decision.stop.event_id.clone(), decision.state))
        .collect()
}

#[test]
fn fresh_supersession_persists_before_a_later_provider_read_fails() {
    for phase in [
        "renewal_refresh",
        "pre_cancel_refresh",
        "invalid_retry_time",
    ] {
        let recording = Recording::final_failure();
        let mut database = recording.seed();
        let id = recording.initial_state.subscription_id.as_deref().unwrap();
        let stop = &recording.events[2];
        let invoice = stop["data"]["object"].clone();
        let current = recording.snapshot(id, "active");
        let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
        install(&fake, current.clone(), slice::from_ref(&invoice));
        fake.expect_subscription(
            SubscriptionRequest::CancelImmediately(id.into()),
            SubscriptionOutcome::Reject(503),
        );
        assert!(matches!(
            send(&database, &fake, stop),
            Err(Error::StripeHttp { status: 503 })
        ));
        let mut resumed = invoice.clone();
        resumed["auto_advance"] = json!(true);
        let mut unowned = invoice.clone();
        unowned["customer"] = json!("cus_derived_unowned_read");
        if phase == "renewal_refresh" {
            let mut with_later_invoice = current.clone();
            unowned["id"] = json!("in_zzz_derived_unowned_renewal");
            with_later_invoice["latest_invoice"] = unowned.clone();
            install(&fake, with_later_invoice, &[resumed, unowned]);
        } else if phase == "invalid_retry_time" {
            resumed["next_payment_attempt"] = json!(i64::MAX);
            install(&fake, current.clone(), &[resumed]);
        } else {
            install(&fake, current.clone(), &[]);
            fake.state.lock().unwrap().raw_read_sequences.insert(
                format!("/v1/invoices/{}", invoice["id"].as_str().unwrap()),
                [invoice.clone(), resumed, unowned].into(),
            );
        }
        let observed = json!({"id":"evt_derived_resume_then_failed_read", "created":1793845030, "type":"customer.subscription.updated", "data":{"object":{"id":id}}});
        assert_eq!(
            send(&database, &fake, &observed)
                .err()
                .map(|error| match error {
                    Error::Ownership => "ownership",
                    Error::StripeState => "invalid_time",
                    _ => "unexpected",
                }),
            Some(if phase == "invalid_retry_time" {
                "invalid_time"
            } else {
                "ownership"
            }),
            "{phase}"
        );
        assert_eq!(
            decisions(&database, &recording),
            vec![(
                "evt_1UYGEFAGa5h8DTS3z2H40Eaa".into(),
                CancellationState::Superseded
            )],
            "{phase}"
        );
        database.reconnect_store();
        install(&fake, current, &[invoice]);
        assert_eq!(
            deliver(&database, &fake, stop),
            EventOutcome::Applied,
            "{phase}"
        );
        assert_eq!(
            fake.state
                .lock()
                .unwrap()
                .requests
                .iter()
                .filter(|request| request.method == "DELETE")
                .count(),
            1,
            "{phase}"
        );
    }
}
