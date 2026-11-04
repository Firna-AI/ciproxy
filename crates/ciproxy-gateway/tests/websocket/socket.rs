//! Fake agent using the actual local WebSocket wire boundary.

use std::time::Duration;

use ciproxy_protocol::agent::{AgentMessage, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::fixture::Fixture;

pub(super) struct FakeAgent(WebSocketStream<MaybeTlsStream<TcpStream>>);

impl FakeAgent {
    pub(super) async fn connect(url: &str) -> Self {
        Self(connect_async(url).await.unwrap().0)
    }

    pub(super) async fn send(&mut self, message: AgentMessage) {
        self.0
            .send(Message::Text(
                serde_json::to_string(&message).unwrap().into(),
            ))
            .await
            .unwrap();
    }

    pub(super) async fn next(&mut self) -> ControlMessage {
        let frame = timeout(Duration::from_secs(10), self.0.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            matches!(frame, Message::Text(_)),
            "unexpected frame {frame:?}"
        );
        serde_json::from_str(frame.to_text().unwrap()).unwrap()
    }

    pub(super) async fn closed(&mut self, reason: &str) {
        let frame = timeout(Duration::from_secs(10), self.0.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let expected = Message::Close(Some(CloseFrame {
            code: CloseCode::Policy,
            reason: reason.into(),
        }));
        assert_eq!(frame, expected);
    }

    pub(super) async fn fence(&mut self, fixture: &mut Fixture) {
        let message = AgentMessage::CommandResult {
            command_id: CommandId("test-fence".into()),
            outcome: CommandOutcome::Applied,
        };
        self.send(message.clone()).await;
        let (_, actual) = timeout(Duration::from_secs(10), fixture.events.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(actual, message);
    }

    pub(super) async fn disconnect(&mut self) {
        self.0.close(None).await.unwrap();
    }
}
