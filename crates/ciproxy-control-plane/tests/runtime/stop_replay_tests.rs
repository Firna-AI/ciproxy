//! Saved retirement survives delivery loss and unrelated ready messages.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use ciproxy_control_plane::runtime::OwnMachines;
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::JobAssignmentsMock;
use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrationsMock};
use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use ciproxy_store_interface::runners::{RunnerPlace, RunnerState};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::clock::clock;
use super::recovery_support::{gateway, job, reconnect, runtime, runtime_with_clock, saved};
use super::support::{Fixture, at};

#[tokio::test]
async fn offline_stop_replays_same_identity_until_applied_and_preserves_capacity() {
    let fixture = Fixture::new();
    let stops = Arc::new(Mutex::new(Vec::new()));
    let observed = stops.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                if let ControlMessage::StopRunner { command_id, .. } = message {
                    let mut stops = observed.lock().unwrap();
                    stops.push(command_id);
                    return Ok(if stops.len() == 1 {
                        Delivery::Offline
                    } else {
                        Delivery::Connected
                    });
                }
                Ok(Delivery::Connected)
            })),
    ));
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::retire
            .some_call(matching!(_, _, Some(47)))
            .returns(Ok(RegistrationRetirement::Removed { id: 47 })),
    ));
    let runtime = runtime(&fixture, registrations, gateway);
    runtime.tick().await.unwrap();
    let stopping = saved(&fixture);
    reconnect(&fixture);
    runtime.tick().await.unwrap();
    assert_eq!(stops.lock().unwrap().len(), 2);
    assert_eq!(
        *stops.lock().unwrap(),
        vec![CommandId(stopping.command_id.to_string()); 2]
    );
    assert_eq!(saved(&fixture).issued_at, stopping.issued_at);
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::CommandResult {
                command_id: CommandId(stopping.command_id.to_string()),
                outcome: CommandOutcome::Failed {
                    reason: CommandFailure::RunnerStop,
                },
            },
        )
        .await
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(stops.lock().unwrap().len(), 3);
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::CommandResult {
                command_id: CommandId(stopping.command_id.to_string()),
                outcome: CommandOutcome::Applied,
            },
        )
        .await
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(stops.lock().unwrap().len(), 3);
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    assert_eq!(saved(&fixture).slot_indices, vec![0]);
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::RunnerReady {
                runner_id: RunnerId(stopping.runner.id.to_string()),
            },
        )
        .await
        .unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
}

#[tokio::test]
async fn failed_send_replays_before_job_routing_error_and_running_report_closes_the_stop() {
    let mut fixture = Fixture::new();
    let stops = Arc::new(Mutex::new(Vec::new()));
    let observed = stops.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                if let ControlMessage::StopRunner { command_id, .. } = message {
                    let mut stops = observed.lock().unwrap();
                    stops.push(command_id);
                    if stops.len() == 1 {
                        return Err(ciproxy_gateway::error::Error::UnexpectedMessage);
                    }
                }
                Ok(Delivery::Connected)
            })),
    ));
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::retire
            .some_call(matching!(_, _, Some(47)))
            .returns(Ok(RegistrationRetirement::Removed { id: 47 })),
    ));
    let mut runtime = runtime(&fixture, registrations, gateway);
    runtime.tick().await.unwrap();
    let stopping = saved(&fixture);
    let job = job(&fixture);
    fixture
        .store
        .save_github_job(fixture.workspace, &job, at())
        .unwrap();
    runtime.assignments = Arc::new(Unimock::new(
        JobAssignmentsMock::facts
            .each_call(matching!(_, _, _))
            .answers(&|_, _, _, _| Err(ciproxy_github::error::Error::OrganizationInactive)),
    ));
    runtime.tick().await.unwrap();
    assert_eq!(
        *stops.lock().unwrap(),
        vec![CommandId(stopping.command_id.to_string()); 2]
    );
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    fixture.contact.report.runners[0].state = ciproxy_protocol::runners::RunnerState::Running {
        job_id: JobId(job.id.to_string()),
    };
    fixture.contact.at = at() + chrono::Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
    assert_eq!(stops.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn runner_ready_keeps_retirement_but_guest_job_started_preserves_the_job() {
    let fixture = Fixture::new();
    fixture
        .store
        .retire_own_runner(
            fixture.workspace,
            fixture.intent.runner.id,
            fixture.intent.command_id,
            at(),
            fixture.intent.runner.github_id,
        )
        .unwrap();
    let demand = job(&fixture);
    fixture
        .store
        .save_github_job(fixture.workspace, &demand, at())
        .unwrap();
    let runtime = runtime_with_clock(
        &fixture,
        Arc::new(Unimock::new(())),
        gateway(Arc::new(Mutex::new(Vec::new()))),
        clock(true),
    );
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
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopping);
    runtime
        .agent(
            fixture.workspace,
            fixture.machine,
            AgentMessage::JobStarted {
                runner_id: RunnerId(fixture.intent.runner.id.to_string()),
                job_id: JobId(demand.id.to_string()),
            },
        )
        .await
        .unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Busy);
}

#[tokio::test]
async fn recovery_send_error_does_not_block_another_saved_stop() {
    let fixture = Fixture::new();
    let mut second = fixture.intent.clone();
    second.runner.id = Uuid::new_v4();
    second.runner.github_id = Some(48);
    second.command_id = Uuid::new_v4();
    second.slot_indices = vec![1];
    second.runner.place = RunnerPlace::Machine {
        machine_id: fixture.machine,
        first_slot: 1,
    };
    fixture
        .store
        .create_own_runner(fixture.workspace, &second)
        .unwrap();
    for runner in [&fixture.intent, &second] {
        fixture
            .store
            .retire_own_runner(
                fixture.workspace,
                runner.runner.id,
                runner.command_id,
                at(),
                runner.runner.github_id,
            )
            .unwrap();
    }
    let sent = Arc::new(Mutex::new(Vec::new()));
    let observed = sent.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                if let ControlMessage::StopRunner { runner_id, .. } = message {
                    let mut sent = observed.lock().unwrap();
                    sent.push(runner_id.0);
                    if sent.len() == 1 {
                        return Err(ciproxy_gateway::error::Error::UnexpectedMessage);
                    }
                }
                Ok(Delivery::Connected)
            })),
    ));
    let runtime = runtime(&fixture, Arc::new(Unimock::new(())), gateway);
    runtime.tick().await.unwrap();
    assert_eq!(
        sent.lock()
            .unwrap()
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            fixture.intent.runner.id.to_string(),
            second.runner.id.to_string()
        ])
    );
    assert_eq!(
        fixture
            .store
            .own_runners(fixture.workspace)
            .unwrap()
            .iter()
            .map(|runner| runner.runner.state)
            .collect::<Vec<_>>(),
        vec![RunnerState::Stopping, RunnerState::Stopping]
    );
}
