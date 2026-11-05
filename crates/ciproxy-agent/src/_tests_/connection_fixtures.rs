//! Real in-memory WebSocket handshakes at the production dial boundary.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::version::ProtocolVersion;
use futures_util::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream, duplex};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{WebSocketStream, accept_async, client_async};
use unimock::{MockFn, Unimock, matching};

use crate::connection::WebSocketConnection;
use crate::connection_socket::{SocketParts, WebSocketDialer, WebSocketDialerMock};
use crate::fixtures::{identity, report};

pub(crate) const URL: &str = "wss://gateway.test/agent/";

pub(crate) fn connection(
    result: std::result::Result<SocketParts, tungstenite::Error>,
) -> WebSocketConnection {
    WebSocketConnection::new(URL.to_owned(), dialer(vec![result]))
}

pub(crate) fn dialer(
    results: Vec<std::result::Result<SocketParts, tungstenite::Error>>,
) -> Arc<dyn WebSocketDialer> {
    let count = results.len();
    let results = Mutex::new(VecDeque::from(results));
    Arc::new(Unimock::new(
        WebSocketDialerMock::dial
            .some_call(matching!((url) if *url == URL))
            .answers_arc(Arc::new(move |_, _| {
                results.lock().unwrap().pop_front().unwrap()
            }))
            .n_times(count),
    ))
}

pub(crate) async fn websocket() -> (SocketParts, WebSocketStream<DuplexStream>) {
    let (client, server) = duplex(64 * 1024);
    let (client, server) = tokio::join!(client_async(URL, client), accept_async(server));
    let (writer, reader) = client.unwrap().0.split();
    (
        SocketParts {
            writer: Box::new(writer),
            reader: Box::new(reader),
        },
        server.unwrap(),
    )
}

pub(crate) async fn upgrade_error(status: u16) -> tungstenite::Error {
    let (client, mut server) = duplex(64 * 1024);
    let serve = async {
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            request.push(server.read_u8().await.unwrap());
        }
        let request = String::from_utf8(request).unwrap();
        assert!(request.starts_with("GET /agent/ HTTP/1.1\r\n"));
        assert!(!request.contains("machine-secret"));
        let response = format!("HTTP/1.1 {status} Denied\r\nContent-Length: 0\r\n\r\n");
        server.write_all(response.as_bytes()).await.unwrap();
        server.shutdown().await.unwrap();
    };
    let (result, ()) = tokio::join!(client_async(URL, client), serve);
    assert!(
        matches!(&result, Err(tungstenite::Error::Http(response)) if response.status().as_u16() == status)
    );
    result.unwrap_err()
}

pub(crate) async fn hello() -> AgentMessage {
    let identity = identity();
    AgentMessage::Hello {
        protocol_version: ProtocolVersion::CURRENT,
        workspace_id: identity.workspace_id,
        machine_id: identity.machine_id,
        credential: identity.credential,
        report: report().await,
    }
}
