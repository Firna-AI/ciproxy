//! Machine reports and durable control transitions use only deterministic fakes.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::{CommandId, JobId, OrganizationId, RunnerId};
use ciproxy_protocol::reports::{DrainState, SlotOccupancy};
use ciproxy_protocol::runners::{RunnerExit, RunnerState};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::GuestProvenance;
use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::github_decision_sent;
use crate::fixtures::{applied, controller, report_dependencies, spec, start_image, vm};
use crate::state::{MachineState, Mode};
use crate::terminal_completion::CompletionAction;

#[tokio::test]
async fn starts_a_fresh_vm_and_keeps_registration_out_of_durable_state() {
    let mock = Unimock::new((report_dependencies(),
        start_image(),
        VmsMock::allocate.some_call(matching!((report, used) if report.state == RunnerState::Starting && used.is_empty()))
            .returns(Ok(vm("runner-1", false))),
        StoreMock::save.some_call(matching!((state) if state.runners.len() == 1))
            .answers(&|_, state| {
                assert!(!serde_json::to_string(state).unwrap().contains("one-use-registration")); Ok(())
            }),
        CacheMock::budget.some_call(matching!(564_100_387_328)).returns(Ok(())),
        VmsMock::start.some_call(matching!((record, config) if record.report.runner_id.0 == "runner-1" && matches!(config,
            ciproxy_protocol::guest::AgentToGuest::ConfigureRunner { registration, .. } if registration.expose_secret() == "one-use-registration")))
            .returns(Ok(())),
    ));
    let agent = controller(MachineState::default(), mock);
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
    assert_eq!(agent.state.lock().await.runners.len(), 1);
}

#[tokio::test]
async fn failed_state_save_leaves_no_phantom_runner() {
    let mock = Unimock::new((
        report_dependencies(),
        start_image(),
        VmsMock::allocate
            .each_call(matching!(_, _))
            .answers(&|_, _, _| Ok(vm("runner-1", false))),
        StoreMock::save.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Err(Error::Capacity))
                .once()
                .then()
                .answers(&|_, _| Ok(()));
        }),
        CacheMock::budget
            .some_call(matching!(564_100_387_328))
            .returns(Ok(())),
        VmsMock::start.some_call(matching!(_, _)).returns(Ok(())),
    ));
    let agent = controller(MachineState::default(), mock);
    let command = ControlMessage::StartRunner {
        command_id: CommandId("command-1".to_owned()),
        runner: spec(),
    };
    assert_eq!(
        agent.control(command.clone()).await.unwrap(),
        Some(AgentMessage::CommandResult {
            command_id: CommandId("command-1".to_owned()),
            outcome: CommandOutcome::Failed {
                reason: CommandFailure::RunnerStart
            },
        })
    );
    assert_eq!(agent.state.lock().await.runners.len(), 0);
    assert_eq!(agent.control(command).await.unwrap(), applied());
    assert_eq!(agent.state.lock().await.runners.len(), 1);
}

#[tokio::test]
async fn stop_runner_cannot_stop_a_job_or_another_organization() {
    let agent = controller(
        MachineState {
            runners: vec![vm("job", true), vm("idle", false)],
            ..MachineState::default()
        },
        Unimock::new((
            github_decision_sent(),
            StoreMock::save.some_call(matching!(_)).returns(Ok(())),
        )),
    );
    for (runner, organization) in [("job", "org-1"), ("idle", "org-2")] {
        assert_eq!(
            agent
                .control(ControlMessage::StopRunner {
                    command_id: CommandId("command-1".to_owned()),
                    organization_id: OrganizationId(organization.to_owned()),
                    runner_id: RunnerId(runner.to_owned())
                })
                .await
                .unwrap(),
            Some(AgentMessage::CommandResult {
                command_id: CommandId("command-1".to_owned()),
                outcome: CommandOutcome::Failed {
                    reason: CommandFailure::RunnerStop
                }
            })
        );
    }
    assert_eq!(agent.state.lock().await.runners.len(), 2);
}

#[tokio::test]
async fn reports_current_hardware_and_each_job_or_warm_slot() {
    let mut job = vm("job", true);
    job.report.size.vcpu = 4;
    job.report.slot_indices = vec![0, 1];
    let mut idle = vm("idle", false);
    idle.report.slot_indices = vec![2];
    let agent = controller(
        MachineState {
            runners: vec![job, idle],
            mode: Mode::Draining,
            ..MachineState::default()
        },
        Unimock::new(report_dependencies()),
    );
    let report = agent.report().await.unwrap();
    assert_eq!(report.capacity.0, 6);
    assert_eq!(report.hostname, "fixture-machine");
    assert_eq!(report.hardware.threads, 16);
    assert_eq!(report.drain, DrainState::Draining);
    assert_eq!(
        report
            .slots
            .iter()
            .map(|slot| slot.occupancy.clone())
            .collect::<Vec<_>>(),
        vec![
            SlotOccupancy::Job {
                runner_id: RunnerId("job".to_owned()),
                job_id: JobId("job-1".to_owned())
            },
            SlotOccupancy::Job {
                runner_id: RunnerId("job".to_owned()),
                job_id: JobId("job-1".to_owned())
            },
            SlotOccupancy::Warm {
                runner_id: RunnerId("idle".to_owned())
            },
            SlotOccupancy::Free,
            SlotOccupancy::Free,
            SlotOccupancy::Free,
        ]
    );
}

#[tokio::test]
async fn guest_exit_discards_vm_and_removal_finishes_only_after_the_job() {
    let mock = Unimock::new((
        VmsMock::networks
            .some_call(matching!())
            .returns(Ok(Vec::new())),
        VmsMock::remove_network
            .some_call(matching!(1))
            .returns(Ok(())),
        VmsMock::stop
            .some_call(matching!((record) if record.report.runner_id.0 == "job"))
            .returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        StoreMock::save
            .each_call(
                matching!((state) if state.runners.is_empty() && state.mode == Mode::Removed),
            )
            .answers(&|_, _| Ok(())),
    ));
    let agent = controller(
        MachineState {
            runners: vec![vm("job", true)],
            mode: Mode::Removed,
            ..MachineState::default()
        },
        mock,
    );
    assert_eq!(
        agent
            .guest(
                GuestProvenance::InitialConfiguration,
                "job",
                GuestToAgent::RunnerExited {
                    runner_id: RunnerId("job".to_owned()),
                    exit: RunnerExit { code: Some(0) }
                }
            )
            .await
            .unwrap(),
        Some(AgentMessage::RunnerExited {
            runner_id: RunnerId("job".to_owned()),
            exit: RunnerExit { code: Some(0) }
        })
    );
    assert_eq!(agent.state.lock().await.pending_networks, [1].into());
    assert!(
        agent
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn restore_reconnects_all_owned_vms_without_starting_new_vms() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let restore_calls = observed.clone();
    let mock = Unimock::new(
        VmsMock::restore
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, vm| {
                restore_calls
                    .lock()
                    .unwrap()
                    .push(vm.report.runner_id.0.clone());
                Ok(())
            })),
    );
    let initial = MachineState {
        runners: vec![vm("job", true), vm("idle", false)],
        ..MachineState::default()
    };
    let agent = controller(initial.clone(), mock);
    agent.restore().await.unwrap();
    assert_eq!(*agent.state.lock().await, initial);
    assert_eq!(*observed.lock().unwrap(), vec!["job", "idle"]);
}
