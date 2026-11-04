//! Reconnect delays and revoked identities do not stop running jobs.

use std::sync::Arc;

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::version::ProtocolVersion;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{ConnectionMock, DelayMock};
use crate::controller::AgentMock;
use crate::error::Error;
use crate::fixtures::{identity, report};
use crate::gateway::{Gateway, ReconnectingGateway};

#[tokio::test]
async fn reconnect_grows_its_delay_and_sends_the_full_retained_state() {
    let report = report().await;
    let mock = Arc::new(Unimock::new((
        ConnectionMock::connect.stub(|each| { each.call(matching!()).answers(&|_| Err(Error::Disconnected)).n_times(3)
            .then().answers(&|_| Ok(())); }),
        DelayMock::wait.next_call(matching!(1)).returns(()),
        DelayMock::wait.next_call(matching!(2)).returns(()),
        DelayMock::wait.next_call(matching!(4)).returns(()),
        AgentMock::report.some_call(matching!()).returns(Ok(report)),
        ConnectionMock::send.some_call(matching!((message) if matches!(message, AgentMessage::Hello {
            workspace_id, machine_id, credential, report, .. } if workspace_id.0 == "workspace-1" && machine_id.0 == "machine-1"
            && credential.expose_secret() == "machine-secret" && report.hardware.threads == 16))).returns(Ok(())),
        ConnectionMock::receive_handshake.some_call(matching!()).returns(Ok(ControlMessage::Welcome { protocol_version: ProtocolVersion::CURRENT })),
    )));
    ReconnectingGateway {
        connection: mock.clone(),
        agent: mock.clone(),
        delay: mock,
    }
    .reconnect(&identity())
    .await
    .unwrap();
}

#[tokio::test]
async fn a_revoked_hello_removes_the_machine_before_waiting_for_welcome() {
    let report = report().await;
    let mock = Arc::new(Unimock::new((
        ConnectionMock::connect
            .some_call(matching!())
            .returns(Ok(())),
        AgentMock::report.some_call(matching!()).returns(Ok(report)),
        ConnectionMock::send.some_call(matching!(_)).returns(Ok(())),
        ConnectionMock::receive_handshake
            .some_call(matching!())
            .returns(Ok(ControlMessage::Remove {
                command_id: CommandId("remove-1".to_owned()),
            })),
        AgentMock::control
            .some_call(matching!(ControlMessage::Remove { .. }))
            .returns(Ok(None)),
        AgentMock::remove.some_call(matching!()).returns(Ok(())),
    )));
    assert!(matches!(
        ReconnectingGateway {
            connection: mock.clone(),
            agent: mock.clone(),
            delay: mock
        }
        .reconnect(&identity())
        .await,
        Err(Error::Revoked)
    ));
}
