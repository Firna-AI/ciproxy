//! GitHub removal and durable capacity release.

use super::clock::clock;
use super::support::{Fixture, at};
use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::{Removal, RunnerRegistrationsMock};
use ciproxy_protocol::agent::{AgentMessage, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_store_interface::runners::RunnerState;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use unimock::{MockFn, Unimock, matching};

#[tokio::test]
async fn github_busy_race_preserves_the_vm_across_stale_warm_reports() {
    let fixture = Fixture::new();
    let clock = clock(true);
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(
                matching!((_,_,message) if matches!(message,ControlMessage::DockerBudget {..})),
            )
            .answers(&|_, _, _, _| Ok(Delivery::Connected)),
    ));
    let removals = Arc::new(AtomicUsize::new(0));
    let counted = removals.clone();
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::remove
            .each_call(matching!(_, 47))
            .answers_arc(Arc::new(move |_, _, _| {
                counted.fetch_add(1, Ordering::SeqCst);
                Ok(Removal::Busy)
            })),
    ));
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        registrations,
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
    );
    runtime.tick().await.unwrap();
    assert_eq!(removals.load(Ordering::SeqCst), 1);
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Busy
    );
    runtime.tick().await.unwrap();
    assert_eq!(removals.load(Ordering::SeqCst), 1);
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Busy
    );
}

#[tokio::test]
async fn github_removal_precedes_stop_and_capacity_waits_for_an_absent_vm_report() {
    let mut fixture = Fixture::new();
    let clock = clock(true);
    let events = Arc::new(Mutex::new(Vec::new()));
    let removed = events.clone();
    let stopped = events.clone();
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::remove
            .some_call(matching!(_, 47))
            .answers_arc(Arc::new(move |_, _, _| {
                removed.lock().unwrap().push("github_removed");
                Ok(Removal::Removed)
            })),
    ));
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                if matches!(message, ControlMessage::DockerBudget { .. }) {
                    return Ok(Delivery::Connected);
                }
                assert!(matches!(message, ControlMessage::StopRunner { .. }));
                stopped.lock().unwrap().push("stop_sent");
                Ok(Delivery::Connected)
            }))
            .at_least_times(1),
    ));
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        registrations,
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
    );
    runtime.tick().await.unwrap();
    let stopping = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .remove(0);
    assert_eq!(stopping.runner.state, RunnerState::Stopping);
    assert_eq!(*events.lock().unwrap(), vec!["github_removed", "stop_sent"]);
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
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Stopping
    );
    fixture.contact.report.runners.clear();
    fixture.contact.at = at() + chrono::Duration::seconds(1);
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    runtime.tick().await.unwrap();
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Stopped
    );
}

#[tokio::test]
async fn removed_offline_machine_still_loses_its_idle_github_registration() {
    let fixture = Fixture::new();
    fixture
        .store
        .remove_machine(fixture.workspace, fixture.machine, at())
        .unwrap();
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::remove
            .some_call(matching!(_, 47))
            .returns(Ok(Removal::Removed)),
    ));
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .some_call(matching!(_, _, _))
            .returns(Err(ciproxy_gateway::error::Error::NotFound)),
    ));
    let clock = clock(true);
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        registrations,
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
    );
    runtime.tick().await.unwrap();
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Stopping
    );
}

#[tokio::test]
async fn revoked_app_access_stops_only_its_idle_runner_on_a_shared_machine() {
    let fixture = Fixture::new();
    let clock = clock(true);
    let events = Arc::new(Mutex::new(Vec::new()));
    let observed = events.clone();
    let gateway = Arc::new(Unimock::new(
        MachineGatewayMock::send
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, _, message| {
                if matches!(message, ControlMessage::DockerBudget { .. }) {
                    return Ok(Delivery::Connected);
                }
                assert!(matches!(message, ControlMessage::StopRunner { .. }));
                observed.lock().unwrap().push(message);
                Ok(Delivery::Connected)
            })),
    ));
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::remove
            .some_call(matching!(_, 47))
            .returns(Ok(Removal::AccessGone)),
    ));
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        gateway,
        registrations,
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
    );
    runtime.tick().await.unwrap();
    assert!(
        matches!(&events.lock().unwrap()[0],ciproxy_protocol::agent::ControlMessage::StopRunner{organization_id,runner_id,..} if organization_id.0=="42"&&runner_id.0==fixture.intent.runner.id.to_string())
    );
    assert_eq!(
        fixture.store.own_runners(fixture.workspace).unwrap()[0]
            .runner
            .state,
        RunnerState::Stopping
    );
}
