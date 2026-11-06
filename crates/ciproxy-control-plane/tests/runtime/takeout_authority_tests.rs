//! A pending take-out must ask GitHub before it stops a warm VM.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::runners::RunnerState;

use super::recovery_support::{gateway, other_organization, saved};
use super::support::Fixture;
use super::takeout_support::{Answer, Call, Remote, real_runtime};

async fn busy_on_tick(saved_id: Option<i64>) {
    let fixture = Fixture::new();
    other_organization(&fixture);
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            fixture.intent.runner.id,
            RunnerState::Ready,
            saved_id,
        )
        .unwrap();
    fixture
        .store
        .begin_organization_takeout(fixture.workspace, fixture.organization)
        .unwrap();
    let remote = Remote::new(Answer::Busy);
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = real_runtime(&fixture, remote.clone(), gateway(messages.clone()));
    runtime.tick().await.unwrap();
    assert_eq!(
        *remote.calls.lock().unwrap(),
        if saved_id.is_some() {
            vec![Call::Remove(47)]
        } else {
            vec![Call::Find, Call::Remove(47)]
        }
    );
    assert_eq!(
        *remote.contexts.lock().unwrap(),
        vec![(43, AppState::Removed)]
    );
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
    assert_eq!(saved(&fixture).runner.github_id, Some(47));
    assert_eq!(
        messages
            .lock()
            .unwrap()
            .iter()
            .filter(|message| matches!(message, ControlMessage::StopRunner { .. }))
            .count(),
        0
    );
}

#[tokio::test]
async fn pending_takeout_tick_checks_saved_registration_and_preserves_remote_busy_job() {
    busy_on_tick(Some(47)).await;
}

#[tokio::test]
async fn pending_takeout_tick_looks_up_unsaved_registration_and_preserves_remote_busy_job() {
    busy_on_tick(None).await;
}
