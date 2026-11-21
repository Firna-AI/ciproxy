//! Async ownership and health of cross-process organization mutation leases.

use async_trait::async_trait;

use crate::error::Result;

use super::StoreAdapter;
use super::lease::{Lease, LeaseStore};

/// Shared mutation authority for each full GitHub organization identity.
#[cfg_attr(test, unimock::unimock(api = OrganizationMutationsMock))]
#[async_trait]
pub trait OrganizationMutations: Send + Sync {
    /// Wait for a dedicated lease without holding the ordinary query connection.
    async fn acquire(&self, github_id: i64) -> Result<Box<dyn Lease>>;
}

#[async_trait]
impl OrganizationMutations for StoreAdapter {
    async fn acquire(&self, github_id: i64) -> Result<Box<dyn Lease>> {
        self.store.organization_lease(github_id).await
    }
}
