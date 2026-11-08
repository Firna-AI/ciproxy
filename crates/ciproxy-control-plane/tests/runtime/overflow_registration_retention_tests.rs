//! Registration recovery shares terminal ownership with normal retention.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::RegistrationService;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::StartPhase;
use uuid::Uuid;

use super::overflow_retention_support::{extra_intent, report_end};
use super::overflow_support::Harness;
use super::overflow_takeout_support::prepare;
use super::support::at;
use super::takeout_support::{Answer, Call, Factory, Remote};

#[tokio::test]
async fn organization_expiry_forgets_only_terminal_owned_registrations() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    harness.queued();
    harness.runtime.tick().await.unwrap();
    let store = harness.fixture.store.clone();
    let workspace = harness.fixture.workspace;
    let organization = harness.fixture.organization;
    let intents = store.boat_runner_intents(workspace).unwrap();
    let ended = &intents[0];
    let live = &intents[1];
    let mut missing = ended.clone();
    missing.runner_id = Uuid::new_v4();
    missing.delivery_id = Uuid::new_v4();
    store
        .create_boat_runner_intent(workspace, &missing)
        .unwrap();
    let mut unknown = missing.clone();
    unknown.runner_id = Uuid::new_v4();
    unknown.delivery_id = Uuid::new_v4();
    store
        .create_boat_runner_intent(workspace, &unknown)
        .unwrap();
    harness
        .fixture
        .clear_boat_installation_authority(unknown.runner_id);
    let (sibling_workspace, sibling) = extra_intent(&harness, ended, false).await;
    let (other_workspace, other) = extra_intent(&harness, ended, true).await;
    store
        .record_app_removal(workspace, organization, at())
        .unwrap();
    report_end(&harness, ended.runner_id).await;
    let remote = Remote::new(Answer::DeleteUnavailable);
    harness.use_registrations(Arc::new(RegistrationService {
        store: Arc::new(StoreAdapter::new(store.clone(), harness.ports.clone())),
        scale_sets: Arc::new(Factory(remote.clone())),
    }));
    assert_eq!(
        store
            .purge_removed_organizations(workspace, at() + Duration::days(29))
            .unwrap(),
        0
    );
    assert_eq!(store.boat_runner_intents(workspace).unwrap().len(), 5);
    assert_eq!(
        store
            .purge_removed_organizations(workspace, at() + Duration::days(30))
            .unwrap(),
        1
    );
    let mut retained = store.boat_runner_intents(workspace).unwrap();
    retained.sort_by_key(|intent| intent.runner_id);
    let mut expected = vec![live.clone(), sibling.clone(), unknown.clone()];
    expected.sort_by_key(|intent| intent.runner_id);
    assert_eq!(retained, expected);
    assert_eq!(
        store.boat_runner_intents(other_workspace).unwrap(),
        vec![other.clone()]
    );
    assert_eq!(store.organization(workspace, organization).unwrap(), None);
    assert_eq!(
        store.boat_runtime_snapshot(workspace).unwrap().starts.len(),
        2
    );
    store
        .remove_boat_runner_intent(sibling_workspace, sibling.runner_id)
        .unwrap();
    store
        .remove_boat_runner_intent(other_workspace, other.runner_id)
        .unwrap();
    report_end(&harness, live.runner_id).await;
    *harness.ports.now.lock().unwrap() = at() + Duration::days(31);
    harness.runtime.overflow = None;
    harness.runtime.boat_retirement = None;
    harness.runtime.tick().await.unwrap();
    assert_eq!(store.boat_runner_intents(workspace).unwrap(), vec![unknown]);
    assert_eq!(*remote.calls.lock().unwrap(), Vec::<Call>::new());
    assert_eq!(
        store
            .boat_runtime_snapshot(workspace)
            .unwrap()
            .starts
            .iter()
            .map(|record| record.phase)
            .collect::<Vec<_>>(),
        vec![StartPhase::Ended; 2]
    );
}

#[tokio::test]
async fn completed_takeout_forgets_terminal_registrations_after_retained_reinstall() {
    let mut harness = Harness::new(false, false).await;
    let mut first_job = harness.queued();
    let mut second_job = harness.queued();
    harness.runtime.tick().await.unwrap();
    let store = harness.fixture.store.clone();
    let workspace = harness.fixture.workspace;
    let organization = harness.fixture.organization;
    let intents = store.boat_runner_intents(workspace).unwrap();
    let first = intents
        .iter()
        .find(|intent| intent.github_id == Some(100))
        .unwrap();
    let second = intents
        .iter()
        .find(|intent| intent.github_id == Some(101))
        .unwrap();
    for (job, id) in [(&mut first_job, 100), (&mut second_job, 101)] {
        job.phase = DemandPhase::Started;
        job.started_at = Some(at());
        job.runner_github_id = Some(id);
        store.save_github_job(workspace, job, at()).unwrap();
    }
    let remote = Remote::new(Answer::Busy);
    let event = prepare(&mut harness, remote.clone());
    harness.runtime.lifecycle(event).await.unwrap();
    let takeout = store.pending_organization_takeouts().unwrap().remove(0);
    store.complete_organization_takeout(&takeout).unwrap();
    first_job.phase = DemandPhase::Completed;
    first_job.completed_at = Some(at());
    store.save_github_job(workspace, &first_job, at()).unwrap();
    report_end(&harness, first.runner_id).await;
    let mut reinstalled = store
        .organization(workspace, organization)
        .unwrap()
        .unwrap();
    reinstalled.installation_id = 49;
    store.install_organization(&reinstalled).unwrap();
    let mut newer = first.clone();
    newer.delivery_id = Uuid::new_v4();
    newer.runner_id = Uuid::new_v4();
    newer.github_id = Some(102);
    store.create_boat_runner_intent(workspace, &newer).unwrap();
    *remote.answer.lock().unwrap() = Answer::DeleteUnavailable;
    remote.calls.lock().unwrap().clear();
    harness.runtime.tick().await.unwrap();
    assert_eq!(store.boat_runner_intents(workspace).unwrap().len(), 3);
    let calls = remote.calls.lock().unwrap().clone();
    assert!(calls.contains(&Call::Remove(100)), "{calls:?}");
    assert!(calls.contains(&Call::Remove(102)), "{calls:?}");
    store
        .begin_organization_takeout(workspace, organization)
        .unwrap();
    let event = store
        .pending_github_lifecycle(42)
        .unwrap()
        .last()
        .unwrap()
        .event
        .clone();
    harness.runtime.tick().await.unwrap();
    assert_eq!(store.boat_runner_intents(workspace).unwrap().len(), 3);
    remote.answers.lock().unwrap().insert(100, Answer::Removed);
    remote.answers.lock().unwrap().insert(101, Answer::Busy);
    remote.answers.lock().unwrap().insert(102, Answer::Removed);
    harness.runtime.lifecycle(event).await.unwrap();
    let takeout = store.pending_organization_takeouts().unwrap().remove(0);
    assert_eq!(takeout.installation_id, 49);
    store.complete_organization_takeout(&takeout).unwrap();
    let history = store.github_jobs(workspace, organization).unwrap();
    let runners = store.runners(workspace, organization).unwrap();
    remote.calls.lock().unwrap().clear();
    remote.contexts.lock().unwrap().clear();
    harness.runtime.tick().await.unwrap();
    assert_eq!(
        store.boat_runner_intents(workspace).unwrap(),
        vec![second.clone()]
    );
    assert_eq!(store.github_jobs(workspace, organization).unwrap(), history);
    assert_eq!(store.runners(workspace, organization).unwrap(), runners);
    report_end(&harness, second.runner_id).await;
    let deliveries = store.boat_runtime_snapshot(workspace).unwrap().starts;
    let runners = store.runners(workspace, organization).unwrap();
    remote.answers.lock().unwrap().clear();
    for tick in 1..=3 {
        harness.runtime.tick().await.unwrap();
        assert_eq!(
            store.boat_runner_intents(workspace).unwrap(),
            Vec::new(),
            "tick: {tick}"
        );
        assert_eq!(
            *remote.calls.lock().unwrap(),
            Vec::<Call>::new(),
            "tick: {tick}"
        );
        assert_eq!(*remote.contexts.lock().unwrap(), Vec::new(), "tick: {tick}");
        assert_eq!(
            store.github_jobs(workspace, organization).unwrap(),
            history,
            "tick: {tick}"
        );
        assert_eq!(
            store.runners(workspace, organization).unwrap(),
            runners,
            "tick: {tick}"
        );
        assert_eq!(
            store.boat_runtime_snapshot(workspace).unwrap().starts,
            deliveries,
            "tick: {tick}"
        );
    }
}
