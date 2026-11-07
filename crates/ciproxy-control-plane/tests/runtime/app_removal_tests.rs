//! Immutable owner removals bypass GitHub while take-outs preserve job protection.

use std::sync::{Arc, Mutex};

use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use super::clock::clock;
use super::recovery_support::{
    gateway, job, other_organization, runtime, runtime_with_clock, saved,
};
use super::support::{Fixture, at};
use super::takeout_gateway::Capture;
use super::takeout_installations::Probe;

fn remove(fixture: &Fixture) -> LifecycleEvent {
    fixture
        .store
        .remove_github_installation(fixture.workspace, fixture.organization, 43, at())
        .unwrap();
    fixture
        .store
        .pending_github_lifecycle(42)
        .unwrap()
        .remove(0)
        .event
}

#[tokio::test]
async fn owner_removal_stops_hosted_busy_warm_vm_and_replays_immutable_command() {
    let fixture = Fixture::new();
    other_organization(&fixture);
    let job = job(&fixture);
    fixture
        .store
        .save_github_job(fixture.workspace, &job, at())
        .unwrap();
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
    let event = remove(&fixture);
    let messages = Arc::new(Mutex::new(Vec::new()));
    let runtime = runtime(
        &fixture,
        Arc::new(Unimock::new(())),
        gateway(messages.clone()),
    );
    runtime.lifecycle(event.clone()).await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    let command = saved(&fixture).command_id;
    runtime.lifecycle(event).await.unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    let stops: Vec<_> = messages
        .lock()
        .unwrap()
        .iter()
        .filter_map(|message| match message {
            ControlMessage::StopRunner {
                command_id,
                organization_id,
                ..
            } => Some((command_id.0.clone(), organization_id.0.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(stops, vec![(command.to_string(), "42".to_owned()); 3]);
}

#[tokio::test]
async fn owner_removal_lease_conflict_keeps_the_event_pending() {
    let fixture = Fixture::new();
    other_organization(&fixture);
    let event = remove(&fixture);
    let _lease = fixture
        .store
        .try_lock_own_runners(fixture.workspace)
        .unwrap()
        .unwrap();
    let runtime = runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
    );
    let result = runtime.lifecycle(event).await;
    assert!(matches!(result, Err(Error::StateChanged)), "{result:?}");
    assert_eq!(fixture.store.pending_github_lifecycle(42).unwrap().len(), 1);
}

#[tokio::test]
async fn removal_saves_an_unregistered_start_and_accepts_offline_or_last_machine_delivery() {
    for last in [false, true] {
        let fixture = Fixture::new();
        if !last {
            other_organization(&fixture);
        }
        fixture
            .store
            .update_runner(
                fixture.workspace,
                fixture.organization,
                fixture.intent.runner.id,
                RunnerState::Starting,
                None,
            )
            .unwrap();
        let event = remove(&fixture);
        let commands = Arc::new(Mutex::new(Vec::new()));
        let observed = commands.clone();
        let gateway = Arc::new(Unimock::new(
            MachineGatewayMock::send
                .each_call(matching!(_, _, _))
                .answers_arc(Arc::new(move |_, _, _, message| {
                    assert!(matches!(message, ControlMessage::StopRunner { .. }));
                    observed.lock().unwrap().push(message);
                    if last {
                        Err(ciproxy_gateway::error::Error::NotFound)
                    } else {
                        Ok(ciproxy_gateway::types::Delivery::Offline)
                    }
                })),
        ));
        let runtime = runtime(&fixture, Arc::new(Unimock::new(())), gateway);
        runtime.lifecycle(event.clone()).await.unwrap();
        let stopping = saved(&fixture);
        assert_eq!(
            stopping.runner.state,
            RunnerState::Stopping,
            "last organization: {last}"
        );
        assert_eq!(stopping.runner.github_id, None);
        runtime.lifecycle(event).await.unwrap();
        assert_eq!(saved(&fixture).command_id, stopping.command_id);
        assert_eq!(saved(&fixture).issued_at, stopping.issued_at);
        assert_eq!(commands.lock().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn workspace_takeout_keeps_a_busy_runner_and_records_the_distinct_cause() {
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
    let event = fixture
        .store
        .pending_github_lifecycle(42)
        .unwrap()
        .remove(0)
        .event;
    assert!(matches!(
        &event,
        LifecycleEvent::Removed {
            cause: ciproxy_store_interface::github_lifecycle::RemovalCause::WorkspaceTakeout,
            ..
        }
    ));
    let messages = Capture::new(false);
    let mut runtime = runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        messages.clone(),
        clock(false),
    );
    runtime.installations = Probe::present();
    runtime.lifecycle(event).await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
    assert_eq!(
        messages
            .messages
            .lock()
            .unwrap()
            .iter()
            .filter(|message| matches!(message, ControlMessage::StopRunner { .. }))
            .count(),
        0
    );
}
