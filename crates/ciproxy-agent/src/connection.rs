//! Outbound TLS WebSocket with independent read and write locks.

use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::boundaries::Connection;
use crate::error::{Error, ErrorResultExt, Result};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(crate) struct WebSocketConnection {
    pub(crate) url: String,
    writer: Mutex<Option<SplitSink<Socket, Message>>>,
    reader: Mutex<Option<SplitStream<Socket>>>,
}

impl WebSocketConnection {
    pub(crate) fn new(url: String) -> Self {
        Self {
            url,
            writer: Mutex::new(None),
            reader: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Connection for WebSocketConnection {
    async fn receive_handshake(&self) -> Result<ControlMessage> {
        timeout(Duration::from_secs(20), self.receive())
            .await
            .context("control handshake timeout")?
    }

    async fn connect(&self) -> Result<()> {
        if !self.url.starts_with("wss://") {
            return Err(Error::HttpsRequired);
        }
        let result = timeout(Duration::from_secs(20), connect_async(&self.url))
            .await
            .context("connect control plane timeout")?;
        let (socket, _) = match result {
            Ok(socket) => socket,
            Err(tungstenite::Error::Http(response))
                if matches!(response.status().as_u16(), 401 | 403) =>
            {
                return Err(Error::Revoked);
            }
            Err(error) => return Err(error).context("connect control plane"),
        };
        let (writer, reader) = socket.split();
        *self.writer.lock().await = Some(writer);
        *self.reader.lock().await = Some(reader);
        Ok(())
    }

    async fn send(&self, message: &AgentMessage) -> Result<()> {
        let bytes = serde_json::to_string(message).context("encode agent message")?;
        let mut writer = self.writer.lock().await;
        let writer = writer.as_mut().ok_or(Error::Disconnected)?;
        timeout(
            Duration::from_secs(10),
            writer.send(Message::Text(bytes.into())),
        )
        .await
        .context("send control plane timeout")?
        .context("send control plane message")?;
        Ok(())
    }

    async fn receive(&self) -> Result<ControlMessage> {
        let mut reader = self.reader.lock().await;
        let reader = reader.as_mut().ok_or(Error::Disconnected)?;
        loop {
            match reader.next().await {
                Some(Ok(frame)) => {
                    if let Some(message) = decode_frame(frame)? {
                        return Ok(message);
                    }
                }
                None => return Err(Error::Disconnected),
                Some(Err(error)) => return Err(error).context("read control message"),
            }
        }
    }
}

/// Decode control messages and permanent credential rejection from wire frames.
pub(crate) fn decode_frame(frame: Message) -> Result<Option<ControlMessage>> {
    match frame {
        Message::Text(text) => Ok(Some(
            serde_json::from_str(&text).context("decode control message")?,
        )),
        Message::Close(Some(close))
            if close.code == CloseCode::Policy
                && close.reason.as_str() == "[gateway] authentication failed" =>
        {
            Err(Error::Revoked)
        }
        Message::Close(_) => Err(Error::Disconnected),
        _ => Ok(None),
    }
}

#[cfg(test)]
#[path = "_tests_/connection_tests.rs"]
mod connection_tests;
