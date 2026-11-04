//! Locked scale-set listening with durable-before-ACK delivery.

mod messages;
mod run;

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::github::GithubJob;
use tokio::sync::watch;

use crate::error::Result;
use crate::persistence::session::{ListenerTarget, SessionStore};
use crate::rest::GitHubRest;
use crate::scale_sets::ScaleSetFactory;

/// Demand and lifecycle notifications for the later snapshot/scaler service.
#[cfg_attr(test, unimock::unimock(api = JobOutputMock))]
#[async_trait]
pub trait JobOutput: Send + Sync {
    /// Accept the saved metadata idempotently by organization and job UUID.
    ///
    /// RunFacts are raw API data. This boundary grants no cache trust and
    /// does not turn head_branch or workflow_ref into a full Git ref.
    async fn changed(&self, job: GithubJob) -> Result<()>;
}

/// Options for a continuous listener or an operator's bounded live probe.
#[derive(Clone, Debug)]
pub struct ListenerOptions {
    /// A process-unique session owner name.
    pub owner: String,
    /// The maximum capacity advertised to the GitHub scale-set service.
    pub capacity: u32,
    /// End after this many polls, or keep listening when absent.
    pub max_polls: Option<usize>,
}

/// The runtime entry point for a single organization-label listener.
#[async_trait]
pub trait ScaleSetListener: Send + Sync {
    /// Hold the store lock until the GitHub session has closed.
    async fn run(
        &self,
        target: ListenerTarget,
        options: ListenerOptions,
        stop: watch::Receiver<bool>,
    ) -> Result<()>;
}

/// One listener service, consumed through dyn ScaleSetListener.
pub struct DurableListener {
    /// Blocking-worker persistence and session leases.
    pub store: Arc<dyn SessionStore>,
    /// The three-permission REST boundary for trusted run facts.
    pub rest: Arc<dyn GitHubRest>,
    /// The per-organization scale-set API factory.
    pub scale_sets: Arc<dyn ScaleSetFactory>,
    /// Durable demand delivery for the snapshot/scaler service.
    pub output: Arc<dyn JobOutput>,
}

#[async_trait]
impl ScaleSetListener for DurableListener {
    async fn run(
        &self,
        target: ListenerTarget,
        options: ListenerOptions,
        stop: watch::Receiver<bool>,
    ) -> Result<()> {
        self.listen(target, options, stop).await
    }
}
