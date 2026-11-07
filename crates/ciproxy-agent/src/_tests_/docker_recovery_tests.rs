//! Failed drive closure leaves durable recovery pending across agent restart.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::{GuestToAgent, JobPreparation};
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::runners::RunnerExit;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, GuestProvenance, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::docker::DockerDisksMock;
use crate::error::Error;
use crate::fixtures::{controller, trusted_facts, vm};
use crate::state::MachineState;

#[tokio::test]
async fn failed_drive_close_retries_pending_discard_after_restart() {
    let pending = Arc::new(Mutex::new(false));
    let marked = pending.clone();
    let mock = Unimock::new((
        DockerDisksMock::reserved
            .each_call(matching!(_))
            .answers(&|_, _| Ok(true)),
        DockerDisksMock::failed
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                *marked.lock().unwrap() = true;
                Ok(true)
            })),
        VmsMock::clear_docker
            .some_call(matching!(_))
            .returns(Err(Error::GuestMismatch)),
    ))
    .no_verify_in_drop();
    let mut record = vm("job", true);
    record.job_facts = Some(trusted_facts());
    let state = MachineState {
        runners: vec![record],
        ..MachineState::default()
    };
    let mut agent = controller(state.clone(), mock.clone());
    agent.dockers = Some(Arc::new(mock));

    let result = agent
        .guest(
            GuestProvenance::InitialConfiguration,
            "job",
            GuestToAgent::JobPrepared {
                runner_id: RunnerId("job".into()),
                job_id: JobId("job-1".into()),
                outcome: JobPreparation {
                    cache_ready: true,
                    docker_attached: false,
                },
            },
        )
        .await;
    assert!(matches!(result, Err(Error::GuestMismatch)));
    assert!(
        *pending.lock().unwrap(),
        "A failed close must retain a recovery marker"
    );

    let events = Arc::new(Mutex::new(Vec::new()));
    let closed = events.clone();
    let discarded = events.clone();
    let marker = pending.clone();
    let mock = Unimock::new((
        VmsMock::restore.some_call(matching!(_)).returns(Ok(())),
        DockerDisksMock::pending
            .some_call(matching!(_))
            .returns(Ok(true)),
        VmsMock::clear_docker
            .next_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                closed.lock().unwrap().push("closed");
                Ok(())
            })),
        DockerDisksMock::discard
            .next_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                discarded.lock().unwrap().push("discarded");
                *marker.lock().unwrap() = false;
                Ok(())
            })),
    ));
    let mut restored = controller(state, mock.clone());
    restored.dockers = Some(Arc::new(mock));
    restored.restore().await.unwrap();
    assert_eq!(*events.lock().unwrap(), vec!["closed", "discarded"]);
    assert!(!*pending.lock().unwrap());
}

#[tokio::test]
async fn job_exit_discards_pending_failure_before_releasing_its_reservation() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let stopped = events.clone();
    let discarded = events.clone();
    let released = events.clone();
    let mock = Unimock::new((
        VmsMock::stop
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                stopped.lock().unwrap().push("stopped");
                Ok(())
            })),
        DockerDisksMock::pending
            .some_call(matching!(_))
            .returns(Ok(true)),
        DockerDisksMock::discard
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                discarded.lock().unwrap().push("discarded");
                Ok(())
            })),
        DockerDisksMock::release
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                released.lock().unwrap().push("released");
                Ok(())
            })),
        CacheMock::release.some_call(matching!(_)).returns(Ok(())),
        StoreMock::save.some_call(matching!(_)).returns(Ok(())),
    ))
    .no_verify_in_drop();
    let mut agent = controller(
        MachineState {
            runners: vec![vm("job", true)],
            ..MachineState::default()
        },
        mock.clone(),
    );
    agent.dockers = Some(Arc::new(mock));

    agent
        .guest(
            GuestProvenance::Recovery,
            "job",
            GuestToAgent::RunnerExited {
                runner_id: RunnerId("job".into()),
                exit: RunnerExit { code: Some(0) },
            },
        )
        .await
        .unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec!["stopped", "discarded", "released"]
    );
    assert!(agent.state.lock().await.runners.is_empty());
}
