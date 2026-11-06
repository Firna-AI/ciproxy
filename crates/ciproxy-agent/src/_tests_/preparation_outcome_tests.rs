//! Failed setup suppresses only its cache assignment and closes dormant storage.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::guest::{GuestToAgent, JobPreparation};
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::docker::DockerDisksMock;
use crate::fixtures::{applied, controller, trusted_facts, vm};
use crate::state::{MachineState, PreparationState};

async fn replay_restored(state: &MachineState, outcome: JobPreparation) {
    let restored: MachineState =
        serde_json::from_slice(&serde_json::to_vec(state).unwrap()).unwrap();
    assert_eq!(
        restored.runners[0].preparation,
        PreparationState::Completed(outcome)
    );
    let mock = Unimock::new((
        VmsMock::restore.some_call(matching!(_)).returns(Ok(())),
        CacheMock::prepare
            .some_call(matching!(_, _))
            .returns(Ok(None)),
        VmsMock::send
            .some_call(matching!(_, _))
            .answers(&|_, _, message| {
                assert_eq!(
                    message,
                    &AgentToGuest::PrepareJob {
                        job_id: JobId("job-1".into()),
                        facts: trusted_facts(),
                        cache: None,
                        docker: None
                    }
                );
                Ok(())
            }),
    ));
    let mut agent = controller(restored, mock.clone());
    agent.dockers = Some(Arc::new(mock));
    agent.restore().await.unwrap();
    assert_eq!(
        agent
            .control(ControlMessage::JobFacts {
                command_id: CommandId("command-1".into()),
                runner_id: RunnerId("job".into()),
                job_id: JobId("job-1".into()),
                facts: trusted_facts()
            })
            .await
            .unwrap(),
        applied()
    );
}

#[tokio::test]
async fn failed_preparation_suppresses_counts_and_releases_only_after_drive_close_ack() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let fallback = events.clone();
    let closed = events.clone();
    let released = events.clone();
    let mock = Unimock::new((
        StoreMock::save.some_call(matching!(_)).returns(Ok(())),
        CacheMock::fallback
            .next_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                fallback.lock().unwrap().push("fallback");
                Ok(())
            })),
        DockerDisksMock::reserved
            .some_call(matching!(_))
            .returns(Ok(true)),
        VmsMock::clear_docker
            .next_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                closed.lock().unwrap().push("closed");
                Ok(())
            })),
        DockerDisksMock::release
            .next_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                released.lock().unwrap().push("released");
                Ok(())
            })),
    ));
    let mut record = vm("job", true);
    record.job_facts = Some(trusted_facts());
    let mut agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        mock.clone(),
    );
    agent.dockers = Some(Arc::new(mock));
    assert_eq!(
        agent
            .guest(
                "job",
                GuestToAgent::JobPrepared {
                    runner_id: RunnerId("job".into()),
                    job_id: JobId("job-1".into()),
                    outcome: JobPreparation::default()
                }
            )
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        *events.lock().unwrap(),
        vec!["fallback", "closed", "released"]
    );
    let saved = agent.state.lock().await.clone();
    replay_restored(&saved, JobPreparation::default()).await;
}

#[tokio::test]
async fn a_mounted_docker_disk_keeps_its_reservation_when_only_cache_preparation_fails() {
    let mock = Unimock::new((
        CacheMock::fallback.some_call(matching!(_)).returns(Ok(())),
        StoreMock::save.some_call(matching!(_)).returns(Ok(())),
    ));
    let mut record = vm("job", true);
    record.job_facts = Some(trusted_facts());
    let mut agent = controller(
        MachineState {
            runners: vec![record],
            ..MachineState::default()
        },
        mock.clone(),
    );
    agent.dockers = Some(Arc::new(mock));
    assert_eq!(
        agent
            .guest(
                "job",
                GuestToAgent::JobPrepared {
                    runner_id: RunnerId("job".into()),
                    job_id: JobId("job-1".into()),
                    outcome: JobPreparation {
                        cache_ready: false,
                        docker_attached: true
                    }
                }
            )
            .await
            .unwrap(),
        None
    );
    assert_eq!(agent.state.lock().await.runners.len(), 1);
    let saved = agent.state.lock().await.clone();
    replay_restored(
        &saved,
        JobPreparation {
            cache_ready: false,
            docker_attached: true,
        },
    )
    .await;
}
