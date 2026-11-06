//! Installation and repository persistence used by the App service.

use async_trait::async_trait;
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::scale_sets::ScaleSet;
use uuid::Uuid;

use crate::error::Result;

use super::{StoreAdapter, query};

/// Store operations used only by App installation orchestration.
#[cfg_attr(test, unimock::unimock(api = InstallationStoreMock))]
#[async_trait]
pub trait InstallationStore: Send + Sync {
    /// Look up the registration named by a verified installation.
    async fn by_installation(&self, installation_id: i64) -> Result<Option<Organization>>;
    /// Look up current and retained registrations of a GitHub organization.
    async fn by_github_id(&self, github_id: i64) -> Result<Vec<Organization>>;
    /// Read an undelivered initial Admin assignment.
    async fn pending_admin(&self, workspace_id: Uuid) -> Result<Option<i64>>;
    /// Confirm delivery to the role service.
    async fn confirm_admin(&self, workspace_id: Uuid) -> Result<()>;
    /// Read retained workspace labels, including edits and deletions.
    async fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>>;
    /// Read active and retired bindings until their remote cleanup is confirmed.
    async fn scale_sets(&self, organization: &Organization) -> Result<Vec<ScaleSet>>;
    /// Read a retained managed group across this GitHub account's registrations.
    async fn runner_group(&self, github_id: i64) -> Result<Option<i64>>;
}

#[async_trait]
impl InstallationStore for StoreAdapter {
    async fn by_installation(&self, installation_id: i64) -> Result<Option<Organization>> {
        query(self.store.clone(), move |store| {
            store.organization_by_installation_id(installation_id)
        })
        .await
    }
    async fn by_github_id(&self, github_id: i64) -> Result<Vec<Organization>> {
        query(self.store.clone(), move |store| {
            store.organizations_by_github_id(github_id)
        })
        .await
    }
    async fn pending_admin(&self, workspace_id: Uuid) -> Result<Option<i64>> {
        query(self.store.clone(), move |store| {
            store.pending_initial_admin(workspace_id)
        })
        .await
    }
    async fn confirm_admin(&self, workspace_id: Uuid) -> Result<()> {
        query(self.store.clone(), move |store| {
            store.confirm_initial_admin(workspace_id)
        })
        .await
    }
    async fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>> {
        query(self.store.clone(), move |store| store.labels(workspace_id)).await
    }
    async fn scale_sets(&self, organization: &Organization) -> Result<Vec<ScaleSet>> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.dashboard_scale_sets(organization.workspace_id, organization.id)
        })
        .await
    }
    async fn runner_group(&self, github_id: i64) -> Result<Option<i64>> {
        query(self.store.clone(), move |store| {
            store.github_runner_group(github_id)
        })
        .await
    }
}
