//! A late GitHub answer cannot replace a newer authenticated execution observation.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_store_interface::runners::RunnerState;

use super::held_remote::HeldRemote;
use super::progress_support::reserve_warm;
use super::recovery_support::{gateway, runtime, saved};
use super::support::Fixture;

#[tokio::test]
async fn late_retirement_results_preserve_newer_busy_and_terminal_agent_observations() {
    for (answer, exited, state) in [
        (
            RegistrationRetirement::Removed { id: 47 },
            false,
            RunnerState::Busy,
        ),
        (
            RegistrationRetirement::Busy { id: 47 },
            true,
            RunnerState::Stopped,
        ),
    ] {
        let fixture = Fixture::new();
        let remote = HeldRemote::new(fixture.workspace, false, answer);
        let messages = Arc::new(Mutex::new(Vec::new()));
        let runtime = runtime(&fixture, remote.clone(), gateway(messages.clone()));
        runtime.tick().await.unwrap();
        remote.entered.notified().await;
        runtime
            .agent(
                fixture.workspace,
                fixture.machine,
                AgentMessage::JobStarted {
                    runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                    job_id: JobId("new-job".into()),
                },
            )
            .await
            .unwrap();
        if exited {
            runtime
                .agent(
                    fixture.workspace,
                    fixture.machine,
                    AgentMessage::RunnerExited {
                        runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                        exit: ciproxy_protocol::runners::RunnerExit { code: Some(0) },
                        usage: None,
                    },
                )
                .await
                .unwrap();
            runtime
                .agent(
                    fixture.workspace,
                    fixture.machine,
                    AgentMessage::RunnerReady {
                        runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                    },
                )
                .await
                .unwrap();
        }
        remote.release.add_permits(1);
        runtime.quiesce().await.unwrap();
        let current = saved(&fixture);
        assert_eq!(current.runner.state, state);
        assert_eq!(current.command_id, fixture.intent.command_id);
        assert_eq!(current.runner.github_id, Some(47));
        assert_eq!(current.slot_indices, vec![0]);
        assert_eq!(current.runner.size, fixture.intent.runner.size);
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
}

#[tokio::test]
async fn late_create_after_saved_absence_keeps_cleanup_identity_and_never_sends_start() {
    let fixture = Fixture::new();
    reserve_warm(&fixture);
    let remote = HeldRemote::new(
        fixture.workspace,
        true,
        RegistrationRetirement::Removed { id: 100 },
    );
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime(&fixture, remote.clone(), gateway(messages.clone()));
    runtime.tick().await.unwrap();
    remote.entered.notified().await;
    let pending = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|runner| runner.runner.github_id.is_none())
        .unwrap();
    fixture
        .store
        .update_runner(
            fixture.workspace,
            fixture.organization,
            pending.runner.id,
            RunnerState::Stopped,
            None,
        )
        .unwrap();
    remote.release.add_permits(1);
    runtime.quiesce().await.unwrap();
    let ended = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|runner| runner.runner.id == pending.runner.id)
        .unwrap();
    assert_eq!(ended.runner.state, RunnerState::Stopped);
    assert_eq!(ended.command_id, pending.command_id);
    assert_eq!(ended.runner.github_id, Some(100));
    assert_eq!(*remote.retirements.lock().unwrap(), vec![Some(100)]);
    assert_eq!(
        messages
            .lock()
            .unwrap()
            .iter()
            .filter(|message| matches!(message, ControlMessage::StartRunner { .. }))
            .count(),
        0
    );
}
