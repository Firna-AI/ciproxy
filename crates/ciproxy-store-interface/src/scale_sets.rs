//! Scale set records and session ownership.

use uuid::Uuid;

use crate::error::Result;

/// A GitHub scale set for one organization label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaleSet {
    /// The internal scale set identifier.
    pub id: Uuid,
    /// The organization label served by the scale set.
    pub label_id: Uuid,
    /// The GitHub scale set identifier.
    pub github_id: i64,
    /// The GitHub runner group that restricts repository access.
    pub runner_group_id: i64,
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
    fn create_scale_set(&self, organization_id: Uuid, scale_set: &ScaleSet) -> Result<()>;
    /// Read all scale sets of one organization.
    fn scale_sets(&self, organization_id: Uuid) -> Result<Vec<ScaleSet>>;
    /// Try to own a session, returning None when another session owns it.
    ///
    /// An unknown scale set returns NotFound. Keep the returned handle alive
    /// for the entire GitHub message session.
    fn try_lock_scale_set(
        &self,
        organization_id: Uuid,
        scale_set_id: Uuid,
    ) -> Result<Option<Box<dyn ScaleSetSession>>>;
}
