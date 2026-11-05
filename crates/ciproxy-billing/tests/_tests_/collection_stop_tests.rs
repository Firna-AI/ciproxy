//! Durable collection stops survive provider failure and service restart.

use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::EventOutcome;
use ciproxy_store_interface::billing::renewals::RenewalOutcome;
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};
use serde_json::json;

use super::fake::Fake;
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::live_replay::{Recording, deliver, install, saved, send};

#[test]
fn verified_stop_retries_after_provider_failure_and_restart_without_its_original_event() {
    let recording = Recording::final_failure();
    let mut database = recording.seed();
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    let stop = &recording.events[2];
    let invoice = stop["data"]["object"].clone();
    let pending = recording
        .reads
        .values()
        .find(|body| body["id"] == id && body["pending_update"].is_object())
        .unwrap()
        .clone();
    let fake = Fake::new(serde_json::from_value(pending.clone()).unwrap());
    install(&fake, pending, &[invoice]);
    fake.expect_subscription(
        SubscriptionRequest::CancelImmediately(id.into()),
        SubscriptionOutcome::Reject(503),
    );
    assert!(matches!(
        send(&database, &fake, stop),
        Err(Error::StripeHttp { status: 503 })
    ));
    assert_eq!(
        (
            saved(&database, &recording).tier,
            saved(&database, &recording).status
        ),
        (Tier::Team, SubscriptionState::PaymentFailed)
    );
    {
        let mut session = database
            .store
            .try_lock_billing(recording.initial_state.workspace_id)
            .unwrap()
            .unwrap();
        let renewals = session.renewals(id).unwrap();
        assert_eq!(renewals.len(), 1);
        assert_eq!(renewals[0].outcome, RenewalOutcome::Open);
        assert_eq!(
            renewals[0]
                .automatic_stop
                .as_ref()
                .map(|stop| (stop.event_id.as_str(), stop.attempt_count)),
            Some(("evt_1UYGEFAGa5h8DTS3z2H40Eaa", 9))
        );
        assert!(!session.event_applied(stop["id"].as_str().unwrap()).unwrap());
    }
    database.reconnect_store();
    let mut ended = recording.snapshot(id, "canceled");
    ended["cancellation_details"]["reason"] = json!("cancellation_requested");
    fake.expect_subscription(
        SubscriptionRequest::CancelImmediately(id.into()),
        SubscriptionOutcome::respond(serde_json::from_value(ended).unwrap()),
    );
    assert_eq!(
        deliver(&database, &fake, &recording.events[3]),
        EventOutcome::Applied
    );
    let state = saved(&database, &recording);
    assert_eq!(
        (state.tier, state.status),
        (Tier::Free, SubscriptionState::Ended)
    );
    assert_eq!(
        state.ended_banner_until,
        Some("2027-01-19T04:12:02Z".parse().unwrap())
    );
    assert_eq!(deliver(&database, &fake, stop), EventOutcome::Applied);
    assert_eq!(deliver(&database, &fake, stop), EventOutcome::Duplicate);
    let requests = fake
        .state
        .lock()
        .unwrap()
        .requests
        .iter()
        .filter(|request| request.method == "DELETE")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].key, requests[1].key);
    assert_eq!(
        requests[0].key.as_deref(),
        Some("ciproxy:renewal-stop:in_1UYGBnAGa5h8DTS3nWEvyYLM")
    );
}

#[test]
fn unconfirmed_or_wrong_owner_cancellation_never_records_free() {
    for variation in [
        "still_active",
        "different_subscription",
        "different_customer",
    ] {
        let recording = Recording::final_failure();
        let database = recording.seed();
        let id = recording.initial_state.subscription_id.as_deref().unwrap();
        let active = recording.snapshot(id, "active");
        let fake = Fake::new(serde_json::from_value(active.clone()).unwrap());
        let invoice = recording.events[2]["data"]["object"].clone();
        install(&fake, active.clone(), &[invoice]);
        let mut response = recording.snapshot(id, "canceled");
        match variation {
            "still_active" => response = active,
            "different_subscription" => response["id"] = json!("sub_derived_other"),
            "different_customer" => response["customer"] = json!("cus_derived_other"),
            _ => unreachable!(),
        }
        fake.expect_subscription(
            SubscriptionRequest::CancelImmediately(id.into()),
            SubscriptionOutcome::respond(serde_json::from_value(response).unwrap()),
        );
        let result = send(&database, &fake, &recording.events[2]);
        assert_eq!(
            result.err().map(|error| match error {
                Error::Ownership => "ownership",
                Error::StripeState => "unconfirmed",
                _ => "unexpected",
            }),
            Some(if variation == "still_active" {
                "unconfirmed"
            } else {
                "ownership"
            }),
            "{variation}"
        );
        let state = saved(&database, &recording);
        assert_eq!(
            (state.tier, state.status, state.free_started_at),
            (
                Tier::Team,
                SubscriptionState::PaymentFailed,
                recording.initial_state.free_started_at
            ),
            "{variation}"
        );
    }
}

#[test]
fn a_newer_manual_change_prevents_replaying_an_older_automatic_stop() {
    let recording = Recording::final_failure();
    let database = recording.seed();
    let id = recording.initial_state.subscription_id.as_deref().unwrap();
    let stop = &recording.events[2];
    let current = recording.snapshot(id, "active");
    let fake = Fake::new(serde_json::from_value(current.clone()).unwrap());
    install(&fake, current, &[stop["data"]["object"].clone()]);
    fake.expect_subscription(
        SubscriptionRequest::CancelImmediately(id.into()),
        SubscriptionOutcome::Reject(503),
    );
    assert!(matches!(
        send(&database, &fake, stop),
        Err(Error::StripeHttp { status: 503 })
    ));
    let mut manual = stop.clone();
    manual["id"] = json!("evt_derived_newer_manual_stop");
    manual["created"] = json!(stop["created"].as_i64().unwrap() + 1);
    manual["request"]["id"] = json!("req_derived_operator");
    assert_eq!(deliver(&database, &fake, &manual), EventOutcome::Applied);
    assert_eq!(deliver(&database, &fake, stop), EventOutcome::Applied);
    assert_eq!(saved(&database, &recording).tier, Tier::Team);
    let renewals = database
        .store
        .try_lock_billing(recording.initial_state.workspace_id)
        .unwrap()
        .unwrap()
        .renewals(id)
        .unwrap();
    assert_eq!(renewals[0].automatic_stop, None);
    assert_eq!(
        fake.state
            .lock()
            .unwrap()
            .requests
            .iter()
            .filter(|request| request.method == "DELETE")
            .count(),
        1
    );
}
