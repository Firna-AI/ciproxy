//! Take-out uses durable retirement proof across both runner destinations.

use std::sync::atomic::Ordering;

use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runners::RunnerState;
use uuid::Uuid;

use super::overflow_support::Harness;
use super::overflow_takeout_support::prepare;
use super::recovery_support::saved;
use super::support::at;
use super::takeout_support::{Answer, Call, Remote};

#[tokio::test]
async fn takeout_waits_for_boat_retirement_and_saves_idle_and_busy_answers() {
    let mut harness = Harness::new(false, false).await;
    let mut idle = harness.queued();
    let mut running = harness.queued();
    harness.runtime.tick().await.unwrap();
    let store = harness.fixture.store.clone();
    let workspace = harness.fixture.workspace;
    let organization = harness.fixture.organization;
    let intents = store.boat_runner_intents(workspace).unwrap();
    let idle_id = intents
        .iter()
        .find(|intent| intent.github_id == Some(100))
        .unwrap()
        .runner_id;
    let running_id = intents
        .iter()
        .find(|intent| intent.github_id == Some(101))
        .unwrap()
        .runner_id;
    idle.phase = DemandPhase::Completed;
    idle.completed_at = Some(at());
    running.phase = DemandPhase::Started;
    running.started_at = Some(at());
    running.runner_github_id = Some(101);
    store.save_github_job(workspace, &idle, at()).unwrap();
    store.save_github_job(workspace, &running, at()).unwrap();
    store
        .update_runner(
            workspace,
            organization,
            idle_id,
            RunnerState::Ready,
            Some(100),
        )
        .unwrap();
    store
        .update_runner(
            workspace,
            organization,
            harness.fixture.intent.runner.id,
            RunnerState::Ready,
            Some(47),
        )
        .unwrap();
    let remote = Remote::new(Answer::Removed);
    remote
        .answers
        .lock()
        .unwrap()
        .insert(100, Answer::DeleteUnavailable);
    remote.answers.lock().unwrap().insert(101, Answer::Busy);
    let event = prepare(&mut harness, remote.clone());
    let result = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    assert_eq!(store.pending_github_lifecycle(42).unwrap().len(), 1);
    assert_eq!(saved(&harness.fixture).runner.state, RunnerState::Stopping);
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 0);
    assert_eq!(
        store
            .boat_runtime_snapshot(workspace)
            .unwrap()
            .starts
            .iter()
            .map(|record| (record.phase, record.ends_at))
            .collect::<Vec<_>>(),
        vec![(StartPhase::Running, None); 2]
    );
    assert_eq!(
        store
            .runners(workspace, organization)
            .unwrap()
            .iter()
            .find(|runner| runner.id == running_id)
            .unwrap()
            .state,
        RunnerState::Busy
    );

    remote.answers.lock().unwrap().insert(100, Answer::Removed);
    harness.ports.delete_fail.store(true, Ordering::SeqCst);
    remote
        .answers
        .lock()
        .unwrap()
        .insert(101, Answer::DeleteUnavailable);
    let result = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    remote.answers.lock().unwrap().insert(101, Answer::Busy);
    harness.runtime.lifecycle(event.clone()).await.unwrap();
    let retiring = store.boat_runtime_snapshot(workspace).unwrap();
    assert_eq!(
        retiring
            .starts
            .iter()
            .find(|record| record.runner_id == idle_id)
            .map(|record| (record.phase, record.ends_at)),
        Some((StartPhase::Deleting, None))
    );
    assert_eq!(
        retiring
            .starts
            .iter()
            .find(|record| record.runner_id == running_id)
            .map(|record| (record.phase, record.ends_at)),
        Some((StartPhase::Running, None))
    );
    assert_eq!(
        store
            .boat_runner_intents(workspace)
            .unwrap()
            .iter()
            .map(|intent| intent.runner_id)
            .collect::<Vec<_>>(),
        vec![running_id]
    );
    let idle_removals = remote
        .calls
        .lock()
        .unwrap()
        .iter()
        .filter(|call| **call == Call::Remove(100))
        .count();
    harness.runtime.lifecycle(event).await.unwrap();
    assert_eq!(
        remote
            .calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| **call == Call::Remove(100))
            .count(),
        idle_removals
    );
    harness.ports.delete_fail.store(false, Ordering::SeqCst);
    harness.service.cleanup(workspace).await.unwrap();
    let retired = store.boat_runtime_snapshot(workspace).unwrap();
    assert_eq!(
        retired
            .starts
            .iter()
            .find(|record| record.runner_id == idle_id)
            .map(|record| (record.phase, record.ends_at)),
        Some((StartPhase::Ended, Some(at())))
    );
    assert_eq!(
        retired
            .starts
            .iter()
            .find(|record| record.runner_id == running_id)
            .map(|record| (record.phase, record.ends_at)),
        Some((StartPhase::Running, None))
    );
    assert_eq!(
        store
            .github_jobs(workspace, organization)
            .unwrap()
            .iter()
            .find(|job| job.id == running.id)
            .unwrap()
            .phase,
        DemandPhase::Started
    );
    running.phase = DemandPhase::Completed;
    running.completed_at = Some(at());
    store.save_github_job(workspace, &running, at()).unwrap();
    remote.answers.lock().unwrap().insert(101, Answer::Removed);
    harness.runtime.job(workspace, running).await.unwrap();
    assert_eq!(
        store
            .boat_runtime_snapshot(workspace)
            .unwrap()
            .starts
            .iter()
            .map(|record| (record.phase, record.ends_at))
            .collect::<Vec<_>>(),
        vec![(StartPhase::Ended, Some(at())); 2]
    );
    assert_eq!(harness.ports.launches.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn takeout_keeps_ambiguous_starts_pending_and_retires_ended_or_absent_intents() {
    let mut harness = Harness::new(false, false).await;
    let mut old = harness.queued();
    harness.runtime.tick().await.unwrap();
    let workspace = harness.fixture.workspace;
    let store = harness.fixture.store.clone();
    let ended = store.boat_runner_intents(workspace).unwrap().remove(0);
    old.phase = DemandPhase::Completed;
    old.completed_at = Some(at());
    store.save_github_job(workspace, &old, at()).unwrap();
    harness.service.cleanup(workspace).await.unwrap();
    harness.ports.pending.store(true, Ordering::SeqCst);
    harness.queued();
    harness.runtime.tick().await.unwrap();
    store.create_boat_runner_intent(workspace, &ended).unwrap();
    let mut absent = ended.clone();
    absent.delivery_id = Uuid::new_v4();
    absent.runner_id = Uuid::new_v4();
    absent.github_id = None;
    store.create_boat_runner_intent(workspace, &absent).unwrap();
    let pending = store
        .boat_runner_intents(workspace)
        .unwrap()
        .into_iter()
        .find(|intent| intent.github_id == Some(101))
        .unwrap();
    let remote = Remote::new(Answer::LookupUnavailable);
    remote
        .answers
        .lock()
        .unwrap()
        .insert(100, Answer::DeleteUnavailable);
    let event = prepare(&mut harness, remote.clone());
    let result = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    assert_eq!(store.boat_runner_intents(workspace).unwrap().len(), 3);
    assert_eq!(store.pending_github_lifecycle(42).unwrap().len(), 1);
    *remote.answer.lock().unwrap() = Answer::Busy;
    remote.answers.lock().unwrap().insert(100, Answer::Removed);
    let result = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    let held = store.boat_runner_intents(workspace).unwrap();
    assert_eq!(held.len(), 2);
    assert_eq!(
        held.iter()
            .find(|intent| intent.runner_id == absent.runner_id)
            .unwrap()
            .github_id,
        Some(47)
    );
    assert_eq!(
        store
            .runners(workspace, harness.fixture.organization)
            .unwrap()
            .iter()
            .filter(|runner| runner.id == absent.runner_id)
            .count(),
        0
    );
    *remote.answer.lock().unwrap() = Answer::Absent;
    remote.answers.lock().unwrap().insert(100, Answer::Removed);
    let result = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    assert_eq!(
        store.boat_runner_intents(workspace).unwrap(),
        vec![pending.clone()]
    );
    let record = store
        .boat_runtime_snapshot(workspace)
        .unwrap()
        .starts
        .into_iter()
        .find(|record| record.delivery_id == pending.delivery_id)
        .unwrap();
    assert_eq!(record.phase, StartPhase::Creating);
    assert_eq!(record.sandbox, None);
    assert_eq!(record.ends_at, None);
    store
        .remove_boat_runner_intent(workspace, pending.runner_id)
        .unwrap();
    let result = harness.runtime.lifecycle(event).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    assert_eq!(store.boat_runner_intents(workspace).unwrap().len(), 0);
    assert_eq!(harness.ports.creates.lock().unwrap().len(), 2);
    assert_eq!(harness.ports.launches.load(Ordering::SeqCst), 1);
    assert_eq!(harness.ports.deletes.load(Ordering::SeqCst), 1);
    assert_eq!(store.pending_github_lifecycle(42).unwrap().len(), 1);
}
