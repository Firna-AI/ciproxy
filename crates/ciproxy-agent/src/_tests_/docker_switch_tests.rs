//! VM drive failure keeps the job's root fallback available during cleanup.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::{ControlMessage, JobPreparationOutcome};
use ciproxy_protocol::cache::{CacheConnection, DockerDiskAccess, DockerDiskAttachment};
use ciproxy_protocol::guest::{AgentToGuest, JobDecision};
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, HostMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::docker::DockerDisksMock;
use crate::error::Error;
use crate::fixtures::{controller, hardware, trusted_facts, vm};
use crate::state::MachineState;

#[tokio::test]
async fn failed_drive_switch_releases_job_on_root_while_drive_cleanup_waits() {
    let sent = Arc::new(Mutex::new(Vec::new()));
    let captured = sent.clone();
    let cache = CacheConnection {
        url: Secret::new("http://machine:7887"),
        token: Secret::new("session"),
    };
    let connection = cache.clone();
    let mock = Unimock::new((
        StoreMock::save
            .each_call(matching!(_))
            .answers(&|_, _| Ok(())),
        HostMock::hardware
            .some_call(matching!())
            .returns(Ok(hardware())),
        CacheMock::prepare
            .some_call(matching!(_, _))
            .returns(Ok(Some((
                connection,
                JobScope::Unknown {
                    reason: UnknownJobScopeReason::MissingToken,
                },
                4096,
            )))),
        DockerDisksMock::prepare
            .some_call(matching!(_, _, _, _, _, _))
            .returns(Ok(Some(DockerDiskAttachment {
                free_bytes: 4096,
                drive_id: "docker".into(),
                access: DockerDiskAccess::Persistent,
            }))),
        VmsMock::swap_docker
            .some_call(matching!(_))
            .returns(Err(Error::GuestMismatch)),
        DockerDisksMock::failed
            .some_call(matching!(_))
            .returns(Ok(true)),
        DockerDisksMock::pending
            .some_call(matching!(_))
            .returns(Ok(true)),
        VmsMock::clear_docker
            .some_call(matching!(_))
            .returns(Err(Error::GuestMismatch)),
        VmsMock::send
            .some_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, message| {
                captured.lock().unwrap().push(message.clone());
                Ok(())
            })),
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
        .control(ControlMessage::JobPreparation {
            command_id: CommandId("facts".into()),
            runner_id: RunnerId("job".into()),
            job_id: JobId("job-1".into()),
            outcome: JobPreparationOutcome::TrustedFacts {
                facts: Box::new(trusted_facts()),
            },
        })
        .await
        .unwrap();
    assert_eq!(
        *sent.lock().unwrap(),
        vec![AgentToGuest::PrepareJob {
            job_id: JobId("job-1".into()),
            decision: JobDecision::MachineCache {
                facts: Box::new(trusted_facts()),
                cache,
                docker: None
            },
        }]
    );
}
