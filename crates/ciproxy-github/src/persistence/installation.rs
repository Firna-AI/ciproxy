//! Installation and repository persistence used by the App service.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::github::{GithubInstallationState, Repository};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::{Organization, RepositoryAccess};
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use crate::error::Result;

use super::{StoreAdapter, query};

/// The atomic registration request for a new or retained workspace.
#[derive(Clone, Debug)]
pub struct Registration {
    /// A workspace to create atomically, absent for an existing workspace.
    pub workspace: Option<Workspace>,
    /// The organization's registration.
    pub organization: Organization,
    /// Initial labels, inserted only when the workspace is new.
    pub defaults: Vec<Label>,
    /// The requester saved for first-Admin delivery with a new workspace.
    pub initial_admin: i64,
}

/// Store operations used only by App installation orchestration.
#[cfg_attr(test, unimock::unimock(api = InstallationStoreMock))]
#[async_trait]
pub trait InstallationStore: Send + Sync {
    /// Look up the registration named by a verified installation.
    async fn by_installation(&self, installation_id: i64) -> Result<Option<Organization>>;
    /// Look up current and retained registrations of a GitHub organization.
    async fn by_github_id(&self, github_id: i64) -> Result<Vec<Organization>>;
    /// Persist workspace defaults and registration in one transaction.
    async fn register(&self, registration: Registration) -> Result<()>;
    /// Read an undelivered initial Admin assignment.
    async fn pending_admin(&self, workspace_id: Uuid) -> Result<Option<i64>>;
    /// Confirm delivery to the role service.
    async fn confirm_admin(&self, workspace_id: Uuid) -> Result<()>;
    /// Read retained workspace labels, including edits and deletions.
    async fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>>;
    /// Read the scale sets that already belong to one organization.
    async fn scale_sets(&self, organization: &Organization) -> Result<Vec<ScaleSet>>;
    /// Read a retained managed group across this GitHub account's registrations.
    async fn runner_group(&self, github_id: i64) -> Result<Option<i64>>;
    /// Save current remote identities while preserving the stable internal row.
    async fn save_scale_set(&self, workspace_id: Uuid, scale_set: ScaleSet) -> Result<ScaleSet>;
    /// Atomically replace the repository snapshot.
    async fn save_repositories(
        &self,
        organization: &Organization,
        repositories: Vec<Repository>,
    ) -> Result<()>;
    /// Persist a repository access choice after the group has restricted it.
    async fn save_access(
        &self,
        organization: &Organization,
        access: RepositoryAccess,
    ) -> Result<()>;
    /// Atomically save verified login and state without reviving removed access.
    async fn save_snapshot(
        &self,
        organization: &Organization,
        snapshot: GithubInstallationState,
    ) -> Result<Organization>;
    /// Record removal, credential revocation, and retained-data deadlines.
    async fn remove(&self, organization: &Organization, at: DateTime<Utc>) -> Result<()>;
    /// Report whether the workspace has any surviving installed organization.
    async fn has_organizations(&self, workspace_id: Uuid) -> Result<bool>;
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
    async fn register(&self, registration: Registration) -> Result<()> {
        query(self.store.clone(), move |store| {
            store.register_github_installation(
                registration.workspace.as_ref(),
                &registration.organization,
                &registration.defaults,
                registration.initial_admin,
            )
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
            store.scale_sets(organization.workspace_id, organization.id)
        })
        .await
    }
    async fn runner_group(&self, github_id: i64) -> Result<Option<i64>> {
        query(self.store.clone(), move |store| {
            store.github_runner_group(github_id)
        })
        .await
    }
    async fn save_scale_set(&self, workspace_id: Uuid, scale_set: ScaleSet) -> Result<ScaleSet> {
        query(self.store.clone(), move |store| {
            store.save_github_scale_set(workspace_id, &scale_set)
        })
        .await
    }
    async fn save_repositories(
        &self,
        organization: &Organization,
        repositories: Vec<Repository>,
    ) -> Result<()> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.replace_repositories(organization.workspace_id, organization.id, &repositories)
        })
        .await
    }
    async fn save_access(
        &self,
        organization: &Organization,
        access: RepositoryAccess,
    ) -> Result<()> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.save_repository_access(organization.workspace_id, organization.id, &access)
        })
        .await
    }
    async fn save_snapshot(
        &self,
        organization: &Organization,
        snapshot: GithubInstallationState,
    ) -> Result<Organization> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.save_github_installation_state(
                organization.workspace_id,
                organization.id,
                &snapshot,
            )
        })
        .await
    }
    async fn remove(&self, organization: &Organization, at: DateTime<Utc>) -> Result<()> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.record_app_removal(organization.workspace_id, organization.id, at)
        })
        .await
    }
    async fn has_organizations(&self, workspace_id: Uuid) -> Result<bool> {
        query(self.store.clone(), move |store| {
            Ok(store
                .organizations(workspace_id)?
                .iter()
                .any(|organization| organization.removed_at.is_none()))
        })
        .await
    }
}
