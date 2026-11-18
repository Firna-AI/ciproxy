//! Fake agent using the actual local WebSocket wire boundary.

use std::time::Duration;

use ciproxy_protocol::agent::{AgentMessage, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::http::StatusCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::Frame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::{CloseCode, Data, OpCode};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::fixture::Fixture;

pub(super) struct FakeAgent(WebSocketStream<MaybeTlsStream<TcpStream>>);

impl FakeAgent {
    pub(super) async fn connect(url: &str) -> Self {
        Self(connect_async(url).await.unwrap().0)
    }

    pub(super) async fn upgrade_status(url: &str) -> StatusCode {
        match connect_async(url).await {
            Ok(_) => StatusCode::SWITCHING_PROTOCOLS,
            Err(Error::Http(response)) => response.status(),
            Err(error) => panic!("unexpected upgrade failure: {error}"),
        }
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
        let result = timeout(Duration::from_secs(10), self.0.next()).await;
        assert!(result.is_ok(), "gateway did not send the expected close");
        let frame = result.unwrap().unwrap().unwrap();
        let expected = Message::Close(Some(CloseFrame {
            code: CloseCode::Policy,
            reason: reason.into(),
        }));
        assert_eq!(frame, expected);
    }

    pub(super) async fn send_fragmented(&mut self, message: AgentMessage) {
        let text = serde_json::to_vec(&message).unwrap();
        let chunks: Vec<_> = text.chunks(1024 * 1024).collect();
        for (index, chunk) in chunks.iter().enumerate() {
            let opcode = if index == 0 {
                Data::Text
            } else {
                Data::Continue
            };
            self.0
                .send(Message::Frame(Frame::message(
                    chunk.to_vec(),
                    OpCode::Data(opcode),
                    index + 1 == chunks.len(),
                )))
                .await
                .unwrap();
        }
    }

    pub(super) async fn announce_text_frame(&mut self, payload_bytes: u16) {
        let MaybeTlsStream::Plain(socket) = self.0.get_mut() else {
            panic!("loopback fixture must use a plain TCP stream");
        };
        let length = payload_bytes.to_be_bytes();
        socket
            .write_all(&[0x81, 0xfe, length[0], length[1], 0, 0, 0, 0])
            .await
            .unwrap();
    }

    pub(super) async fn acknowledge_close(&mut self) {
        self.0.flush().await.unwrap();
        self.server_ended_without_reply().await;
    }

    pub(super) async fn server_ended_without_reply(&mut self) {
        let MaybeTlsStream::Plain(socket) = self.0.get_mut() else {
            panic!("loopback fixture must use a plain TCP stream");
        };
        let mut byte = [0];
        let result = timeout(Duration::from_secs(10), socket.read(&mut byte)).await;
        assert!(result.is_ok(), "gateway did not finish the close handshake");
        assert_eq!(result.unwrap().unwrap(), 0);
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
