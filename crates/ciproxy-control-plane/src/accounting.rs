//! Coalesced accounting passes run outside every runner operation and lease.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_github::runtime::Runtime;
use ciproxy_github::timing::JobTiming;
use ciproxy_metering::service::Metering;
use ciproxy_store_interface::store::Store;
use tokio::sync::{Mutex, Notify, watch};
use uuid::Uuid;

use crate::tasks::OwnedTask;

/// Supervised accounting interface with one coalesced pending request.
#[async_trait]
pub trait Accounting: Send + Sync {
    /// Request a pass without waiting for accounting or timing lookups.
    fn request(&self);
    /// Run requested passes until cancellation, with no additional timer.
    async fn run(&self, stop: watch::Receiver<bool>);
    /// Complete one isolated pass for callers that need an accounting boundary.
    async fn pass(&self);
}

/// Timing, metering, and retention dependencies of the hosted accounting worker.
#[derive(Clone)]
pub struct AccountingWorker {
    store: Arc<dyn Store>,
    timing: Arc<dyn JobTiming>,
    metering: Arc<dyn Metering>,
    clock: Arc<dyn Runtime>,
    requested: Arc<Notify>,
    /// This lock serializes accounting passes and has no execution callers.
    pass_lock: Arc<Mutex<()>>,
}

impl AccountingWorker {
    /// Build accounting with the same durable facts and clock as execution.
    pub fn new(
        store: Arc<dyn Store>,
        timing: Arc<dyn JobTiming>,
        metering: Arc<dyn Metering>,
        clock: Arc<dyn Runtime>,
    ) -> Self {
        Self {
            store,
            timing,
            metering,
            clock,
            requested: Arc::new(Notify::new()),
            pass_lock: Arc::new(Mutex::new(())),
        }
    }

    async fn workspace(&self, workspace_id: Uuid) {
        if let Err(error) = self.timing.repair(workspace_id).await {
            tracing::warn!(%workspace_id,%error,"job timing recovery will retry");
        }
        let metering = self.metering.clone();
        let at = self.clock.now();
        let refreshed =
            tokio::task::spawn_blocking(move || metering.refresh(workspace_id, at)).await;
        match refreshed {
            Ok(Ok(outcome)) if outcome.stale_jobs == 0 && outcome.stale_sandboxes == 0 => {
                let store = self.store.clone();
                match tokio::task::spawn_blocking(move || store.purge_github_jobs(workspace_id, at))
                    .await
                {
                    Ok(Ok(_)) => {}
                    result => {
                        tracing::warn!(%workspace_id,?result,"accounted raw job retention will retry")
                    }
                }
            }
            Ok(Ok(_)) => {}
            Ok(Err(error)) => tracing::warn!(%workspace_id,%error,"metering accounting will retry"),
            Err(error) => {
                tracing::warn!(%workspace_id,%error,"metering worker failed; accounting will retry")
            }
        }
    }
}

#[async_trait]
impl Accounting for AccountingWorker {
    fn request(&self) {
        self.requested.notify_one();
    }

    async fn run(&self, mut stop: watch::Receiver<bool>) {
        loop {
            if *stop.borrow() {
                return;
            }
            tokio::select! {
                _ = self.requested.notified() => {},
                _ = stop.changed() => return,
            }
            tokio::select! {
                _ = self.pass() => {},
                _ = stop.changed() => return,
            }
        }
    }

    async fn pass(&self) {
        let _pass = self.pass_lock.lock().await;
        let store = self.store.clone();
        let workspaces = match tokio::task::spawn_blocking(move || store.routing_workspaces()).await
        {
            Ok(Ok(workspaces)) => workspaces,
            result => {
                tracing::warn!(?result, "accounting workspace discovery will retry");
                return;
            }
        };
        for workspace_id in workspaces {
            let worker = self.clone();
            let mut task = OwnedTask(tokio::spawn(
                async move { worker.workspace(workspace_id).await },
            ));
            if let Err(error) = (&mut task.0).await {
                tracing::warn!(%workspace_id,%error,"accounting workspace failed; independent workspaces continue");
            }
        }
    }
}
