//! Graceful shutdown keeps a raced job alive until the guest or unit exits.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use ciproxy_protocol::runners::{RunnerExit, RunnerState};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::GuestProvenance;
use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::cache_authority::CacheAuthority;
use crate::controller::Agent;
use crate::fixtures::github_decision_sent;
use crate::fixtures::{applied, controller, vm};
use crate::state::{MachineState, Mode};
use crate::terminal_completion::CompletionAction;

#[tokio::test]
async fn drain_waits_for_confirmed_stop_and_keeps_jobs_until_resume() {
    let mock = Unimock::new((
        github_decision_sent(),
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        VmsMock::request_stop
            .each_call(matching!((record) if record.report.runner_id.0 == "idle"))
            .answers(&|_, _| Ok(())),
        VmsMock::configuration_lost
            .each_call(matching!(_))
            .answers(&|_, _| Ok(false)),
    ));
    let agent = controller(
        MachineState {
            runners: vec![vm("idle", false), vm("job", true)],
            ..MachineState::default()
        },
        mock,
    );
    assert_eq!(
        agent
            .control(ControlMessage::Drain {
                command_id: CommandId("command-1".to_owned())
            })
            .await
            .unwrap(),
        applied()
    );
    let mut github_job = vm("job", true);
    github_job.cache_authority = CacheAuthority::GitHubOnly;
    assert_eq!(
        agent.state.lock().await.runners,
        vec![vm("idle", false), github_job.clone()]
    );
    assert_eq!(
        agent
            .control(ControlMessage::StopRunner {
                command_id: CommandId("command-1".to_owned()),
                organization_id: ciproxy_protocol::identifiers::OrganizationId("org-1".to_owned()),
                runner_id: RunnerId("idle".to_owned())
            })
            .await
            .unwrap(),
        applied()
    );
    let mut stopping = vm("idle", false);
    stopping.report.state = RunnerState::Stopping;
    assert_eq!(agent.state.lock().await.mode, Mode::Draining);
    assert_eq!(agent.state.lock().await.runners, vec![stopping, github_job]);
    assert_eq!(
        agent
            .control(ControlMessage::Resume {
                command_id: CommandId("command-1".to_owned())
            })
            .await
            .unwrap(),
        applied()
    );
    assert_eq!(agent.state.lock().await.mode, Mode::Active);
    agent.remove().await.unwrap();
    assert_eq!(agent.state.lock().await.mode, Mode::Removed);
    assert_eq!(
        agent
            .control(ControlMessage::Drain {
                command_id: CommandId("command-1".to_owned())
            })
            .await
            .unwrap(),
        applied()
    );
    assert_eq!(agent.state.lock().await.mode, Mode::Removed);
    assert!(
        !agent
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
    assert_eq!(
        agent
            .control(ControlMessage::Resume {
                command_id: CommandId("command-1".to_owned())
            })
            .await
            .unwrap(),
        Some(AgentMessage::CommandResult {
            command_id: CommandId("command-1".to_owned()),
            outcome: CommandOutcome::Failed {
                reason: CommandFailure::RunnerStart
            },
        })
    );
}

#[tokio::test]
async fn a_late_job_started_survives_pending_removal_until_runner_exit() {
    let mock = Unimock::new((
        VmsMock::networks.some_call(matching!()).returns(Ok(Vec::new())),
        VmsMock::remove_network.some_call(matching!(1)).returns(Ok(())),
        StoreMock::save.each_call(matching!(_)).answers(&|_, _| Ok(())),
        VmsMock::request_stop.some_call(matching!(_)).returns(Ok(())),
        VmsMock::configuration_lost.some_call(matching!(_)).returns(Ok(false)),
        VmsMock::stop.some_call(matching!((record) if record.report.state == RunnerState::Running { job_id: JobId("raced-job".to_owned()) })).returns(Ok(())),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
    ));
    let agent = controller(
        MachineState {
            runners: vec![vm("runner", false)],
            ..MachineState::default()
        },
        mock,
    );
    agent.remove().await.unwrap();
    assert_eq!(
        agent.state.lock().await.runners[0].report.state,
        RunnerState::Stopping
    );
    assert!(
        !agent
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
    assert_eq!(
        agent
            .guest(
                GuestProvenance::InitialConfiguration,
                "runner",
                GuestToAgent::JobStarted {
                    runner_id: RunnerId("runner".to_owned()),
                    job_id: JobId("raced-job".to_owned())
                }
            )
            .await
            .unwrap(),
        Some(AgentMessage::JobStarted {
            runner_id: RunnerId("runner".to_owned()),
            job_id: JobId("raced-job".to_owned()),
        })
    );
    assert_eq!(agent.state.lock().await.mode, Mode::Removed);
    assert_eq!(
        agent.state.lock().await.runners[0].report.state,
        RunnerState::Running {
            job_id: JobId("raced-job".to_owned())
        }
    );
    assert!(
        !agent
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
    agent
        .guest(
            GuestProvenance::InitialConfiguration,
            "runner",
            GuestToAgent::RunnerExited {
                runner_id: RunnerId("runner".to_owned()),
                exit: RunnerExit { code: Some(0) },
            },
        )
        .await
        .unwrap();
    assert_eq!(agent.state.lock().await.pending_networks, [1].into());
    assert!(
        agent
            .complete_removal(CompletionAction::Stop)
            .await
            .unwrap()
    );
}
