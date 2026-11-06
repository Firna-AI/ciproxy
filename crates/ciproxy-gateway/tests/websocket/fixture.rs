//! Deterministic service inputs with real HTTP routing and persistence.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use ciproxy_gateway::boundary::{AgentEvents, Authorization, Clock, Entropy};
use ciproxy_gateway::connections::{Connections, LocalConnections};
use ciproxy_gateway::error::{Error, Result};
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_gateway::transport::{AgentTransport, WebSocketTransport};
use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::gateway::GatewayStore;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::database::Database;

pub(super) const WORKSPACE: Uuid = Uuid::from_u128(1);
pub(super) const OTHER: Uuid = Uuid::from_u128(2);
pub(super) const ADMIN: Uuid = Uuid::from_u128(7);

pub(super) fn at() -> DateTime<Utc> {
    "2026-11-04T12:00:00Z".parse().unwrap()
}

pub(super) struct FakeClock(pub(super) AtomicI64);

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        at() + chrono::Duration::seconds(self.0.load(Ordering::SeqCst))
    }
}

struct FakeEntropy(AtomicU64);

impl Entropy for FakeEntropy {
    fn identifier(&self) -> Result<Uuid> {
        Ok(Uuid::from_u128(u128::from(
            self.0.fetch_add(1, Ordering::SeqCst),
        )))
    }
    fn secret(&self) -> Result<Secret> {
        Ok(Secret::new(format!(
            "dummy-token-{}",
            self.0.fetch_add(1, Ordering::SeqCst)
        )))
    }
}

struct Authority;

impl Authorization for Authority {
    fn require_admin(&self, principal: Uuid, workspace_id: Uuid) -> Result<()> {
        if principal == ADMIN && workspace_id == WORKSPACE {
            Ok(())
        } else {
            Err(Error::Forbidden)
        }
    }
}

struct Events(UnboundedSender<(Session, AgentMessage)>);

impl AgentEvents for Events {
    fn machine_removed(
        &self,
        _workspace_id: uuid::Uuid,
        _machine_id: uuid::Uuid,
    ) -> ciproxy_gateway::error::Result<()> {
        Ok(())
    }
    fn receive(&self, session: Session, message: AgentMessage) -> Result<()> {
        self.0.send((session, message)).unwrap();
        Ok(())
    }
}

pub(super) struct Fixture {
    pub(super) database: Database,
    pub(super) gateway: Arc<dyn MachineGateway>,
    pub(super) connections: Arc<dyn Connections>,
    pub(super) clock: Arc<FakeClock>,
    pub(super) events: UnboundedReceiver<(Session, AgentMessage)>,
    pub(super) url: String,
    server: JoinHandle<()>,
}

impl Fixture {
    pub(super) async fn start() -> Self {
        let database = Database::new();
        database.workspace(WORKSPACE);
        database.workspace(OTHER);
        let clock = Arc::new(FakeClock(AtomicI64::new(0)));
        let (sender, events) = unbounded_channel();
        let store: Arc<dyn GatewayStore> = database.store.clone();
        let connections: Arc<dyn Connections> = Arc::new(LocalConnections::default());
        let gateway: Arc<dyn MachineGateway> = Arc::new(Gateway::new(
            store,
            Arc::new(Authority),
            clock.clone(),
            Arc::new(FakeEntropy(AtomicU64::new(100))),
            connections.clone(),
            Arc::new(Events(sender)),
        ));
        let transport: &dyn AgentTransport =
            &WebSocketTransport::new(gateway.clone(), connections.clone());
        let router = transport.router();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}/agent/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            database,
            gateway,
            connections,
            clock,
            events,
            url,
            server,
        }
    }

    pub(super) fn advance_to(&self, seconds: i64) {
        self.clock.0.store(seconds, Ordering::SeqCst);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}
