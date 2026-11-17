//! Real gateway WebSocket read path with explicit saved-report acknowledgement.

use std::sync::Arc;
use std::time::Duration;

use ciproxy_gateway::connections::Connections;
use ciproxy_gateway::gateway::{MachineGateway, MachineGatewayMock};
use ciproxy_gateway::transport::{AgentTransport, WebSocketTransport};
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{MachineId, WorkspaceId};
use ciproxy_protocol::reports::MachineReport;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use futures_util::{SinkExt, StreamExt};
use sha2::{Digest, Sha256};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::support::{Fixture, at};

pub(super) struct Socket {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    server: JoinHandle<()>,
    pub(super) report_saved: Arc<Notify>,
    _observed: Unimock,
}

impl Socket {
    pub(super) async fn open(
        fixture: &Fixture,
        machine: Uuid,
        report: MachineReport,
        gateway: Arc<dyn MachineGateway>,
        connections: Arc<dyn Connections>,
    ) -> Self {
        let credential = Secret::new("progress-fixture-credential");
        let hash = TokenHash::new(Sha256::digest(credential.expose_secret().as_bytes()).to_vec());
        fixture
            .store
            .create_scoped_token(
                &Token {
                    id: Uuid::new_v4(),
                    workspace_id: fixture.workspace,
                    subject: TokenSubject::Machine(machine),
                    created_at: at(),
                    expires_at: None,
                    revoked_at: None,
                },
                &hash,
            )
            .unwrap();
        let report_saved = Arc::new(Notify::new());
        let saved = report_saved.clone();
        let open = gateway.clone();
        let receive = gateway.clone();
        let observed = Unimock::new((
            MachineGatewayMock::open
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, message| open.open(message))),
            MachineGatewayMock::receive
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, session, message| {
                    let report = matches!(message, AgentMessage::Report { .. });
                    let result = receive.receive(session, message);
                    if result.is_ok() && report {
                        saved.notify_one();
                    }
                    result
                })),
            MachineGatewayMock::is_current
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, session| gateway.is_current(session))),
        ));
        let transport = WebSocketTransport::new(Arc::new(observed.clone()), connections);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/agent/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, transport.router()).await.unwrap();
        });
        let (socket, _) = connect_async(url).await.unwrap();
        let mut client = Self {
            socket,
            server,
            report_saved,
            _observed: observed,
        };
        client
            .send(AgentMessage::Hello {
                protocol_version: ProtocolVersion::CURRENT,
                workspace_id: WorkspaceId(fixture.workspace.to_string()),
                machine_id: MachineId(machine.to_string()),
                credential,
                report,
            })
            .await;
        assert!(matches!(
            client.next().await,
            ControlMessage::Welcome { .. }
        ));
        assert!(matches!(client.next().await, ControlMessage::Drain { .. }));
        client
    }

    pub(super) async fn send(&mut self, message: AgentMessage) {
        self.socket
            .send(Message::Text(
                serde_json::to_string(&message).unwrap().into(),
            ))
            .await
            .unwrap();
    }

    pub(super) async fn next(&mut self) -> ControlMessage {
        let message = timeout(Duration::from_secs(30), self.socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        serde_json::from_str(message.to_text().unwrap()).unwrap()
    }

    /// Finish the connection and join the listener before mock owners verify.
    pub(super) async fn close(mut self) {
        self.socket.close(None).await.unwrap();
        timeout(Duration::from_secs(30), async {
            while self.socket.next().await.is_some() {}
        })
        .await
        .unwrap();
        self.server.abort();
        let stopped = (&mut self.server).await;
        assert!(matches!(stopped, Err(error) if error.is_cancelled()));
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.server.abort();
    }
}
