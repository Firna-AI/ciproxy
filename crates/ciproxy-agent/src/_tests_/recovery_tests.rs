//! Start retries re-deliver only matching startup registrations and keep VM files.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::identifiers::{CommandId, RunnerId};
use ciproxy_protocol::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{StoreMock, VmsMock};
use crate::controller::Agent;
use crate::fixtures::{applied, controller, report_dependencies, spec, vm};
use crate::state::MachineState;

#[tokio::test]
async fn a_matching_starting_retry_delivers_registration_without_recreating_the_vm() {
    let mut record = vm("runner-1", false);
    record.report.state = RunnerState::Starting;
    record.report.configuration_lost = true;
    let mut initial = MachineState {
        runners: vec![record],
        ..MachineState::default()
    };
    let mock = Unimock::new((report_dependencies(),
        StoreMock::save.some_call(matching!((state) if !state.runners[0].report.configuration_lost)).answers(&|_, _| Ok(())),
        VmsMock::send.some_call(matching!((_, message) if matches!(message, AgentToGuest::ConfigureRunner { runner_id, registration, .. }
            if runner_id.0 == "runner-1" && registration.expose_secret() == "one-use-registration"))).returns(Ok(())),
    ));
    let agent = controller(initial.clone(), mock);
    assert_eq!(
        agent
            .control(ControlMessage::StartRunner {
                command_id: CommandId("command-1".to_owned()),
                runner: spec()
            })
            .await
            .unwrap(),
        applied()
    );
    initial.runners[0].report.configuration_lost = false;
    assert_eq!(*agent.state.lock().await, initial);
}

#[tokio::test]
async fn a_starting_retry_cannot_change_organization_label_or_size() {
    for field in ["organization", "label", "size"] {
        let mut record = vm("runner-1", false);
        record.report.state = RunnerState::Starting;
        let initial = MachineState {
            runners: vec![record],
            ..MachineState::default()
        };
        let agent = controller(initial.clone(), Unimock::new(report_dependencies()));
        let mut runner = spec();
        match field {
            "organization" => runner.organization_id.0 = "other".to_owned(),
            "label" => runner.label.0 = "other".to_owned(),
            _ => runner.size.vcpu = 4,
        }
        assert_eq!(
            agent
                .control(ControlMessage::StartRunner {
                    command_id: CommandId("command-1".to_owned()),
                    runner
                })
                .await
                .unwrap(),
            Some(AgentMessage::CommandResult {
                command_id: CommandId("command-1".to_owned()),
                outcome: CommandOutcome::Failed {
                    reason: CommandFailure::RunnerStart
                },
            }),
            "{field}"
        );
        assert_eq!(*agent.state.lock().await, initial);
    }
}

#[tokio::test]
async fn runner_ready_persists_and_forwards_the_warm_transition() {
    let mut record = vm("runner", false);
    record.report.state = RunnerState::Starting;
    let mock = Unimock::new(
        StoreMock::save
            .some_call(matching!((state) if state.runners[0].report.state == RunnerState::Warm))
            .returns(Ok(())),
    );
    let agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        mock,
    );
    assert_eq!(
        agent
            .guest(
                "runner",
                GuestToAgent::RunnerReady {
                    runner_id: RunnerId("runner".to_owned())
                }
            )
            .await
            .unwrap(),
        Some(AgentMessage::RunnerReady {
            runner_id: RunnerId("runner".to_owned())
        })
    );
    assert_eq!(
        agent.state.lock().await.runners[0].report.state,
        RunnerState::Warm
    );
}
