//! Restart recovery for memberships, sign-in expiry, labels, and take-outs.

use async_trait::async_trait;
use ciproxy_store_interface::dashboard::records::Takeout;

use crate::error::{Result, ServiceResultExt};
use crate::handlers::{labels, organizations};
use crate::state::{ApiState, query};

/// Background work driven by the control-plane owner at least hourly.
#[async_trait]
pub trait ApiMaintenance: Send + Sync {
    /// Reconcile due memberships, authentication retention, and pending take-outs.
    async fn reconcile(&self) -> Result<()>;
}

/// Production adapter sharing the same API collaborators as the HTTP router.
pub struct Maintenance {
    /// The already constructed runtime API state.
    pub state: ApiState,
}

#[async_trait]
impl ApiMaintenance for Maintenance {
    async fn reconcile(&self) -> Result<()> {
        let mut failure = self.state.access.reconcile().await.err();
        let pending = query(self.state.store.clone(), |store| {
            store.pending_organization_takeouts()
        })
        .await?;
        for removal in pending {
            if let Err(error) = self.retry_takeout(removal).await {
                failure = Some(error);
            }
        }
        let workspaces = query(self.state.store.clone(), |store| {
            store.retired_label_workspaces()
        })
        .await?;
        for workspace in workspaces {
            let retired = query(self.state.store.clone(), move |store| {
                store.retired_dashboard_labels(workspace)
            })
            .await?;
            for label in retired {
                if let Err(error) = labels::cleanup_label(&self.state, workspace, label.id).await {
                    failure = Some(error);
                }
            }
        }
        let now = self.state.runtime.now();
        query(self.state.store.clone(), move |store| {
            store.expire_dashboard_auth(now)
        })
        .await?;
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(())
    }
}

impl Maintenance {
    /// Keep each exact installation's retry independent of every other organization.
    async fn retry_takeout(&self, removal: Takeout) -> Result<()> {
        let workspace = removal.workspace_id;
        let organization = removal.organization_id;
        let current = query(self.state.store.clone(), move |store| {
            store.organization(workspace, organization)
        })
        .await?;
        let Some(current) = current else {
            return Ok(());
        };
        let id = current.github_id;
        self.state.github.replay_lifecycle(id).await.api()?;
        let _lease = query(self.state.store.clone(), move |store| {
            store.lock_github_organization(id)
        })
        .await?;
        organizations::finish_takeout(&self.state, &removal).await
    }
}
