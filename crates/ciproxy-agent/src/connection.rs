//! Outbound TLS WebSocket with independent read and write locks.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::agent::AgentMessage;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

use crate::boundaries::Connection;
use crate::connection_socket::{SocketReader, SocketWriter, WebSocketDialer};
use crate::connection_types::{
    ConnectionError, ConnectionErrorResultExt, ConnectionResult, ControlEvent,
};

struct Writer {
    socket: SocketWriter,
    hello_sent: Arc<AtomicBool>,
}

struct Reader {
    socket: SocketReader,
    hello_sent: Arc<AtomicBool>,
}

pub(crate) struct WebSocketConnection {
    url: String,
    dialer: Arc<dyn WebSocketDialer>,
    writer: Mutex<Option<Writer>>,
    reader: Mutex<Option<Reader>>,
}

impl WebSocketConnection {
    pub(crate) fn new(url: String, dialer: Arc<dyn WebSocketDialer>) -> Self {
        Self {
            url,
            dialer,
            writer: Mutex::new(None),
            reader: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Connection for WebSocketConnection {
    async fn receive_handshake(&self) -> ConnectionResult<ControlEvent> {
        timeout(Duration::from_secs(20), self.receive())
            .await
            .context("control handshake timeout")?
    }

    async fn connect(&self) -> ConnectionResult<()> {
        *self.writer.lock().await = None;
        *self.reader.lock().await = None;
        if !self.url.starts_with("wss://") {
            return Err(ConnectionError::HttpsRequired);
        }
        let socket = timeout(Duration::from_secs(20), self.dialer.dial(&self.url))
            .await
            .context("connect control plane timeout")?
            .context("connect control plane")?;
        let hello_sent = Arc::new(AtomicBool::new(false));
        *self.writer.lock().await = Some(Writer {
            socket: socket.writer,
            hello_sent: hello_sent.clone(),
        });
        *self.reader.lock().await = Some(Reader {
            socket: socket.reader,
            hello_sent,
        });
        Ok(())
    }

    async fn send(&self, message: &AgentMessage) -> ConnectionResult<()> {
        let bytes = serde_json::to_string(message).context("encode agent message")?;
        let mut writer = self.writer.lock().await;
        let writer = writer.as_mut().ok_or(ConnectionError::Disconnected)?;
        timeout(
            Duration::from_secs(10),
            writer.socket.send(Message::Text(bytes.into())),
        )
        .await
        .context("send control plane timeout")?
        .context("send control plane message")?;
        if matches!(message, AgentMessage::Hello { .. }) {
            writer.hello_sent.store(true, Ordering::Release);
        }
        Ok(())
    }

    async fn receive(&self) -> ConnectionResult<ControlEvent> {
        let mut reader = self.reader.lock().await;
        let reader = reader.as_mut().ok_or(ConnectionError::Disconnected)?;
        loop {
            match reader.socket.next().await {
                Some(Ok(frame)) => {
                    if let Some(event) = decode_frame(frame)? {
                        if matches!(event, ControlEvent::CredentialRejected)
                            && !reader.hello_sent.load(Ordering::Acquire)
                        {
                            return Err(ConnectionError::Disconnected);
                        }
                        return Ok(event);
                    }
                }
                None => return Err(ConnectionError::Disconnected),
                Some(Err(error)) => return Err(error).context("read control message"),
            }
        }
    }
}

fn decode_frame(frame: Message) -> ConnectionResult<Option<ControlEvent>> {
    match frame {
        Message::Text(text) => Ok(Some(ControlEvent::Message(Box::new(
            serde_json::from_str(&text).context("decode control message")?,
        )))),
        Message::Close(Some(close))
            if close.code == CloseCode::Policy
                && close.reason.as_str() == "[gateway] authentication failed" =>
        {
            Ok(Some(ControlEvent::CredentialRejected))
        }
        Message::Close(_) => Err(ConnectionError::Disconnected),
        _ => Ok(None),
    }
}

#[cfg(test)]
#[path = "_tests_/connection_tests.rs"]
mod connection_tests;
