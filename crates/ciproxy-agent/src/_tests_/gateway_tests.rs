//! Reconnect delays and revoked identities do not stop running jobs.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Poll;

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::version::ProtocolVersion;
use futures_util::{SinkExt, StreamExt, pin_mut, poll};
use tokio_tungstenite::tungstenite::{
    Message, protocol::CloseFrame, protocol::frame::coding::CloseCode,
};
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{ConnectionMock, DelayMock};
use crate::connection_fixtures::{connection, websocket};
use crate::connection_types::{ConnectionError, ControlEvent};
use crate::controller::AgentMock;
use crate::error::Error;
use crate::fixtures::{identity, report};
use crate::gateway::{Gateway, ReconnectingGateway};

#[tokio::test]
async fn reconnect_grows_its_delay_and_sends_the_full_retained_state() {
    let report = report().await;
    let mock = Arc::new(Unimock::new((
        ConnectionMock::connect.stub(|each| { each.call(matching!()).answers(&|_| Err(ConnectionError::Disconnected)).n_times(3)
            .then().answers(&|_| Ok(())); }),
        DelayMock::wait.next_call(matching!(1)).returns(()),
        DelayMock::wait.next_call(matching!(2)).returns(()),
        DelayMock::wait.next_call(matching!(4)).returns(()),
        AgentMock::report.some_call(matching!()).returns(Ok(report)),
        ConnectionMock::send.some_call(matching!((message) if matches!(message, AgentMessage::Hello {
            workspace_id, machine_id, credential, report, .. } if workspace_id.0 == "workspace-1" && machine_id.0 == "machine-1"
            && credential.expose_secret() == "machine-secret" && report.hardware.threads == 16))).returns(Ok(())),
        ConnectionMock::receive_handshake.some_call(matching!()).returns(Ok(ControlEvent::Message(Box::new(ControlMessage::Welcome { protocol_version: ProtocolVersion::CURRENT })))),
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
    for (scenario, frame, removed_by_command) in [
        (
            "permanent rejection",
            Message::Close(Some(CloseFrame {
                code: CloseCode::Policy,
                reason: "[gateway] authentication failed".into(),
            })),
            false,
        ),
        (
            "remove command",
            Message::Text(
                serde_json::to_string(&ControlMessage::Remove {
                    command_id: CommandId("remove-1".to_owned()),
                })
                .unwrap()
                .into(),
            ),
            true,
        ),
    ] {
        let (socket, mut peer) = websocket().await;
        let removals = Arc::new(AtomicUsize::new(0));
        let removed = removals.clone();
        let dependencies = (
            AgentMock::report
                .some_call(matching!())
                .returns(Ok(report.clone())),
            AgentMock::remove
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    removed.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })),
        );
        let mock = if removed_by_command {
            Unimock::new((
                dependencies,
                AgentMock::control
                    .some_call(matching!(ControlMessage::Remove { .. }))
                    .returns(Ok(None)),
            ))
        } else {
            Unimock::new(dependencies)
        };
        let mock = Arc::new(mock);
        let gateway = ReconnectingGateway {
            connection: Arc::new(connection(Ok(socket))),
            agent: mock.clone(),
            delay: mock,
        };
        let identity = identity();
        let result = gateway.reconnect(&identity);
        pin_mut!(result);
        assert!(matches!(poll!(result.as_mut()), Poll::Pending));
        let hello = peer.next().await.unwrap().unwrap();
        assert!(matches!(
            serde_json::from_str::<AgentMessage>(hello.to_text().unwrap()).unwrap(),
            AgentMessage::Hello { .. }
        ));
        peer.send(frame).await.unwrap();
        let result = result.await;
        assert!(matches!(result, Err(Error::Revoked)), "{scenario}");
        assert_eq!(removals.load(Ordering::SeqCst), 1, "{scenario}");
    }
}

#[tokio::test]
async fn hello_preparation_and_send_failures_retry_without_removing_the_machine() {
    let snapshot = report().await;
    let removals = Arc::new(AtomicUsize::new(0));
    let removed = removals.clone();
    let mock = Arc::new(Unimock::new((
        ConnectionMock::connect
            .each_call(matching!())
            .answers(&|_| Ok(())),
        AgentMock::report.stub(|each| {
            each.call(matching!())
                .answers(&|_| Err(Error::Revoked))
                .n_times(1)
                .then()
                .answers_arc(Arc::new(move |_| Ok(snapshot.clone())));
        }),
        ConnectionMock::send.stub(|each| {
            each.call(matching!(_))
                .answers(&|_, _| Err(ConnectionError::Disconnected))
                .n_times(1)
                .then()
                .answers(&|_, _| Ok(()));
        }),
        ConnectionMock::receive_handshake.stub(|each| {
            each.call(matching!())
                .answers(&|_| {
                    Ok(ControlEvent::Message(Box::new(ControlMessage::Welcome {
                        protocol_version: ProtocolVersion::CURRENT,
                    })))
                })
                .n_times(1)
                .then()
                .answers(&|_| Ok(ControlEvent::CredentialRejected));
        }),
        DelayMock::wait.next_call(matching!(1)).returns(()),
        DelayMock::wait.next_call(matching!(2)).returns(()),
        AgentMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                removed.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
    )));
    let gateway = ReconnectingGateway {
        connection: mock.clone(),
        agent: mock.clone(),
        delay: mock,
    };
    let result = gateway.reconnect(&identity()).await;
    assert!(matches!(result, Ok(())));
    assert_eq!(removals.load(Ordering::SeqCst), 0);
    let result = gateway.reconnect(&identity()).await;
    assert!(matches!(result, Err(Error::Revoked)));
    assert_eq!(removals.load(Ordering::SeqCst), 1);
}
