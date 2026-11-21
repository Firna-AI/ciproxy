//! Workspace ownership and explicit credential revocation.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;

/// The customer tenant shared by one or more GitHub organizations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    /// The stable workspace identifier.
    pub id: Uuid,
    /// The customer-visible workspace name.
    pub name: String,
    /// The deletion deadline after the last organization's App removal.
    pub delete_after: Option<DateTime<Utc>>,
}

/// Queries for the shared customer tenant.
pub trait WorkspaceStore: Send + Sync {
    /// Create a workspace with default settings and no pending deletion.
    fn create_workspace(&self, workspace: &Workspace) -> Result<()>;
    /// Read one workspace by its identifier.
    fn workspace(&self, workspace_id: Uuid) -> Result<Option<Workspace>>;
    /// Revoke execution access while preserving live sandbox cleanup access.
    ///
    /// Removing the last organization also performs these mutations.
    /// Retired Boat keys and sandbox-end tokens stay until their runners end.
    /// This operation never deletes retained workspace configuration.
    fn revoke_workspace_execution_access(
        &self,
        workspace_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Delete one empty workspace when its saved retention deadline has ended.
    ///
    /// Return false before the deadline, after a reinstall, or when absent.
    fn purge_expired_workspace(&self, workspace_id: Uuid, at: DateTime<Utc>) -> Result<bool>;
}
