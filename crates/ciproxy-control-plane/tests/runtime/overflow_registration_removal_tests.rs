//! Owner removal defers registration recovery until a new installation exists.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RegistrationService;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::StartPhase;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::support::at;
use super::takeout_support::{Answer, Call, Factory, Remote};

#[tokio::test]
async fn owner_removal_defers_ended_and_missing_registration_recovery_until_reinstall() {
    let mut harness = Harness::new(false, false).await;
    let mut job = harness.queued();
    harness.runtime.tick().await.unwrap();
    let store = harness.fixture.store.clone();
    let workspace = harness.fixture.workspace;
    let organization = harness.fixture.organization;
    let ended = store.boat_runner_intents(workspace).unwrap().remove(0);
    let mut missing = ended.clone();
    missing.runner_id = Uuid::new_v4();
    missing.delivery_id = Uuid::new_v4();
    missing.github_id = Some(101);
    store
        .create_boat_runner_intent(workspace, &missing)
        .unwrap();
    let expected = store.boat_runner_intents(workspace).unwrap();
    job.phase = DemandPhase::Completed;
    job.completed_at = Some(at());
    store.save_github_job(workspace, &job, at()).unwrap();
    store
        .record_app_removal(workspace, organization, at())
        .unwrap();
    let token = harness.ports.inputs.lock().unwrap()[0]["exit_report_token"]
        .as_str()
        .unwrap()
        .to_owned();
    harness
        .service
        .report_end(Secret::new(token), ended.runner_id)
        .await
        .unwrap();
    let completed = store
        .boat_runtime_snapshot(workspace)
        .unwrap()
        .starts
        .remove(0);
    assert_eq!(completed.phase, StartPhase::Ended);
    assert_eq!(completed.ends_at, Some(at()));
    let remote = Remote::new(Answer::DeleteUnavailable);
    harness.use_registrations(Arc::new(RegistrationService {
        store: Arc::new(StoreAdapter::new(store.clone(), harness.ports.clone())),
        scale_sets: Arc::new(Factory(remote.clone())),
    }));
    for tick in 1..=3 {
        harness.runtime.tick().await.unwrap();
        assert_eq!(
            *remote.calls.lock().unwrap(),
            Vec::<Call>::new(),
            "tick: {tick}"
        );
        assert_eq!(*remote.contexts.lock().unwrap(), Vec::new(), "tick: {tick}");
        assert_eq!(
            store.boat_runner_intents(workspace).unwrap(),
            expected,
            "tick: {tick}"
        );
        assert_eq!(
            store.boat_runtime_snapshot(workspace).unwrap().starts,
            vec![completed.clone()],
            "tick: {tick}"
        );
    }
    let mut reinstalled = store
        .organization(workspace, organization)
        .unwrap()
        .unwrap();
    reinstalled.installation_id = 49;
    store.install_organization(&reinstalled).unwrap();
    *remote.answer.lock().unwrap() = Answer::Removed;
    harness.runtime.tick().await.unwrap();
    assert_eq!(store.boat_runner_intents(workspace).unwrap(), Vec::new());
    let mut calls = remote.calls.lock().unwrap().clone();
    calls.sort_by_key(|call| match call {
        Call::Find => 0,
        Call::Remove(id) => *id,
    });
    assert_eq!(calls, vec![Call::Remove(100), Call::Remove(101)]);
    assert_eq!(
        remote
            .contexts
            .lock()
            .unwrap()
            .iter()
            .map(|(installation, _)| *installation)
            .collect::<Vec<_>>(),
        vec![49; 2]
    );
    assert_eq!(
        store.boat_runtime_snapshot(workspace).unwrap().starts,
        vec![completed]
    );
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.launches.load(Ordering::SeqCst), 1);
}
