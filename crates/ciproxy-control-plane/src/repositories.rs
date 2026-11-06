//! Refresh retained installation authority and repositories without extra events.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_github::app::GitHubApp;
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::store::Store;

use crate::error::{ErrorResultExt, Result};

#[cfg_attr(test,unimock::unimock(api=InstallationsMock))]
#[async_trait]
pub(crate) trait Installations: Send + Sync {
    async fn retained(&self) -> Result<Vec<i64>>;
}

pub(crate) struct StoredInstallations {
    pub(crate) store: Arc<dyn Store>,
}

#[async_trait]
impl Installations for StoredInstallations {
    async fn retained(&self) -> Result<Vec<i64>> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            let mut ids = Vec::new();
            for workspace in store.routing_workspaces().internal()? {
                ids.extend(
                    store
                        .organizations(workspace)
                        .internal()?
                        .into_iter()
                        .filter(|organization| organization.app_state != AppState::Removed)
                        .map(|organization| organization.installation_id),
                );
            }
            Ok(ids)
        })
        .await
        .internal()?
    }
}

#[async_trait]
pub(crate) trait RepositoryRefresh: Send + Sync {
    async fn refresh(&self) -> Result<()>;
}

pub(crate) struct RefreshRepositories {
    pub(crate) installations: Arc<dyn Installations>,
    pub(crate) app: Arc<dyn GitHubApp>,
}

#[async_trait]
impl RepositoryRefresh for RefreshRepositories {
    async fn refresh(&self) -> Result<()> {
        for installation_id in self.installations.retained().await? {
            if let Err(error) = self.app.refresh_repositories(installation_id).await {
                tracing::warn!(%installation_id,%error,"installation metadata refresh will retry; other organizations continue");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/repository_refresh_tests.rs"]
mod repository_refresh_tests;
