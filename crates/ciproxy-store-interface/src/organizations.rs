//! Organization installation records and data-retention queries.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;

/// The GitHub App state for an organization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum AppState {
    /// GitHub permits normal App access.
    Active,
    /// GitHub temporarily suspended App access.
    Suspended,
    /// GitHub removed the App.
    Removed,
}

/// The private repositories of one organization that may use ciproxy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepositoryAccess {
    /// All visible private repositories, including future repositories.
    AllPrivate,
    /// Only the named GitHub repository identifiers.
    Selected(Vec<i64>),
}

/// A current or retained GitHub registration in one workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Organization {
    /// The registration identifier, preserved with this workspace's history.
    pub id: Uuid,
    /// The workspace that owns this current or retained registration.
    pub workspace_id: Uuid,
    /// The GitHub organization identifier.
    pub github_id: i64,
    /// The GitHub organization login.
    pub login: String,
    /// The latest GitHub App installation identifier.
    pub installation_id: i64,
    /// The current App state.
    pub app_state: AppState,
    /// The organization's allowed private repositories.
    pub repositories: RepositoryAccess,
    /// When GitHub removed the App.
    pub removed_at: Option<DateTime<Utc>>,
    /// When retained data becomes eligible for deletion.
    pub delete_after: Option<DateTime<Utc>>,
}

/// Queries for organization records and removal bookkeeping.
pub trait OrganizationStore: Send + Sync {
    /// Insert or reinstall an organization while preserving retained data.
    fn install_organization(&self, organization: &Organization) -> Result<()>;
    /// Read an organization by its internal identifier.
    fn organization(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Option<Organization>>;
    /// Read current and retained registrations owned by one workspace.
    fn organizations(&self, workspace_id: Uuid) -> Result<Vec<Organization>>;
    /// Read current and retained registrations for a trusted GitHub identity.
    ///
    /// The caller selects the workspace according to the install intent.
    fn organizations_by_github_id(&self, github_id: i64) -> Result<Vec<Organization>>;
    /// Find the organization named by a verified GitHub installation event.
    fn organization_by_installation_id(&self, installation_id: i64)
    -> Result<Option<Organization>>;
    /// Change suspension without revoking credentials or deleting data.
    fn set_app_suspended(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        suspended: bool,
    ) -> Result<()>;
    /// Replace repository access for one organization in its workspace.
    fn save_repository_access(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        repositories: &RepositoryAccess,
    ) -> Result<()>;
    /// Record App removal while preserving live sandbox cleanup access.
    ///
    /// Shared execution credentials stay while another organization remains.
    /// The last removal revokes execution access and starts workspace retention.
    /// Repeated removal keeps the original retention deadline.
    fn record_app_removal(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Delete expired organization data without deleting shared workspace data.
    fn purge_removed_organizations(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<usize>;
}
