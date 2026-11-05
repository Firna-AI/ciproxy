//! Only rejection after this socket sends Hello can remove a machine.

use ciproxy_protocol::agent::AgentMessage;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

use crate::boundaries::Connection;
use crate::connection::WebSocketConnection;
use crate::connection_fixtures::{URL, dialer, hello, websocket};
use crate::connection_types::{ConnectionError, ControlEvent};

#[derive(Debug, Eq, PartialEq)]
enum Outcome {
    Rejected,
    Disconnected,
    Unexpected,
}

#[tokio::test]
async fn only_the_exact_gateway_policy_rejection_revokes_the_credential() {
    let hello = hello().await;
    for (scenario, close, send_hello, expected) in [
        (
            "permanent credential rejection",
            Some((CloseCode::Policy, "[gateway] authentication failed")),
            true,
            Outcome::Rejected,
        ),
        (
            "unsupported protocol version",
            Some((CloseCode::Policy, "[gateway] unsupported protocol version")),
            true,
            Outcome::Disconnected,
        ),
        (
            "connection replaced",
            Some((CloseCode::Policy, "[gateway] connection replaced")),
            true,
            Outcome::Disconnected,
        ),
        (
            "same reason with an ordinary close code",
            Some((CloseCode::Normal, "[gateway] authentication failed")),
            true,
            Outcome::Disconnected,
        ),
        (
            "different reason with the same prefix",
            Some((
                CloseCode::Policy,
                "[gateway] authentication failed: details",
            )),
            true,
            Outcome::Disconnected,
        ),
        ("no close payload", None, true, Outcome::Disconnected),
        (
            "before Hello on a replacement socket",
            Some((CloseCode::Policy, "[gateway] authentication failed")),
            false,
            Outcome::Disconnected,
        ),
    ] {
        let (previous, mut previous_peer) = websocket().await;
        let (current, mut peer) = websocket().await;
        let connection =
            WebSocketConnection::new(URL.to_owned(), dialer(vec![Ok(previous), Ok(current)]));
        connection.connect().await.unwrap();
        connection.send(&hello).await.unwrap();
        assert_eq!(
            serde_json::from_str::<AgentMessage>(
                previous_peer
                    .next()
                    .await
                    .unwrap()
                    .unwrap()
                    .to_text()
                    .unwrap()
            )
            .unwrap(),
            hello,
        );
        connection.connect().await.unwrap();
        let received = if send_hello {
            connection.send(&hello).await.unwrap();
            Some(
                serde_json::from_str::<AgentMessage>(
                    peer.next().await.unwrap().unwrap().to_text().unwrap(),
                )
                .unwrap(),
            )
        } else {
            None
        };
        assert_eq!(received, send_hello.then(|| hello.clone()));
        let frame = close.map(|(code, reason)| CloseFrame {
            code,
            reason: reason.into(),
        });
        peer.send(Message::Close(frame)).await.unwrap();
        let outcome = match connection.receive_handshake().await {
            Ok(ControlEvent::CredentialRejected) => Outcome::Rejected,
            Err(ConnectionError::Disconnected) => Outcome::Disconnected,
            _ => Outcome::Unexpected,
        };
        assert_eq!(outcome, expected, "{scenario}");
    }
}
