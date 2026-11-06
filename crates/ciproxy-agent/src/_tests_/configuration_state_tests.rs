//! Confirmed startup loss survives reports and clears for a fresh pending delivery.

use ciproxy_protocol::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{StoreMock, VmsMock};
use crate::controller::Agent;
use crate::fixtures::{controller, report_dependencies, vm};
use crate::state::MachineState;

#[tokio::test]
async fn configuration_loss_is_durable_and_queued_observations_use_current_bridge_state() {
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Starting;
    let mock = Unimock::new((
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::configuration_lost.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Ok(true))
                .once()
                .then()
                .answers(&|_, _| Ok(false));
        }),
    ));
    let agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        mock,
    );
    agent.configuration_lost("runner-1").await.unwrap();
    let saved = agent.state.lock().await.clone();
    assert!(saved.runners[0].report.configuration_lost);
    assert!(
        !serde_json::to_string(&saved)
            .unwrap()
            .contains("registration")
    );
    let durable = serde_json::to_vec(&saved).unwrap();
    let saved: MachineState = serde_json::from_slice(&durable).unwrap();
    let restored = controller(saved, Unimock::new(report_dependencies()));
    assert!(restored.report().await.unwrap().runners[0].configuration_lost);
    agent.configuration_lost("runner-1").await.unwrap();
    assert!(
        !agent.state.lock().await.runners[0]
            .report
            .configuration_lost
    );
}
