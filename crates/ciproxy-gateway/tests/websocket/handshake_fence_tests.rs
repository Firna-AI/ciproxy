//! Focused actual WebSocket output when handshake reconciliation is stale.

use std::sync::Arc;

use ciproxy_gateway::connections::{Connections, ConnectionsMock, LocalConnections};
use ciproxy_gateway::gateway::{MachineGateway, MachineGatewayMock};
use ciproxy_gateway::transport::{AgentTransport, WebSocketTransport};
use ciproxy_gateway::types::{Handshake, Session};
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{CommandId, MachineId};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use tokio::net::TcpListener;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::fixture::WORKSPACE;
use super::messages::{hello, report};
use super::socket::FakeAgent;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn closes_before_sending_a_stale_handshake_drain() {
    let session = Session {
        workspace_id: WORKSPACE,
        machine_id: Uuid::from_u128(3),
        id: Uuid::from_u128(4),
    };
    let connections: Arc<dyn Connections> = Arc::new(LocalConnections::default());
    let opening = connections.clone();
    let gateway_mock = Unimock::new((
        MachineGatewayMock::open
            .some_call(matching!(AgentMessage::Hello { .. }))
            .answers_arc(Arc::new(move |_, _| {
                Ok(Handshake {
                    session,
                    outbox: opening.attach(session)?,
                    replies: vec![
                        ControlMessage::Welcome {
                            protocol_version: ProtocolVersion::CURRENT,
                        },
                        ControlMessage::Drain {
                            command_id: CommandId("old-drain".into()),
                        },
                    ],
                })
            })),
        MachineGatewayMock::is_current
            .some_call(matching!((scope) if scope.id == Uuid::from_u128(4)))
            .returns(Ok(false)),
    ));
    let gateway: Arc<dyn MachineGateway> = Arc::new(gateway_mock.clone());
    let registry_mock = Unimock::new(
        ConnectionsMock::detach
            .some_call(matching!((scope) if scope.id == Uuid::from_u128(4)))
            .answers_arc(Arc::new(move |_, scope| connections.detach(scope))),
    );
    let transport: &dyn AgentTransport =
        &WebSocketTransport::new(gateway.clone(), Arc::new(registry_mock.clone()));
    let router = transport.router();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/agent/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let mut agent = FakeAgent::connect(&url).await;
    agent
        .send(hello(
            WORKSPACE,
            MachineId(session.machine_id.to_string()),
            Secret::new("dummy-credential"),
            1,
            report(),
        ))
        .await;
    assert_eq!(
        agent.next().await,
        ControlMessage::Welcome {
            protocol_version: ProtocolVersion::CURRENT
        }
    );
    agent.closed("[gateway] connection replaced").await;
    server.abort();
}
