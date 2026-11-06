//! Fixed gateway collaborators with the real PostgreSQL adapter.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use ciproxy_gateway::boundary::{AgentEvents, Authorization, Clock, Entropy};
use ciproxy_gateway::connections::Connections;
use ciproxy_gateway::error::Result;
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_gateway::types::Session;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::gateway::GatewayStore;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use uuid::Uuid;

use super::database::Database;

pub(super) const WORKSPACE: Uuid = Uuid::from_u128(1);
pub(super) const INSTALL: &str = "dummy-trace-install-secret";
pub(super) const MACHINE: &str = "dummy-trace-machine-credential";

struct Authorized;
impl Authorization for Authorized {
    fn require_admin(&self, _principal: Uuid, _workspace: Uuid) -> Result<()> {
        Ok(())
    }
}

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        "2026-11-04T12:00:00Z".parse().unwrap()
    }
}

struct FixedEntropy {
    identities: AtomicU64,
    secrets: AtomicU64,
}
impl Entropy for FixedEntropy {
    fn identifier(&self) -> Result<Uuid> {
        Ok(Uuid::from_u128(u128::from(
            self.identities.fetch_add(1, Ordering::SeqCst),
        )))
    }
    fn secret(&self) -> Result<Secret> {
        Ok(Secret::new(
            if self.secrets.fetch_add(1, Ordering::SeqCst) == 0 {
                INSTALL
            } else {
                MACHINE
            },
        ))
    }
}

struct Events;
impl AgentEvents for Events {
    fn machine_removed(
        &self,
        _workspace_id: uuid::Uuid,
        _machine_id: uuid::Uuid,
    ) -> ciproxy_gateway::error::Result<()> {
        Ok(())
    }
    fn receive(&self, _session: Session, _message: AgentMessage) -> Result<()> {
        Ok(())
    }
}

pub(super) fn gateway(
    database: &Database,
    connections: Arc<dyn Connections>,
) -> Arc<dyn MachineGateway> {
    let store: Arc<dyn GatewayStore> = PgStoreFactory.connect(&database.url).unwrap();
    Arc::new(Gateway::new(
        store,
        Arc::new(Authorized),
        Arc::new(FixedClock),
        Arc::new(FixedEntropy {
            identities: AtomicU64::new(100),
            secrets: AtomicU64::new(0),
        }),
        connections,
        Arc::new(Events),
    ))
}
