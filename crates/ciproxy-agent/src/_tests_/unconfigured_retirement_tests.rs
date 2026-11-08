//! Lost registrations retire only after an authorized stop and return capacity.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::{GuestToAgent, HANDOFF_PROTOCOL_VERSION};
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, GuestProvenance, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{applied, controller, report_dependencies, spec, start_image, vm};
use crate::state::MachineState;

fn stop(runner: &str) -> ControlMessage {
    ControlMessage::StopRunner {
        command_id: CommandId("command-1".into()),
        organization_id: OrganizationId("org-1".into()),
        runner_id: RunnerId(runner.into()),
    }
}

fn hello(received: bool) -> GuestToAgent {
    GuestToAgent::Hello {
        protocol_version: ProtocolVersion::CURRENT,
        handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
        image_id: vm("runner-1", false).report.image_id,
        configuration_received: received,
    }
}

#[tokio::test]
async fn unresponsive_unconfigured_guest_releases_capacity_for_a_replacement() {
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Starting;
    let mut replacement = vm("replacement", false);
    replacement.network_index = 2;
    replacement.report.state = RunnerState::Starting;
    let allocated = replacement.clone();
    let mock = Unimock::new((
        report_dependencies(),
        start_image(),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::configuration_lost
            .some_call(matching!(_))
            .returns(Ok(true)),
        VmsMock::request_stop
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::stop
            .some_call(matching!((vm) if vm.report.runner_id.0 == "runner-1"))
            .returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        VmsMock::allocate
            .some_call(matching!((report, used) if report.slot_indices == [0] && *used == [1]))
            .returns(Ok(allocated)),
        CacheMock::budget
            .some_call(matching!(564_100_387_328))
            .returns(Ok(())),
        VmsMock::start
            .some_call(matching!((vm, _) if vm.report.runner_id.0 == "replacement"))
            .returns(Ok(())),
    ));
    let agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        mock,
    );
    agent.configuration_lost("runner-1").await.unwrap();
    agent
        .guest(GuestProvenance::Recovery, "runner-1", hello(false))
        .await
        .unwrap();
    assert_eq!(
        agent.report().await.unwrap().runners[0].state,
        RunnerState::Starting
    );

    assert_eq!(agent.control(stop("runner-1")).await.unwrap(), applied());

    let report = agent.report().await.unwrap();
    assert_eq!(report.runners, vec![]);
    assert_eq!(report.slots[0].occupancy, SlotOccupancy::Free);
    assert_eq!(agent.state.lock().await.pending_networks, [1].into());
    let mut runner = spec();
    runner.runner_id = RunnerId("replacement".into());
    assert_eq!(
        agent
            .control(ControlMessage::StartRunner {
                command_id: CommandId("command-1".into()),
                runner,
            })
            .await
            .unwrap(),
        applied()
    );
    assert_eq!(agent.state.lock().await.runners, vec![replacement]);
    assert_eq!(agent.control(stop("replacement")).await.unwrap(), applied());
    assert_eq!(
        agent.report().await.unwrap().runners[0].state,
        RunnerState::Stopping
    );
}

#[tokio::test]
async fn unconfigured_stop_failure_remains_owned_and_reconciliation_retries() {
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Starting;
    record.report.configuration_lost = true;
    let mock = Unimock::new((
        report_dependencies(),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::stop.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Err(Error::GuestMismatch))
                .once()
                .then()
                .answers(&|_, _| Ok(()));
        }),
        VmsMock::restore.some_call(matching!(_)).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        CacheMock::refresh_budget
            .some_call(matching!(_))
            .returns(Ok(())),
        VmsMock::networks
            .some_call(matching!())
            .returns(Ok(vec![1])),
        VmsMock::remove_network
            .some_call(matching!(1))
            .returns(Ok(())),
    ));
    let agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        mock,
    );
    let response = agent.control(stop("runner-1")).await.unwrap().unwrap();
    assert!(matches!(
        response,
        AgentMessage::CommandResult {
            outcome: CommandOutcome::Failed {
                reason: CommandFailure::RunnerStop
            },
            ..
        }
    ));
    assert_eq!(
        agent.report().await.unwrap().runners[0].state,
        RunnerState::Stopping
    );

    agent.reconcile().await.unwrap();

    assert_eq!(agent.report().await.unwrap().runners, vec![]);
    assert_eq!(agent.state.lock().await.pending_networks, [].into());
}

#[tokio::test]
async fn a_confirmed_registration_keeps_guest_shutdown_after_a_loss_report() {
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Stopping;
    record.report.configuration_lost = true;
    let mock = Unimock::new((
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::request_stop
            .some_call(matching!(_))
            .returns(Ok(())),
    ));
    let agent = controller(
        MachineState {
            runners: vec![record.clone()],
            ..MachineState::default()
        },
        mock,
    );

    agent
        .guest(GuestProvenance::Recovery, "runner-1", hello(true))
        .await
        .unwrap();

    record.report.configuration_lost = false;
    assert_eq!(agent.state.lock().await.runners, vec![record]);
}
