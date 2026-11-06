//! GitHub App installation, requester linkage, and lifecycle orchestration.

mod coordination;
mod delivery;
mod installation;
mod lifecycle;
mod reconcile;
pub mod webhook;

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::organizations::{Organization, RepositoryAccess};
use uuid::Uuid;

use crate::error::Result;
use crate::installation_state::InstallationApi;
use crate::persistence::installation::InstallationStore;
use crate::persistence::lifecycle::LifecycleStore;
use crate::persistence::mutation::OrganizationMutations;
use crate::rest::GitHubRest;
use crate::runtime::Runtime;
use crate::scale_sets::ScaleSetFactory;

use self::webhook::InstallationEvent;

/// The role service's persisted installation-intent and first-Admin boundary.
#[cfg_attr(test, unimock::unimock(api = InstallAuthorityMock))]
#[async_trait]
pub trait InstallAuthority: Send + Sync {
    /// Resolve the initiating Admin, including requests an owner approves later.
    ///
    /// The implementation persists intent by requester. It checks the Admin
    /// role and the Free organization limit before it returns a workspace.
    async fn intended_workspace(
        &self,
        requester_id: i64,
        github_organization_id: i64,
    ) -> Result<Option<Uuid>>;
    /// Idempotently grant the first installer the Admin role in a new workspace.
    async fn ensure_initial_admin(&self, workspace_id: Uuid, github_user_id: i64) -> Result<()>;
}

/// The scaler, machine gateway, and billing services' lifecycle delivery seam.
#[cfg_attr(test, unimock::unimock(api = LifecycleOutputMock))]
#[async_trait]
pub trait LifecycleOutput: Send + Sync {
    /// Apply the documented event idempotently, preserving all running jobs.
    async fn changed(&self, event: LifecycleEvent) -> Result<()>;
}

/// The result of an authenticated installation event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstallationOutcome {
    /// The organization is registered in the named workspace.
    Organization(Organization),
    /// A personal account receives the organizations-only setup outcome.
    OrganizationsOnly {
        /// The installation, for the setup page to identify it safely.
        installation_id: i64,
        /// The GitHub settings page where the user can remove the App.
        remove_url: String,
    },
    /// The event belongs to a registration ciproxy no longer retains.
    Ignored,
}

/// The complete App integration entry point used by the hosted control plane.
#[unimock::unimock(api=GitHubAppMock)]
#[async_trait]
pub trait GitHubApp: Send + Sync {
    /// Discover organizations that need lifecycle replay after a process restart.
    async fn pending_lifecycle_organizations(&self) -> Result<Vec<i64>>;
    /// Replay one organization's immutable events under its mutation lease.
    async fn replay_lifecycle(&self, github_id: i64) -> Result<()>;
    /// Handle a verified installation lifecycle event.
    async fn installation(&self, event: InstallationEvent) -> Result<InstallationOutcome>;
    /// Refresh visibility and branches after repository/installation changes.
    async fn refresh_repositories(&self, installation_id: i64) -> Result<()>;
    /// Apply an Admin's allowed-repository choice through the isolated group.
    async fn set_repository_access(
        &self,
        installation_id: i64,
        access: RepositoryAccess,
    ) -> Result<()>;
}

/// The App service, consumed through dyn GitHubApp at the composition root.
pub struct AppService {
    /// Waiting cross-process mutation authority for GitHub organizations.
    pub leases: Arc<dyn OrganizationMutations>,
    /// App-authenticated current installation identity and suspension state.
    pub installations: Arc<dyn InstallationApi>,
    /// Installation persistence through blocking workers.
    pub store: Arc<dyn InstallationStore>,
    /// Transactional lifecycle writes and ordered durable notification delivery.
    pub outbox: Arc<dyn LifecycleStore>,
    /// The three-permission REST client.
    pub rest: Arc<dyn GitHubRest>,
    /// Organization-scoped scale-set APIs.
    pub scale_sets: Arc<dyn ScaleSetFactory>,
    /// The later sign-in service's intent and role adapter.
    pub authority: Arc<dyn InstallAuthority>,
    /// Lifecycle consumers from other milestones.
    pub lifecycle: Arc<dyn LifecycleOutput>,
    /// Clock and identifier inputs.
    pub runtime: Arc<dyn Runtime>,
}

#[async_trait]
impl GitHubApp for AppService {
    async fn pending_lifecycle_organizations(&self) -> Result<Vec<i64>> {
        self.outbox.organizations().await
    }
    async fn replay_lifecycle(&self, github_id: i64) -> Result<()> {
        self.replay_pending_lifecycle(github_id).await
    }
    async fn installation(&self, event: InstallationEvent) -> Result<InstallationOutcome> {
        self.handle_installation(event).await
    }
    async fn refresh_repositories(&self, installation_id: i64) -> Result<()> {
        self.refresh(installation_id).await
    }
    async fn set_repository_access(
        &self,
        installation_id: i64,
        access: RepositoryAccess,
    ) -> Result<()> {
        self.change_access(installation_id, access).await
    }
}
