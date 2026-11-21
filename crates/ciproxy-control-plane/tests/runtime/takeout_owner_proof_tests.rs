//! Only the exact App-JWT installation 404 grants the pending owner exception.

use axum::http::StatusCode;
use ciproxy_control_plane::error::Error;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use ciproxy_store_interface::runners::RunnerState;

use super::recovery_support::{other_organization, saved};
use super::settled::Settled;
use super::support::{Fixture, at};
use super::takeout_gateway::Capture;
use super::takeout_installation_http::Http;
use super::takeout_support::{Answer, Call, Remote, real_runtime};

#[tokio::test]
async fn exact_app_jwt_404_during_pending_takeout_applies_owner_authority_without_registration() {
    let fixture = Fixture::new();
    other_organization(&fixture);
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Busy,
            Some(47),
        )
        .unwrap();
    fixture
        .store
        .begin_organization_takeout(fixture.workspace, fixture.organization)
        .unwrap();
    fixture
        .store
        .remove_github_installation(fixture.workspace, fixture.organization, 43, at())
        .unwrap();
    let pending = fixture.store.pending_github_lifecycle(42).unwrap();
    assert_eq!(pending.len(), 1);
    assert!(
        matches!(&pending[0].event, LifecycleEvent::Removed { cause: RemovalCause::WorkspaceTakeout, organization, .. } if organization.removed_at.is_none())
    );
    let remote = Remote::new(Answer::Busy);
    let gateway = Capture::new(false);
    let http = Http::start(StatusCode::NOT_FOUND).await;
    let mut runtime = real_runtime(&fixture, remote.clone(), gateway.clone());
    runtime.installations = http.api.clone();
    runtime
        .settled_lifecycle(pending[0].event.clone())
        .await
        .unwrap();
    assert_eq!(
        *http.requests.lock().unwrap(),
        vec![("/app/installations/43".to_owned(), true)]
    );
    assert_eq!(*remote.calls.lock().unwrap(), Vec::<Call>::new());
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    let messages = gateway.messages.lock().unwrap();
    assert_eq!(messages.len(), 1);
    assert!(
        matches!(&messages[0], ControlMessage::StopRunner { organization_id, runner_id, .. } if organization_id.0 == "42" && runner_id.0 == fixture.intent.runner.id.to_string())
    );
}

#[tokio::test]
async fn app_jwt_403_is_unavailable_and_grants_no_owner_or_ordinary_stop_authority() {
    let fixture = Fixture::new();
    other_organization(&fixture);
    fixture
        .store
        .begin_organization_takeout(fixture.workspace, fixture.organization)
        .unwrap();
    let event = fixture
        .store
        .pending_github_lifecycle(42)
        .unwrap()
        .remove(0)
        .event;
    let remote = Remote::new(Answer::Busy);
    let gateway = Capture::new(false);
    let http = Http::start(StatusCode::FORBIDDEN).await;
    let mut runtime = real_runtime(&fixture, remote.clone(), gateway.clone());
    runtime.installations = http.api.clone();
    let result = runtime.settled_lifecycle(event).await;
    assert!(matches!(result, Err(Error::Internal(_))), "{result:?}");
    assert_eq!(
        *http.requests.lock().unwrap(),
        vec![("/app/installations/43".to_owned(), true)]
    );
    assert_eq!(*remote.calls.lock().unwrap(), Vec::<Call>::new());
    assert_eq!(saved(&fixture).runner.state, RunnerState::Ready);
    assert_eq!(gateway.messages.lock().unwrap().len(), 0);
    assert_eq!(fixture.store.pending_github_lifecycle(42).unwrap().len(), 1);
}
