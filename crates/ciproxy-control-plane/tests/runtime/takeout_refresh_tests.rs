//! Pending positive installation proof refreshes before owner-removal recovery.

use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_store_interface::runners::RunnerState;

use super::overflow_support::Harness;
use super::overflow_takeout_support::prepare;
use super::recovery_support::saved;
use super::support::at;
use super::takeout_installation_http::Http;
use super::takeout_support::{Answer, Call, Remote};

#[tokio::test]
async fn pending_takeout_refreshes_positive_installation_proof_and_stops_only_after_exact_404() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.tick().await.unwrap();
    harness.runtime.quiesce().await.unwrap();
    let intents = harness
        .fixture
        .store
        .boat_runner_intents(harness.fixture.workspace)
        .unwrap();
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0].github_id, Some(100));
    let boat_runner = intents[0].runner_id;
    harness
        .fixture
        .store
        .update_runner(
            harness.fixture.workspace,
            harness.fixture.organization,
            harness.fixture.intent.runner.id,
            RunnerState::Ready,
            Some(47),
        )
        .unwrap();
    let remote = Remote::new(Answer::DeleteUnavailable);
    let event = prepare(&mut harness, remote.clone());
    let http = Http::start(StatusCode::OK).await;
    harness.runtime.overflow = None;
    harness.runtime.installations = http.api.clone();
    let fixture = &harness.fixture;
    let runtime = &harness.runtime;
    let pending = runtime.lifecycle(event.clone()).await;
    assert!(matches!(pending, Err(Error::StateChanged)), "{pending:?}");
    let failed = runtime.quiesce().await;
    assert!(matches!(failed, Err(Error::StateChanged)), "{failed:?}");
    let calls = remote.calls.lock().unwrap().clone();
    assert_eq!(
        calls.len(),
        2,
        "failed={failed:?}, http_requests={:?}, http_status={:?}, own={:?}, boat={:?}",
        http.requests.lock().unwrap(),
        http.status.lock().unwrap(),
        saved(fixture),
        fixture
            .store
            .boat_runner_guard(fixture.workspace, boat_runner)
    );
    assert!(calls.contains(&Call::Remove(47)));
    assert!(calls.contains(&Call::Remove(100)));
    assert_eq!(saved(fixture).runner.state, RunnerState::Ready);
    assert_eq!(harness.own_starts.load(Ordering::SeqCst), 0);
    http.entered.notified().await;
    let before = fixture
        .store
        .organization(fixture.workspace, fixture.organization)
        .unwrap()
        .unwrap();
    fixture
        .store
        .remove_github_installation(fixture.workspace, fixture.organization, 43, at())
        .unwrap();
    assert_eq!(
        fixture
            .store
            .organization(fixture.workspace, fixture.organization)
            .unwrap()
            .unwrap(),
        before
    );
    assert_eq!(fixture.store.pending_github_lifecycle(42).unwrap().len(), 1);
    http.hold.store(true, Ordering::SeqCst);
    *http.status.lock().unwrap() = StatusCode::NOT_FOUND;
    let pending = runtime.lifecycle(event.clone()).await;
    assert!(matches!(pending, Err(Error::StateChanged)), "{pending:?}");
    assert_eq!(saved(fixture).runner.state, RunnerState::Ready);
    assert_eq!(saved(fixture).command_id, fixture.intent.command_id);
    assert_eq!(fixture.store.pending_github_lifecycle(42).unwrap().len(), 1);
    http.release.add_permits(1);
    let refreshed = runtime.quiesce().await;
    assert!(
        matches!(refreshed, Ok(()) | Err(Error::StateChanged)),
        "{refreshed:?}"
    );
    assert_eq!(
        *http.requests.lock().unwrap(),
        vec![
            ("/app/installations/43".to_owned(), true),
            ("/app/installations/43".to_owned(), true),
        ]
    );
    runtime.lifecycle(event.clone()).await.unwrap();
    runtime.quiesce_lifecycle(&event).await.unwrap();
    assert_eq!(saved(fixture).runner.state, RunnerState::Stopping);
    assert_ne!(saved(fixture).command_id, fixture.intent.command_id);
    assert_eq!(fixture.store.pending_github_lifecycle(42).unwrap().len(), 1);
    runtime.lifecycle(event.clone()).await.unwrap();
    runtime.quiesce_lifecycle(&event).await.unwrap();
    assert_eq!(http.requests.lock().unwrap().len(), 2);
}
