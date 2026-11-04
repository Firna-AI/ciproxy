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

/// A GitHub organization that uses ciproxy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Organization {
    /// The stable internal organization identifier.
    pub id: Uuid,
    /// The GitHub organization identifier.
    pub github_id: i64,
    /// The GitHub organization login.
    pub login: String,
    /// The latest GitHub App installation identifier.
    pub installation_id: i64,
    /// The current App state.
    pub app_state: AppState,
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
    fn organization(&self, organization_id: Uuid) -> Result<Option<Organization>>;
    /// Find retained data from the organization identifier supplied by GitHub.
    fn organization_by_github_id(&self, github_id: i64) -> Result<Option<Organization>>;
    /// Find the organization named by a verified GitHub installation event.
    fn organization_by_installation_id(&self, installation_id: i64)
    -> Result<Option<Organization>>;
    /// Change suspension without revoking credentials or deleting data.
    fn set_app_suspended(&self, organization_id: Uuid, suspended: bool) -> Result<()>;
    /// Record removal, revoke credentials, and erase every Boat key atomically.
    fn record_app_removal(&self, organization_id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Delete removed organizations whose 30-day retention ended.
    fn purge_removed_organizations(&self, at: DateTime<Utc>) -> Result<usize>;
}
