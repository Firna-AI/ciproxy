//! A saved stop can consume late loss proof and keep it through another restart.

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::guest::{GuestToAgent, HANDOFF_PROTOCOL_VERSION};
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_protocol::reports::SlotOccupancy;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, GuestProvenance, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{applied, controller, report_dependencies, vm};
use crate::state::MachineState;

#[tokio::test]
async fn saved_stop_uses_late_hello_loss_proof_and_preserves_it_across_restart() {
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Stopping;
    let image_id = record.report.image_id.clone();
    let saved = serde_json::to_vec(&MachineState {
        runners: vec![record],
        ..MachineState::default()
    })
    .unwrap();
    let mock = Unimock::new((
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::configuration_lost.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Ok(false))
                .once()
                .then()
                .answers(&|_, _| Ok(true));
        }),
        VmsMock::request_stop
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::stop
            .some_call(matching!(_))
            .returns(Err(Error::GuestMismatch)),
    ));
    let agent = controller(serde_json::from_slice(&saved).unwrap(), mock);
    assert_eq!(
        agent
            .control(ControlMessage::StopRunner {
                command_id: CommandId("command-1".into()),
                organization_id: OrganizationId("org-1".into()),
                runner_id: RunnerId("runner-1".into()),
            })
            .await
            .unwrap(),
        applied()
    );
    assert!(
        !agent.state.lock().await.runners[0]
            .report
            .configuration_lost
    );

    let result = agent
        .guest(
            GuestProvenance::Recovery,
            "runner-1",
            GuestToAgent::Hello {
                protocol_version: ProtocolVersion::CURRENT,
                handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
                image_id,
                configuration_received: false,
            },
        )
        .await;

    assert!(
        matches!(result, Err(Error::GuestMismatch)),
        "late proof must attempt host cleanup: {result:?}"
    );
    let saved = agent.state.lock().await.clone();
    assert!(saved.runners[0].report.configuration_lost);
    assert_eq!(saved.runners[0].report.state, RunnerState::Stopping);
    let durable = serde_json::to_vec(&saved).unwrap();
    let restored = controller(
        serde_json::from_slice(&durable).unwrap(),
        Unimock::new((
            report_dependencies(),
            VmsMock::restore.some_call(matching!(_)).returns(Ok(())),
            VmsMock::stop.some_call(matching!(_)).returns(Ok(())),
            CacheMock::release.some_call(matching!(_)).returns(Ok(())),
            StoreMock::save.some_call(matching!(_)).returns(Ok(())),
        )),
    );

    restored.restore().await.unwrap();

    let report = restored.report().await.unwrap();
    assert_eq!(report.runners, vec![]);
    assert_eq!(report.slots[0].occupancy, SlotOccupancy::Free);
}
