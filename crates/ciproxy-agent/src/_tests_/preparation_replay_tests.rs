//! Repeated hosted facts cannot change a drive after pre-action setup starts.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::cache::{CacheConnection, DockerDiskAccess, DockerDiskAttachment};
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::{CommandId, JobId, RunnerId};
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::secret::Secret;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, HostMock, StoreMock, VmsMock};
use crate::controller::Agent;
use crate::docker::DockerDisksMock;
use crate::fixtures::{applied, controller, hardware, trusted_facts, vm};
use crate::state::{MachineState, PreparationState};

#[tokio::test]
async fn repeated_facts_do_not_repeat_attachment_before_the_guest_acknowledges_setup() {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let sent = messages.clone();
    let cache = CacheConnection {
        url: Secret::new("http://machine:7887"),
        token: Secret::new("same"),
    };
    let connection = cache.clone();
    let disk = DockerDiskAttachment {
        free_bytes: 80_000_000_000,
        drive_id: "docker".into(),
        access: DockerDiskAccess::DisposableCopy,
    };
    let attachment = disk.clone();
    let mock = Unimock::new((
        StoreMock::save.some_call(matching!(_)).returns(Ok(())),
        CacheMock::prepare
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, _| {
                Ok(Some((
                    connection.clone(),
                    JobScope::Unknown {
                        reason: UnknownJobScopeReason::MissingToken,
                    },
                    7,
                )))
            })),
        HostMock::hardware
            .some_call(matching!())
            .returns(Ok(hardware())),
        DockerDisksMock::prepare
            .some_call(matching!(_, _, _, 7, 0, _))
            .returns(Ok(Some(attachment))),
        VmsMock::swap_docker.some_call(matching!(_)).returns(Ok(())),
        VmsMock::send
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, message| {
                sent.lock().unwrap().push(message.clone());
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
    for _ in 0..2 {
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
    assert_eq!(
        *messages.lock().unwrap(),
        vec![
            AgentToGuest::PrepareJob {
                job_id: JobId("job-1".into()),
                facts: trusted_facts(),
                cache: Some(cache.clone()),
                docker: Some(disk)
            },
            AgentToGuest::PrepareJob {
                job_id: JobId("job-1".into()),
                facts: trusted_facts(),
                cache: Some(cache),
                docker: None
            },
        ]
    );
    assert_eq!(
        agent.state.lock().await.runners[0].preparation,
        PreparationState::Started
    );
}
