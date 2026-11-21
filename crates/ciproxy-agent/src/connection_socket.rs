//! TLS dialing is separate from the machine credential exchange.

use async_trait::async_trait;
use futures_util::{Sink, Stream, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{self, Message};

/// An independent write half of the connected WebSocket.
pub(crate) type SocketWriter = Box<dyn Sink<Message, Error = tungstenite::Error> + Unpin + Send>;
/// An independent read half of the connected WebSocket.
pub(crate) type SocketReader =
    Box<dyn Stream<Item = std::result::Result<Message, tungstenite::Error>> + Unpin + Send>;

/// A completed WebSocket upgrade, before the agent sends a credential.
pub struct SocketParts {
    /// Outbound frames.
    pub(crate) writer: SocketWriter,
    /// Inbound frames.
    pub(crate) reader: SocketReader,
}

#[cfg_attr(test, unimock::unimock(api = WebSocketDialerMock))]
#[async_trait]
/// Connect and perform the WebSocket upgrade without machine authentication.
pub(crate) trait WebSocketDialer: Send + Sync {
    /// Return the stream halves or the original upgrade failure.
    async fn dial(&self, url: &str) -> std::result::Result<SocketParts, tungstenite::Error>;
}

/// Production dialer with normal TLS certificate verification.
pub(crate) struct TlsWebSocketDialer;

#[async_trait]
impl WebSocketDialer for TlsWebSocketDialer {
    async fn dial(&self, url: &str) -> std::result::Result<SocketParts, tungstenite::Error> {
        let (socket, _) = connect_async(url).await?;
        let (writer, reader) = socket.split();
        Ok(SocketParts {
            writer: Box::new(writer),
            reader: Box::new(reader),
        })
    }
}
