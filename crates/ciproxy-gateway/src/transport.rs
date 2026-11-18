//! Axum WebSocket adapter for outbound agent connections at /agent/.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;

use crate::connections::{Connections, Outbox};
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::MachineGateway;
use crate::types::{Handshake, Session};

/// The HTTP composition boundary, mounted behind the control plane's TLS ingress.
pub trait AgentTransport: Send + Sync {
    /// Return the agent route for merging into the control-plane router.
    fn router(&self) -> Router;
}

/// Resource limits applied to each transport and its agent connections.
#[derive(Clone, Copy, Debug)]
pub struct TransportLimits {
    /// Maximum bytes in one WebSocket message or frame.
    pub max_message_bytes: usize,
    /// Time allowed for the first complete agent message.
    pub first_message_timeout: Duration,
    /// Total time allowed to send a close and wait for the peer.
    pub close_timeout: Duration,
    /// Maximum connections that have not completed authentication.
    pub max_pending_handshakes: usize,
}

impl Default for TransportLimits {
    fn default() -> Self {
        Self {
            max_message_bytes: 16 * 1024 * 1024,
            first_message_timeout: Duration::from_secs(15),
            close_timeout: Duration::from_secs(5),
            max_pending_handshakes: 8,
        }
    }
}

/// WebSocket transport using the same gateway and mailboxes as command callers.
#[derive(Clone)]
pub struct WebSocketTransport {
    gateway: Arc<dyn MachineGateway>,
    connections: Arc<dyn Connections>,
    limits: TransportLimits,
    pending_handshakes: Arc<Semaphore>,
}

impl WebSocketTransport {
    /// Construct at the HTTP composition root, then consume through dyn AgentTransport.
    pub fn new(gateway: Arc<dyn MachineGateway>, connections: Arc<dyn Connections>) -> Self {
        Self::with_limits(gateway, connections, TransportLimits::default())
    }

    /// Construct a transport with explicit resource limits.
    pub fn with_limits(
        gateway: Arc<dyn MachineGateway>,
        connections: Arc<dyn Connections>,
        limits: TransportLimits,
    ) -> Self {
        Self {
            gateway,
            connections,
            limits,
            pending_handshakes: Arc::new(Semaphore::new(limits.max_pending_handshakes)),
        }
    }

    async fn run(self, mut socket: WebSocket, permit: OwnedSemaphorePermit) {
        let handshake = {
            let _permit = permit;
            match self.authenticate(&mut socket).await {
                Ok(Some(handshake)) => handshake,
                Ok(None) => return,
                Err(error) => {
                    close(&mut socket, error, self.limits.close_timeout).await;
                    return;
                }
            }
        };
        let session = handshake.session;
        let result = self.serve_authenticated(&mut socket, handshake).await;
        if let Err(error) = self.connections.detach(session) {
            tracing::warn!(%error, "agent connection cleanup failed");
        }
        if let Err(error) = result {
            close(&mut socket, error, self.limits.close_timeout).await;
        }
    }

    async fn authenticate(&self, socket: &mut WebSocket) -> Result<Option<Handshake>> {
        let message = match timeout(self.limits.first_message_timeout, receive(socket)).await {
            Ok(message) => message?,
            Err(_) => return Err(Error::HandshakeTimeout),
        };
        let Some(message) = message else {
            return Ok(None);
        };
        let gateway = self.gateway.clone();
        let handshake = tokio::task::spawn_blocking(move || gateway.open(message))
            .await
            .internal()??;
        Ok(Some(handshake))
    }

    async fn serve_authenticated(
        &self,
        socket: &mut WebSocket,
        handshake: Handshake,
    ) -> Result<()> {
        for reply in handshake.replies {
            if matches!(
                reply,
                ControlMessage::Drain { .. } | ControlMessage::Resume { .. }
            ) {
                let gateway = self.gateway.clone();
                let session = handshake.session;
                let current = tokio::task::spawn_blocking(move || gateway.is_current(session))
                    .await
                    .internal()??;
                if !current {
                    return Err(Error::Replaced);
                }
            }
            send(socket, reply).await?;
        }
        self.serve_session(socket, handshake.session, handshake.outbox)
            .await
    }

    async fn serve_session(
        &self,
        socket: &mut WebSocket,
        session: Session,
        mut outbox: Box<dyn Outbox>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                biased;
                command = outbox.next() => {
                    let Some(command) = command else { return Err(Error::Replaced); };
                    let removal = matches!(command, ControlMessage::Remove { .. });
                    let gateway = self.gateway.clone();
                    let current = tokio::task::spawn_blocking(move || gateway.is_current(session)).await.internal()??;
                    if current || removal { send(socket, command).await?; }
                }
                message = receive(socket) => {
                    let Some(message) = message? else { return Ok(()); };
                    let gateway = self.gateway.clone();
                    match tokio::task::spawn_blocking(move || gateway.receive(session, message)).await.internal()? {
                        Ok(()) => (),
                        Err(error) => return Err(error),
                    }
                }
            }
        }
    }
}

impl AgentTransport for WebSocketTransport {
    fn router(&self) -> Router {
        let transport = self.clone();
        Router::new().route(
            "/agent/",
            get(move |upgrade: WebSocketUpgrade| {
                let transport = transport.clone();
                async move {
                    let Ok(permit) = transport.pending_handshakes.clone().try_acquire_owned()
                    else {
                        return StatusCode::SERVICE_UNAVAILABLE.into_response();
                    };
                    upgrade
                        .max_message_size(transport.limits.max_message_bytes)
                        .max_frame_size(transport.limits.max_message_bytes)
                        .on_upgrade(move |socket| transport.run(socket, permit))
                        .into_response()
                }
            }),
        )
    }
}

async fn receive(socket: &mut WebSocket) -> Result<Option<AgentMessage>> {
    loop {
        let Some(message) = socket.recv().await else {
            return Ok(None);
        };
        match message.internal()? {
            Message::Text(text) => {
                return match serde_json::from_str(&text) {
                    Ok(message) => Ok(Some(message)),
                    Err(_) => Err(Error::InvalidMessage),
                };
            }
            Message::Close(_) => return Ok(None),
            Message::Ping(_) | Message::Pong(_) => (),
            Message::Binary(_) => return Err(Error::InvalidMessage),
        }
    }
}

async fn send(socket: &mut WebSocket, message: ControlMessage) -> Result<()> {
    let text = serde_json::to_string(&message).internal()?;
    socket.send(Message::Text(text.into())).await.internal()
}

async fn close(socket: &mut WebSocket, error: Error, limit: Duration) {
    tracing::debug!(%error, "agent connection closed");
    let _ = timeout(limit, async {
        if socket
            .send(Message::Close(Some(CloseFrame {
                code: 1008,
                reason: error.to_string().into(),
            })))
            .await
            .is_err()
        {
            return;
        }
        while let Some(Ok(message)) = socket.recv().await {
            if matches!(message, Message::Close(_)) {
                break;
            }
        }
    })
    .await;
}
