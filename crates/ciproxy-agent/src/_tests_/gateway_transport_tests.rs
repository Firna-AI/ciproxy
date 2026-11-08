//! HTTP upgrade denials cannot revoke machine identity or VM ownership.

use std::sync::Arc;
use std::task::Poll;

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::runners::RunnerState;
use ciproxy_protocol::version::ProtocolVersion;
use futures_util::{SinkExt, StreamExt, pin_mut, poll};
use tokio_tungstenite::tungstenite::Message;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{DelayMock, StoreMock, VmsMock};
use crate::cache_authority::CacheAuthority;
use crate::connection::WebSocketConnection;
use crate::connection_fixtures::{URL, dialer, upgrade_error, websocket};
use crate::error::Error;
use crate::fixtures::github_decision_sent;
use crate::fixtures::{controller, identity, report_dependencies, vm};
use crate::gateway::{Gateway, ReconnectingGateway};
use crate::state::{MachineState, Mode};

#[tokio::test]
async fn http_upgrade_denials_retry_without_changing_identity_or_vms() {
    let unauthorized = upgrade_error(401).await;
    let forbidden = upgrade_error(403).await;
    let (socket, mut peer) = websocket().await;
    let (removal_socket, mut removal_peer) = websocket().await;
    let welcome = ControlMessage::Welcome {
        protocol_version: ProtocolVersion::CURRENT,
    };
    peer.send(Message::Text(
        serde_json::to_string(&welcome).unwrap().into(),
    ))
    .await
    .unwrap();
    let dialer = dialer(vec![
        Err(unauthorized),
        Err(forbidden),
        Ok(socket),
        Ok(removal_socket),
    ]);
    let initial = MachineState {
        failed_starts: Default::default(),
        pending_networks: Default::default(),
        docker_budget: None,
        identity: Some(identity()),
        mode: Mode::Active,
        runners: vec![vm("running", true), vm("warm", false)],
    };
    let agent = Arc::new(controller(
        initial.clone(),
        Unimock::new((
            github_decision_sent(),
            report_dependencies(),
            StoreMock::save
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
            VmsMock::request_stop
                .each_call(matching!(_))
                .answers(&|_, _| Ok(())),
        )),
    ));
    let delay = Arc::new(Unimock::new((
        DelayMock::wait.next_call(matching!(1)).returns(()),
        DelayMock::wait.next_call(matching!(2)).returns(()),
    )));
    let gateway = ReconnectingGateway {
        connection: Arc::new(WebSocketConnection::new(URL.to_owned(), dialer)),
        agent: agent.clone(),
        delay,
    };
    let result = gateway.reconnect(&identity()).await;
    assert!(matches!(result, Ok(())));
    assert_eq!(*agent.state.lock().await, initial);
    let hello = serde_json::from_str::<AgentMessage>(
        peer.next().await.unwrap().unwrap().to_text().unwrap(),
    )
    .unwrap();
    let AgentMessage::Hello {
        workspace_id,
        machine_id,
        credential,
        report,
        ..
    } = hello
    else {
        panic!("expected Hello after retry")
    };
    assert_eq!(workspace_id, identity().workspace_id);
    assert_eq!(machine_id, identity().machine_id);
    assert_eq!(credential, identity().credential);
    assert_eq!(
        report.runners,
        initial
            .runners
            .iter()
            .map(|vm| vm.report.clone())
            .collect::<Vec<_>>()
    );
    let identity = identity();
    let result = gateway.reconnect(&identity);
    pin_mut!(result);
    assert!(matches!(poll!(result.as_mut()), Poll::Pending));
    let hello = removal_peer.next().await.unwrap().unwrap();
    assert!(matches!(
        serde_json::from_str::<AgentMessage>(hello.to_text().unwrap()).unwrap(),
        AgentMessage::Hello { .. }
    ));
    let remove = ControlMessage::Remove {
        command_id: CommandId("authenticated-remove".to_owned()),
    };
    removal_peer
        .send(Message::Text(
            serde_json::to_string(&remove).unwrap().into(),
        ))
        .await
        .unwrap();
    assert!(matches!(result.await, Err(Error::Revoked)));
    let mut expected = initial;
    expected.mode = Mode::Removed;
    expected.runners[0].cache_authority = CacheAuthority::GitHubOnly;
    expected.runners[1].report.state = RunnerState::Stopping;
    assert_eq!(*agent.state.lock().await, expected);
}
