//! Replay changes proxy credentials without repeating pre-action disk setup.

use async_trait::async_trait;
use ciproxy_protocol::cache::CacheConnection;
use ciproxy_protocol::guest::JobPreparation;

use crate::handoff::PreparedProxy;

#[async_trait]
pub(crate) trait PreparedJob: Send {
    async fn replay(&mut self, connection: Option<CacheConnection>) -> JobPreparation;
}

pub(crate) struct Prepared {
    pub(crate) initial: PreparedProxy,
    pub(crate) connection: Option<CacheConnection>,
}

#[async_trait]
impl PreparedJob for Prepared {
    async fn replay(&mut self, connection: Option<CacheConnection>) -> JobPreparation {
        if let Some(proxy) = &mut self.initial.proxy
            && let Some(connection) = connection
            && self.connection.as_ref() != Some(&connection)
        {
            match proxy.replace_connection(connection.clone()).await {
                Ok(()) => self.connection = Some(connection),
                Err(error) => {
                    tracing::warn!(%error, "VM proxy session update will retry; preserve results passthrough")
                }
            }
        }
        self.initial.outcome
    }
}

#[cfg(test)]
#[path = "_tests_/proxy_recovery_tests.rs"]
mod proxy_recovery_tests;
