//! GitHub App installation, requester linkage, and lifecycle orchestration.

mod coordination;
mod installation;
mod lifecycle;
mod reconcile;
pub mod webhook;

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::organizations::{Organization, RepositoryAccess};
use uuid::Uuid;

use crate::error::Result;
use crate::installation_state::InstallationApi;
use crate::persistence::installation::InstallationStore;
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

/// An organization lifecycle change, delivered after the stored state changes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleEvent {
    /// Resume listeners and allow new runner starts.
    Active(Organization),
    /// Stop idle runners and listeners while allowing running jobs to finish.
    Suspended(Organization),
    /// Stop idle runners and delete idle sandboxes without calling GitHub.
    Removed {
        /// The removed organization's retained registration.
        organization: Organization,
        /// Revoke machine connections and end billing when this is true.
        /// Retired Boat credentials remain only for running sandbox cleanup.
        last_organization: bool,
    },
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
#[async_trait]
pub trait GitHubApp: Send + Sync {
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
