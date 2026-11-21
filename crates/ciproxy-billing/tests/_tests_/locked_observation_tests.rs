//! Every owned collection observation reaches the decision owner under the lock.

use std::slice;

use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::EventOutcome;
use ciproxy_store_interface::billing::collection::CancellationState;
use ciproxy_store_interface::billing::state::Tier;
use serde_json::json;

use super::fake::Fake;
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::live_replay::{Recording, deliver, install, saved, send};

#[test]
fn the_first_owned_invoice_resume_retires_the_existing_stop() {
    resumed_observation("invoice");
}

#[test]
fn an_expanded_latest_invoice_resume_retires_the_existing_stop() {
    resumed_observation("expanded_invoice");
}

#[test]
fn an_owned_subscription_pause_retires_the_existing_stop() {
    resumed_observation("pause");
}

fn resumed_observation(source: &str) {
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
    let event = if source == "invoice" {
        install(&fake, current, slice::from_ref(&invoice));
        fake.state.lock().unwrap().raw_read_sequences.insert(
            format!("/v1/invoices/{}", invoice["id"].as_str().unwrap()),
            [resumed].into(),
        );
        stop.clone()
    } else {
        let mut stopped = current;
        let mut snapshot = stopped.clone();
        if source == "expanded_invoice" {
            stopped["latest_invoice"] = invoice.clone();
            snapshot["latest_invoice"] = resumed;
        } else {
            snapshot["pause_collection"] = json!({"behavior":"keep_as_draft"});
        }
        install(&fake, stopped, slice::from_ref(&invoice));
        fake.state
            .lock()
            .unwrap()
            .raw_read_sequences
            .insert(format!("/v1/subscriptions/{id}"), [snapshot].into());
        json!({"id":"evt_derived_first_subscription_observation", "created":1793845030, "type":"customer.subscription.updated", "data":{"object":{"id":id}}})
    };
    assert_eq!(deliver(&database, &fake, &event), EventOutcome::Applied);
    database.reconnect_store();
    let renewals = database
        .store
        .try_lock_billing(recording.initial_state.workspace_id)
        .unwrap()
        .unwrap()
        .renewals(id)
        .unwrap();
    assert_eq!(
        renewals[0]
            .collection
            .decisions
            .iter()
            .map(|decision| decision.state)
            .collect::<Vec<_>>(),
        vec![CancellationState::Superseded],
        "{source}"
    );
    assert_eq!(saved(&database, &recording).tier, Tier::Team);
    let later = json!({"id":"evt_derived_later_manual_stop_observed", "created":1793845040, "type":"customer.subscription.updated", "data":{"object":{"id":id}}});
    assert_eq!(deliver(&database, &fake, &later), EventOutcome::Applied);
    assert_eq!(
        fake.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|request| request.method == "DELETE")
            .count(),
        1,
        "{source}"
    );
}

#[test]
fn known_events_lock_before_stripe_reads_and_duplicates_read_nothing() {
    let recording = Recording::final_failure();
    let database = recording.seed();
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    let current = recording.snapshot(id, "active");
    let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
    install(
        &fake,
        current,
        &[recording.events[2]["data"]["object"].clone()],
    );
    let event = json!({"id":"evt_derived_known_subscription", "created":1793845040, "type":"customer.subscription.updated", "data":{"object":{"id":id}}});
    {
        let _lock = database
            .store
            .try_lock_billing(recording.initial_state.workspace_id)
            .unwrap()
            .unwrap();
        for raw_event in [&event, &recording.events[2]] {
            assert!(matches!(
                send(&database, &fake, raw_event),
                Err(Error::Busy)
            ));
            let requests = fake.state.lock().unwrap().requests.len();
            assert_eq!(
                requests, 0,
                "known events must acquire the billing lock before any Stripe read"
            );
        }
    }
    assert_eq!(deliver(&database, &fake, &event), EventOutcome::Applied);
    let count = fake.state.lock().unwrap().requests.len();
    assert!(count > 0);
    assert_eq!(deliver(&database, &fake, &event), EventOutcome::Duplicate);
    assert_eq!(
        fake.state.lock().unwrap().requests.len(),
        count,
        "a known duplicate must not read a stale provider snapshot"
    );
    let before = database
        .store
        .billing(recording.initial_state.workspace_id)
        .unwrap();
    let mut unknown = recording.snapshot(id, "active");
    unknown["id"] = json!("sub_derived_unknown");
    fake.state
        .lock()
        .unwrap()
        .raw_reads
        .insert("/v1/subscriptions/sub_derived_unknown".into(), unknown);
    let discovered = json!({"id":"evt_derived_unknown_subscription", "created":1793845050, "type":"customer.subscription.updated", "data":{"object":{"id":"sub_derived_unknown"}}});
    assert_eq!(
        deliver(&database, &fake, &discovered),
        EventOutcome::Ignored
    );
    assert_eq!(
        database
            .store
            .billing(recording.initial_state.workspace_id)
            .unwrap(),
        before
    );
    let requests = fake.state.lock().unwrap().requests.len();
    assert_eq!(
        requests,
        count + 1,
        "unknown subscriptions retain one discovery read and apply no snapshot"
    );
}
