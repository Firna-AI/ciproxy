//! GitHub App installation, requester linkage, and lifecycle orchestration.

mod coordination;
mod delivery;
mod installation;
mod lifecycle;
mod linking;
mod reconcile;
pub mod webhook;

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::installation_links::InstallationClaim;
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

/// Exact association decision before allocating a workspace or runner resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallationDecision {
    /// An unambiguous normal install uses existing first-install/retention rules.
    Direct,
    /// Fresh installed proof and a trusted choice name this exact workspace.
    Target {
        workspace_id: Uuid,
        admin_approved: bool,
        billing_hold: bool,
    },
    /// The original requester must complete proof and choice.
    AwaitingAssignment,
}

/// The role service's exact installation-claim and first-Admin boundary.
#[cfg_attr(test, unimock::unimock(api = InstallAuthorityMock))]
#[async_trait]
pub trait InstallAuthority: Send + Sync {
    /// Project canonical signed flow and resolve only its Target admission.
    ///
    /// The store has joined the signed creation identity and original person's flow.
    /// An unresolved Add waits for the person's own installed proof and workspace choice.
    /// A chosen workspace can retain independent Admin and billing holds.
    async fn installation_decision(
        &self,
        claim: &InstallationClaim,
    ) -> Result<InstallationDecision>;
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
    /// The installation is durable but has no verified workspace choice yet.
    AwaitingAssignment {
        /// Requester-owned recovery identity.
        claim_id: Uuid,
    },
}

/// The complete App integration entry point used by the hosted control plane.
#[unimock::unimock(api=GitHubAppMock)]
#[async_trait]
pub trait GitHubApp: Send + Sync {
    /// Resolve one saved exact requester claim without inferring a target.
    async fn resolve_installation_link(&self, claim_id: Uuid) -> Result<InstallationOutcome>;
    /// Retry billing-eligible activation while preserving explicit Admin holds.
    async fn reconcile_organization_activation(&self, workspace_id: Uuid) -> Result<()>;
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
    /// The sign-in service's exact claim, local admission, and role adapter.
    pub authority: Arc<dyn InstallAuthority>,
    /// Lifecycle consumers from other milestones.
    pub lifecycle: Arc<dyn LifecycleOutput>,
    /// Clock and identifier inputs.
    pub runtime: Arc<dyn Runtime>,
}

#[async_trait]
impl GitHubApp for AppService {
    async fn resolve_installation_link(&self, id: Uuid) -> Result<InstallationOutcome> {
        self.resolve_link(id).await
    }
    async fn reconcile_organization_activation(&self, workspace: Uuid) -> Result<()> {
        self.activate_workspace(workspace).await
    }
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
