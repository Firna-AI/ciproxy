//! Axum WebSocket adapter for outbound agent connections at /agent/.

use std::sync::Arc;

use axum::Router;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::routing::get;
use ciproxy_protocol::agent::{AgentMessage, ControlMessage};

use crate::connections::{Connections, Outbox};
use crate::error::{Error, ErrorResultExt, Result};
use crate::gateway::MachineGateway;
use crate::types::Session;

/// The HTTP composition boundary, mounted behind the control plane's TLS ingress.
pub trait AgentTransport: Send + Sync {
    /// Return the agent route for merging into the control-plane router.
    fn router(&self) -> Router;
}

/// WebSocket transport using the same gateway and mailboxes as command callers.
#[derive(Clone)]
pub struct WebSocketTransport {
    gateway: Arc<dyn MachineGateway>,
    connections: Arc<dyn Connections>,
}

impl WebSocketTransport {
    /// Construct at the HTTP composition root, then consume through dyn AgentTransport.
    pub fn new(gateway: Arc<dyn MachineGateway>, connections: Arc<dyn Connections>) -> Self {
        Self {
            gateway,
            connections,
        }
    }

    async fn run(self, mut socket: WebSocket) {
        let result = self.authenticate(&mut socket).await;
        let (session, outbox) = match result {
            Ok(Some(active)) => active,
            Ok(None) => return,
            Err(error) => {
                close(&mut socket, error).await;
                return;
            }
        };
        let result = self.serve_session(&mut socket, session, outbox).await;
        if let Err(error) = self.connections.detach(session) {
            tracing::warn!(%error, "agent connection cleanup failed");
        }
        if let Err(error) = result {
            close(&mut socket, error).await;
        }
    }

    async fn authenticate(
        &self,
        socket: &mut WebSocket,
    ) -> Result<Option<(Session, Box<dyn Outbox>)>> {
        let Some(message) = receive(socket).await? else {
            return Ok(None);
        };
        let gateway = self.gateway.clone();
        let handshake = tokio::task::spawn_blocking(move || gateway.open(message))
            .await
            .internal()??;
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
            if let Err(error) = send(socket, reply).await {
                self.connections.detach(handshake.session)?;
                return Err(error);
            }
        }
        Ok(Some((handshake.session, handshake.outbox)))
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
                async move { upgrade.on_upgrade(move |socket| transport.run(socket)) }
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

async fn close(socket: &mut WebSocket, error: Error) {
    tracing::debug!(%error, "agent connection closed");
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: 1008,
            reason: error.to_string().into(),
        })))
        .await;
}
