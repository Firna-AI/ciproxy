//! Reconnect delays and revoked identities do not stop running jobs.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    for (scenario, handshake) in [
        ("permanent rejection", Err(Error::Revoked)),
        (
            "remove command",
            Ok(ControlMessage::Remove {
                command_id: CommandId("remove-1".to_owned()),
            }),
        ),
    ] {
        let removals = Arc::new(AtomicUsize::new(0));
        let removed = removals.clone();
        let dependencies = (
            ConnectionMock::connect
                .some_call(matching!())
                .returns(Ok(())),
            AgentMock::report
                .some_call(matching!())
                .returns(Ok(report.clone())),
            ConnectionMock::send.some_call(matching!(_)).returns(Ok(())),
            AgentMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    removed.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })),
        );
        let mock = match handshake {
            Ok(message) => Unimock::new((
                dependencies,
                ConnectionMock::receive_handshake
                    .some_call(matching!())
                    .returns(Ok(message)),
                AgentMock::control
                    .some_call(matching!(ControlMessage::Remove { .. }))
                    .returns(Ok(None)),
            )),
            Err(error) => Unimock::new((
                dependencies,
                ConnectionMock::receive_handshake
                    .some_call(matching!())
                    .returns(Err(error)),
            )),
        };
        let mock = Arc::new(mock);
        let result = ReconnectingGateway {
            connection: mock.clone(),
            agent: mock.clone(),
            delay: mock.clone(),
        }
        .reconnect(&identity())
        .await;
        assert!(matches!(result, Err(Error::Revoked)), "{scenario}");
        assert_eq!(removals.load(Ordering::SeqCst), 1, "{scenario}");
    }
}
