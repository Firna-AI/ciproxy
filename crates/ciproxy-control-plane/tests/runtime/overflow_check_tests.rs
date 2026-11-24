//! Ordered fallback and warm capacity around provider checks.

use std::slice;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::labels::LabelPlace;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::recovery_support::{gateway, keep_warm};
use super::settled::Settled;

async fn save_unsent(harness: &Harness) -> StartRecord {
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    let mut label = store.labels(workspace).unwrap().remove(0);
    label.order.swap(0, 1);
    store.update_label(workspace, &label).unwrap();
    harness
        .ports
        .maintenance_unavailable
        .store(true, Ordering::SeqCst);
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    let mut saved = store.overflow_runtime_snapshot(workspace).unwrap();
    assert_eq!(saved.starts.len(), 1);
    assert_eq!(saved.starts[0].phase, StartPhase::Unsent);
    saved.starts.remove(0)
}

#[tokio::test]
async fn failed_check_with_a_valid_catalog_withdraws_unsent_before_machine_fallback() {
    let harness = Harness::new(true, false).await;
    let pending = save_unsent(&harness).await;
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    harness.ports.fail.store(true, Ordering::SeqCst);

    harness.runtime.settled_tick().await.unwrap();

    let saved = store.overflow_runtime_snapshot(workspace).unwrap();
    assert_eq!(saved.accounts[0].state.sizes[0].name, "large");
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 0);
    assert_eq!(saved.starts.len(), 1);
    assert_eq!(saved.starts[0].delivery_id, pending.delivery_id);
    assert_eq!(saved.starts[0].phase, StartPhase::CancelledUnsent);
}

#[tokio::test]
async fn healthy_unsent_delivery_keeps_its_id_through_cleanup_and_provider_check() {
    let harness = Harness::new(true, false).await;
    let pending = save_unsent(&harness).await;
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;

    harness.service.cleanup(workspace).await.unwrap();
    let after_cleanup = store.overflow_runtime_snapshot(workspace).unwrap();
    assert_eq!(after_cleanup.starts.as_slice(), slice::from_ref(&pending));
    harness
        .ports
        .maintenance_unavailable
        .store(false, Ordering::SeqCst);
    harness.runtime.settled_tick().await.unwrap();

    let saved = store.overflow_runtime_snapshot(workspace).unwrap();
    assert_eq!(saved.starts.len(), 1);
    assert_eq!(saved.starts[0].delivery_id, pending.delivery_id);
    assert_eq!(saved.starts[0].phase, StartPhase::Running);
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 0);
    let creates = harness.ports.creates.lock().unwrap();
    assert_eq!(creates.len(), 1);
    assert_eq!(creates[0].operation_id.as_uuid(), pending.delivery_id);
}

#[tokio::test]
async fn a_delivery_that_reaches_creating_cannot_be_withdrawn() {
    let harness = Harness::new(true, false).await;
    let pending = save_unsent(&harness).await;
    let store = &harness.fixture.store;
    let workspace = harness.fixture.workspace;
    harness.ports.pending.store(true, Ordering::SeqCst);
    harness
        .ports
        .maintenance_unavailable
        .store(false, Ordering::SeqCst);
    harness
        .service
        .resume(workspace, pending.delivery_id)
        .await
        .unwrap();
    let creating = store.overflow_runtime_snapshot(workspace).unwrap().starts;
    assert_eq!(creating[0].phase, StartPhase::Creating);

    assert!(
        !harness
            .service
            .withdraw_unsent(workspace, pending.delivery_id)
            .await
            .unwrap(),
        "a delivery in Creating was withdrawn"
    );
    assert_eq!(
        store.overflow_runtime_snapshot(workspace).unwrap().starts,
        creating
    );
}

#[tokio::test]
async fn provider_first_demand_stops_a_warm_runner_only_when_the_provider_is_held() {
    for (held, boat_creates, warm_stops) in [(false, 1, 0), (true, 0, 1)] {
        let mut harness = Harness::new(true, false).await;
        let messages = Arc::new(Mutex::new(Vec::new()));
        harness.runtime.gateway = gateway(messages.clone());
        let fixture = &harness.fixture;
        let store = &fixture.store;
        let workspace = fixture.workspace;
        let mut warm = store.labels(workspace).unwrap().remove(0);
        let mut provider_first = warm.clone();
        warm.size = fixture.intent.requested_size;
        store.update_label(workspace, &warm).unwrap();
        keep_warm(fixture);
        store
            .update_runner(
                workspace,
                fixture.organization,
                fixture.intent.runner.id,
                RunnerState::Ready,
                Some(47),
            )
            .unwrap();
        let mut contact = fixture.contact.clone();
        contact.report.capacity = Slots(1);
        store
            .record_contact(workspace, fixture.machine, &contact)
            .unwrap();

        provider_first.id = Uuid::new_v4();
        provider_first.name = "provider-first".to_owned();
        provider_first.order = vec![
            LabelPlace::Provider(OverflowProvider::Boat),
            LabelPlace::Machine,
        ];
        store.create_label(workspace, &provider_first).unwrap();
        let binding = ScaleSet {
            id: Uuid::new_v4(),
            organization_id: fixture.organization,
            label_id: provider_first.id,
            github_id: 60,
            runner_group_id: 45,
        };
        store.create_scale_set(workspace, &binding).unwrap();
        if held {
            store
                .save_overflow_provider_spend_limit(workspace, OverflowProvider::Boat, Some(50_000))
                .unwrap();
        }
        harness.queued_for(provider_first.id, binding.id);

        harness.runtime.settled_tick().await.unwrap();

        let messages = messages.lock().unwrap();
        let stops = messages
            .iter()
            .filter(|message| matches!(message, ControlMessage::StopRunner { runner_id, .. } if runner_id.0 == fixture.intent.runner.id.to_string()))
            .count();
        assert_eq!(stops, warm_stops, "provider held: {held}");
        assert_eq!(
            harness.ports.creates.lock().unwrap().len(),
            boat_creates,
            "provider held: {held}"
        );
    }
}
