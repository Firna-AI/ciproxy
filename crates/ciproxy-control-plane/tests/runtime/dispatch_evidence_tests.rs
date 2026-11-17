//! Caller evidence and PostgreSQL time precision do not reject current dispatch.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_github::registrations::RegistrationRetirement;
use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_store_interface::runners::RunnerState;

use super::held_remote::HeldRemote;
use super::overflow_ports::Ports;
use super::progress_support::reserve_warm;
use super::recovery_support::{gateway, runtime_with_clock};
use super::settled::Settled;
use super::support::{Fixture, at};

#[tokio::test]
async fn new_start_with_submicrosecond_issue_time_reaches_github_and_the_machine() {
    let fixture = Fixture::new();
    reserve_warm(&fixture);
    let clock = Ports::new();
    *clock.now.lock().unwrap() = at() + Duration::nanoseconds(123);
    let remote = HeldRemote::new(
        fixture.workspace,
        true,
        RegistrationRetirement::Removed { id: 100 },
    );
    remote.release.add_permits(1);
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime_with_clock(&fixture, remote.clone(), gateway(messages.clone()), clock);
    runtime.settled_tick().await.unwrap();
    assert_eq!(remote.creates.load(Ordering::SeqCst), 1);
    let runner = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|runner| runner.runner.state == RunnerState::Starting)
        .unwrap();
    assert_eq!(runner.issued_at, at());
    assert_eq!(runner.confirmed_at, None);
    assert!(!runner.acknowledged);
    assert_eq!(runner.runner.github_id, Some(100));
    assert_eq!(runner.registration_scale_set_id, Some(44));
    let messages = messages.lock().unwrap();
    let starts: Vec<_> = messages
        .iter()
        .filter(|message| matches!(message, ControlMessage::StartRunner { .. }))
        .collect();
    assert_eq!(starts.len(), 1);
    assert!(
        matches!(starts[0], ControlMessage::StartRunner { command_id, runner: spec }
            if command_id.0 == runner.command_id.to_string()
                && spec.runner_id.0 == runner.runner.id.to_string())
    );
}

#[tokio::test]
async fn first_failed_start_result_retires_before_a_new_full_report() {
    let fixture = Fixture::new();
    reserve_warm(&fixture);
    let clock = Ports::new();
    let remote = HeldRemote::new(
        fixture.workspace,
        true,
        RegistrationRetirement::Removed { id: 100 },
    );
    remote.release.add_permits(1);
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime_with_clock(
        &fixture,
        remote.clone(),
        gateway(messages.clone()),
        clock.clone(),
    );
    runtime.settled_tick().await.unwrap();
    let runner = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|runner| runner.runner.state == RunnerState::Starting)
        .unwrap();
    assert_eq!(runner.confirmed_at, None);
    assert!(!runner.acknowledged);
    *clock.now.lock().unwrap() = at() + Duration::nanoseconds(123);
    let result = runtime
        .settled_agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::CommandResult {
                command_id: CommandId(runner.command_id.to_string()),
                outcome: CommandOutcome::Failed {
                    reason: CommandFailure::RunnerStart,
                },
            },
        )
        .await;
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(*remote.retirements.lock().unwrap(), vec![Some(100)]);
    let saved = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .into_iter()
        .find(|saved| saved.runner.id == runner.runner.id)
        .unwrap();
    assert_eq!(saved.runner.state, RunnerState::Stopping);
    assert_eq!(saved.confirmed_at, Some(at()));
    assert!(!saved.acknowledged);
    assert_ne!(saved.command_id, runner.command_id);
    assert_eq!(saved.slot_indices, runner.slot_indices);
    assert_eq!(saved.runner.size, runner.runner.size);
    let messages = messages.lock().unwrap();
    let stops: Vec<_> = messages
        .iter()
        .filter(|message| matches!(message, ControlMessage::StopRunner { .. }))
        .collect();
    assert_eq!(stops.len(), 1);
    assert!(
        matches!(stops[0], ControlMessage::StopRunner { command_id, runner_id, .. }
            if command_id.0 == saved.command_id.to_string()
                && runner_id.0 == saved.runner.id.to_string())
    );
}
