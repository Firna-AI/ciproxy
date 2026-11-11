//! Root task that relays job identities to hosted metadata and facts back to cache.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::cache_metadata::{CacheMetadataAnswer, CacheMetadataWork};
use tokio::task::JoinSet;

use super::administration::MetadataAdministration;
use super::client::TrustedMetadata;
use super::error::Result;

/// Retry delay, injected independently of the root metadata service.
#[async_trait]
pub trait RelayDelay: Send + Sync {
    /// Wait before reconnecting to an unavailable local cache service.
    async fn wait(&self);
}

/// Runtime retry delay constructed at the host composition root.
pub struct TokioRelayDelay;

#[async_trait]
impl RelayDelay for TokioRelayDelay {
    async fn wait(&self) {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Root-owned relay task, cancelled together with its in-flight work.
#[async_trait]
pub trait MetadataRelay: Send + Sync {
    /// Serve queued work until the root task is cancelled.
    async fn run(&self);
}

/// Coordinate local admin and hosted metadata without sharing their credentials.
#[derive(Clone)]
pub struct RootMetadataRelay {
    /// Private cache admin operations, containing no machine credential.
    pub administration: Arc<dyn MetadataAdministration>,
    /// Hosted client kept only in the root process.
    pub hosted: Arc<dyn TrustedMetadata>,
    /// Delay used only when local administration is unavailable.
    pub delay: Arc<dyn RelayDelay>,
}

impl RootMetadataRelay {
    async fn answer(&self, work: CacheMetadataWork) -> Result<()> {
        let metadata = match self.hosted.lookup(work.job_id).await {
            Ok(metadata) => Some(metadata),
            Err(_) => {
                tracing::warn!("trusted cache metadata unavailable; using cache fallback");
                None
            }
        };
        self.administration
            .answer(CacheMetadataAnswer {
                request_id: work.request_id,
                metadata,
            })
            .await
    }
}

#[async_trait]
impl MetadataRelay for RootMetadataRelay {
    async fn run(&self) {
        let mut tasks = JoinSet::new();
        loop {
            if self.administration.enable().await.is_err() {
                self.delay.wait().await;
                continue;
            }
            match self.administration.next().await {
                Ok(Some(work)) => {
                    let relay = self.clone();
                    tasks.spawn(async move {
                        if relay.answer(work).await.is_err() {
                            tracing::warn!("cache metadata response unavailable");
                        }
                    });
                }
                Ok(None) => {}
                Err(_) => self.delay.wait().await,
            }
            while tasks.try_join_next().is_some() {}
        }
    }
}
