//! Scale set records and session ownership.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;

/// A GitHub scale set for one organization label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaleSet {
    /// The internal scale set identifier.
    pub id: Uuid,
    /// The GitHub organization served by the scale set.
    pub organization_id: Uuid,
    /// The shared workspace label served for this organization.
    pub label_id: Uuid,
    /// The GitHub scale set identifier.
    pub github_id: i64,
    /// The GitHub runner group that restricts repository access.
    pub runner_group_id: i64,
}

/// Routing and remote cleanup state of one durable binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaleSetState {
    /// The label can acquire new work and register runners.
    Active,
    /// The deleted label retains its remote resource for completion tracking.
    Draining,
    /// Remote deletion succeeded and its confirmation is durable.
    Removed,
}

/// Exact binding context, including retained completion authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaleSetBinding {
    /// Stable organization, label, and remote identities.
    pub scale_set: ScaleSet,
    /// The binding's durable routing and cleanup state.
    pub state: ScaleSetState,
    /// Assigned or started work, or an unconfirmed job notification, remains.
    pub has_tracked_jobs: bool,
}

/// An owned database session that releases its advisory lock on destruction.
pub trait ScaleSetSession: Send {
    /// Check that the database session still owns its lock.
    ///
    /// A listener must stop its GitHub session after this query fails.
    fn check_connection(&mut self) -> Result<()>;
}

/// Queries and exclusive session locks for scale sets.
pub trait ScaleSetStore: Send + Sync {
    /// Insert a scale set for an organization label.
    fn create_scale_set(&self, workspace_id: Uuid, scale_set: &ScaleSet) -> Result<()>;
    /// Read all scale sets of one organization.
    fn scale_sets(&self, workspace_id: Uuid, organization_id: Uuid) -> Result<Vec<ScaleSet>>;
    /// Resolve one exact binding even after retirement and remote cleanup.
    fn scale_set_binding(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        scale_set_id: Uuid,
    ) -> Result<Option<ScaleSetBinding>>;
    /// Discover active listeners and retired bindings with tracked work after restart.
    fn listener_scale_sets(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<ScaleSetBinding>>;
    /// Confirm remote deletion of only this exact saved binding and remote ID.
    fn complete_scale_set_removal(
        &self,
        workspace_id: Uuid,
        scale_set: &ScaleSet,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Try to own a session, returning None when another session owns it.
    ///
    /// An unknown scale set returns NotFound. Keep the returned handle alive
    /// for the entire GitHub message session.
    fn try_lock_scale_set(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        scale_set_id: Uuid,
    ) -> Result<Option<Box<dyn ScaleSetSession>>>;
}
