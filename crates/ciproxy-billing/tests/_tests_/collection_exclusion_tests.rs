//! Fresh ownership and collection evidence exclude recovery and operator changes.

use ciproxy_billing::service::view::EventOutcome;
use ciproxy_store_interface::billing::collection::CancellationState;
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};
use serde_json::{Value, json};

use super::fake::Fake;
use super::live_replay::{Recording, deliver, install, saved};

#[test]
fn automatic_stop_requires_the_same_fresh_unpaid_automatic_unpaused_invoice() {
    for variation in [
        "paid",
        "void",
        "paused",
        "collection_resumed",
        "later_attempt",
        "zero_remaining",
        "manual_invoice",
        "manual_subscription",
        "not_a_renewal",
        "missing_transition",
        "missing_request_provenance",
    ] {
        let recording = Recording::final_failure();
        let database = recording.seed();
        let id = recording.initial_state.subscription_id.as_deref().unwrap();
        let mut current = recording.snapshot(id, "active");
        let mut stop = recording.events[2].clone();
        let mut invoice = stop["data"]["object"].clone();
        match variation {
            "paid" => {
                invoice["status"] = json!("paid");
                invoice["amount_remaining"] = json!(0);
            }
            "void" => invoice["status"] = json!("void"),
            "paused" => current["pause_collection"] = json!({"behavior":"keep_as_draft"}),
            "collection_resumed" => invoice["auto_advance"] = json!(true),
            "later_attempt" => invoice["attempt_count"] = json!(10),
            "zero_remaining" => invoice["amount_remaining"] = json!(0),
            "manual_invoice" => invoice["collection_method"] = json!("send_invoice"),
            "manual_subscription" => current["collection_method"] = json!("send_invoice"),
            "not_a_renewal" => invoice["billing_reason"] = json!("subscription_update"),
            "missing_transition" => stop["data"]["previous_attributes"] = Value::Null,
            "missing_request_provenance" => {
                stop.as_object_mut().unwrap().remove("request");
            }
            _ => unreachable!(),
        }
        let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
        install(&fake, current, &[invoice]);
        assert_eq!(
            deliver(&database, &fake, &stop),
            EventOutcome::Applied,
            "{variation}"
        );
        assert_eq!(saved(&database, &recording).tier, Tier::Team, "{variation}");
        assert_eq!(
            fake.state
                .lock()
                .unwrap()
                .requests
                .iter()
                .filter(|request| request.method == "DELETE")
                .count(),
            0,
            "{variation}"
        );
        let mut session = database
            .store
            .try_lock_billing(recording.initial_state.workspace_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            session
                .renewals(id)
                .unwrap()
                .iter()
                .flat_map(|renewal| &renewal.collection.decisions)
                .filter(|decision| decision.state == CancellationState::Eligible)
                .count(),
            0,
            "{variation}"
        );
    }
}

#[test]
fn recorded_manual_collection_stop_keeps_paid_coverage_without_cancellation() {
    let manual: Value =
        serde_json::from_str(include_str!("fixtures/live/manual_stop.json")).unwrap();
    let mut recording = Recording::final_failure();
    recording.initial_state = serde_json::from_value(manual["initial_state"].clone()).unwrap();
    let database = recording.seed();
    let mut current = recording
        .reads
        .values()
        .find(|body| body["items"].is_object())
        .unwrap()
        .clone();
    current["id"] = json!(recording.initial_state.subscription_id);
    current["customer"] = json!(recording.initial_state.customer_id);
    current["metadata"] = json!({"workspace_id":recording.initial_state.workspace_id});
    current["status"] = json!("past_due");
    current["pending_update"] = Value::Null;
    current["latest_invoice"] = manual["event"]["data"]["object"].clone();
    let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
    install(&fake, current, &[manual["event"]["data"]["object"].clone()]);
    assert_eq!(
        deliver(&database, &fake, &manual["event"]),
        EventOutcome::Applied
    );
    let state = saved(&database, &recording);
    assert_eq!(
        (state.tier, state.status, state.failed_invoice_id),
        (
            Tier::Team,
            SubscriptionState::PaymentFailed,
            Some("in_1UYGHrAGa5h8DTS3mhnRZIPL".into())
        )
    );
    assert_eq!(
        fake.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|request| request.method == "DELETE")
            .count(),
        0
    );
}

#[test]
fn recorded_control_and_single_upgrade_ends_keep_the_confirmed_free_transition() {
    let controls: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/live/controls.json")).unwrap();
    for control in controls {
        let mut recording = Recording::final_failure();
        recording.initial_state = serde_json::from_value(control["initial_state"].clone()).unwrap();
        let database = recording.seed();
        let current = control["subscription"].clone();
        let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
        install(&fake, current, &[]);
        assert_eq!(
            deliver(&database, &fake, &control["event"]),
            EventOutcome::Applied
        );
        let state = saved(&database, &recording);
        assert_eq!(
            (state.tier, state.status),
            (Tier::Free, SubscriptionState::Ended)
        );
        assert_eq!(
            deliver(&database, &fake, &control["event"]),
            EventOutcome::Duplicate
        );
    }
}

#[test]
fn a_payment_void_or_pause_between_verification_and_cancellation_prevents_delete() {
    for variation in ["paid", "void", "paused"] {
        let recording = Recording::final_failure();
        let database = recording.seed();
        let id = recording.initial_state.subscription_id.as_deref().unwrap();
        let current = recording.snapshot(id, "active");
        let stopped = recording.events[2]["data"]["object"].clone();
        let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
        let mut invoice = stopped.clone();
        let mut fresh = current.clone();
        match variation {
            "paid" => {
                invoice["status"] = json!("paid");
                invoice["amount_remaining"] = json!(0);
            }
            "void" => invoice["status"] = json!("void"),
            "paused" => fresh["pause_collection"] = json!({"behavior":"void"}),
            _ => unreachable!(),
        }
        install(&fake, fresh, &[invoice]);
        {
            let mut remote = fake.state.lock().unwrap();
            remote.raw_read_sequences.insert(
                format!("/v1/subscriptions/{id}"),
                [current.clone(), current].into(),
            );
            remote.raw_read_sequences.insert(
                format!("/v1/invoices/{}", stopped["id"].as_str().unwrap()),
                [stopped.clone(), stopped].into(),
            );
        }
        assert_eq!(
            deliver(&database, &fake, &recording.events[2]),
            EventOutcome::Applied,
            "{variation}"
        );
        assert_eq!(saved(&database, &recording).tier, Tier::Team, "{variation}");
        assert_eq!(
            fake.state
                .lock()
                .unwrap()
                .requests
                .iter()
                .filter(|request| request.method == "DELETE")
                .count(),
            0,
            "{variation}"
        );
        let mut session = database
            .store
            .try_lock_billing(recording.initial_state.workspace_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            session.renewals(id).unwrap()[0]
                .collection
                .decisions
                .iter()
                .map(|decision| decision.state)
                .collect::<Vec<_>>(),
            vec![CancellationState::Superseded],
            "{variation}"
        );
    }
}
