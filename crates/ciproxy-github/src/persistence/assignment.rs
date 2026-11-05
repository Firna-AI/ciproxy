//! Tenant-scoped repository snapshots used to prepare agent assignment facts.

use async_trait::async_trait;
use ciproxy_store_interface::error;
use ciproxy_store_interface::github::Repository;

use crate::error::Result;
use crate::persistence::session::ListenerTarget;

use super::{StoreAdapter, query};

/// API metadata needed at the assignment boundary.
#[derive(Clone, Debug)]
pub struct AssignmentSnapshot {
    /// Stable GitHub organization identity, not the internal registration UUID.
    pub organization_id: i64,
    /// Repository identities and default branches from the API snapshot.
    pub repositories: Vec<Repository>,
}

/// Existing store reads used by the hosted assignment builder.
#[cfg_attr(test, unimock::unimock(api = AssignmentStoreMock))]
#[async_trait]
pub trait AssignmentStore: Send + Sync {
    /// Read only the workspace and organization named by the trusted target.
    async fn snapshot(&self, target: ListenerTarget) -> Result<AssignmentSnapshot>;
}

#[async_trait]
impl AssignmentStore for StoreAdapter {
    async fn snapshot(&self, target: ListenerTarget) -> Result<AssignmentSnapshot> {
        query(self.store.clone(), move |store| {
            let organization = store
                .organization(target.workspace_id, target.organization_id)?
                .ok_or(error::Error::NotFound)?;
            Ok(AssignmentSnapshot {
                organization_id: organization.github_id,
                repositories: store.repositories(target.workspace_id, target.organization_id)?,
            })
        })
        .await
    }
}
