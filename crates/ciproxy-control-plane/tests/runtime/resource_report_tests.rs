//! Authenticated reports retain original free requests and exact root bytes.

use std::sync::Arc;

use ciproxy_control_plane::runtime::{MachineRuntime, OwnMachines};
use ciproxy_gateway::gateway::MachineGatewayMock;
use ciproxy_gateway::types::Delivery;
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use super::clock::clock;
use super::support::{Fixture, at};

#[tokio::test]
async fn a_restored_runner_report_corrects_request_and_exact_allocated_bytes() {
    let mut fixture = Fixture::new();
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
    fixture.contact.report.runners[0].requested_size.disk_bytes = 40_000_000_000;
    fixture.contact.report.runners[0].size.disk_bytes = 154_000_004_096;
    fixture
        .store
        .record_contact(fixture.workspace, fixture.machine, &fixture.contact)
        .unwrap();
    let clock = clock(true);
    let runtime = MachineRuntime::new(
        fixture.store.clone(),
        Arc::new(Unimock::new(
            MachineGatewayMock::send
                .some_call(
                    matching!((_,_,message) if matches!(message,ControlMessage::DockerBudget {..})),
                )
                .returns(Ok(Delivery::Connected)),
        )),
        Arc::new(Unimock::new(())),
        Arc::new(AssignmentService {
            store: Arc::new(StoreAdapter::new(fixture.store.clone(), clock.clone())),
        }),
        clock,
        None,
        Arc::new(Unimock::new(())),
    );
    runtime.tick().await.unwrap();
    let runner = fixture
        .store
        .own_runners(fixture.workspace)
        .unwrap()
        .remove(0);
    assert_eq!(runner.requested_size.disk_gb, 40);
    assert_eq!(runner.runner.size.disk_bytes, 154_000_004_096);
    assert_eq!(runner.runner.state, RunnerState::Busy);
    assert_eq!(runner.confirmed_at, Some(at()));
}
