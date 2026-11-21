//! Controlled clock, entropy, and gateway events for real-store HTTP fixtures.

use std::sync::Mutex;

use chrono::{DateTime, Utc};
use ciproxy_api::error::Result;
use ciproxy_api::runtime::Runtime;
use ciproxy_gateway::boundary::AgentEvents;
use ciproxy_protocol::agent::AgentMessage;
use secrecy::SecretString;
use uuid::Uuid;

pub(super) struct Inputs(pub(super) Mutex<DateTime<Utc>>);
impl Inputs {
    pub(super) fn advance(&self, by: chrono::Duration) {
        *self.0.lock().unwrap() += by;
    }
}
impl Runtime for Inputs {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap()
    }
    fn identifier(&self) -> Uuid {
        Uuid::new_v4()
    }
    fn secret(&self) -> Result<SecretString> {
        Ok(Uuid::new_v4().simple().to_string().repeat(2).into())
    }
}
impl ciproxy_gateway::boundary::Clock for Inputs {
    fn now(&self) -> DateTime<Utc> {
        Runtime::now(self)
    }
}
impl ciproxy_github::runtime::Runtime for Inputs {
    fn now(&self) -> DateTime<Utc> {
        Runtime::now(self)
    }
    fn uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}

pub(super) struct Events;
impl AgentEvents for Events {
    fn machine_removed(&self, _: Uuid, _: Uuid) -> ciproxy_gateway::error::Result<()> {
        Ok(())
    }
    fn receive(
        &self,
        _: ciproxy_gateway::types::Session,
        _: AgentMessage,
    ) -> ciproxy_gateway::error::Result<()> {
        Ok(())
    }
}

/// Controlled release service when a scenario does not need the HTTP adapter.
pub(super) struct Releases;
#[async_trait::async_trait]
impl ciproxy_api::agent_releases::AgentReleases for Releases {
    async fn latest(&self) -> Result<String> {
        Ok("0.1.0".to_owned())
    }
}
