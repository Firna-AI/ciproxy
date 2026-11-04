//! Async ownership and health of cross-process organization mutation leases.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ciproxy_store_interface::github::OrganizationLease;

use crate::error::{Error, Result, ResultExt};

use super::{StoreAdapter, query};

/// A live organization mutation lease, separate from a listener session.
#[cfg_attr(test, unimock::unimock(api = MutationLeaseMock))]
#[async_trait]
pub trait MutationLease: Send + Sync {
    /// Stop mutation when the dedicated database session is lost.
    async fn check(&self) -> Result<()>;
}

/// Shared mutation authority for each full GitHub organization identity.
#[cfg_attr(test, unimock::unimock(api = OrganizationMutationsMock))]
#[async_trait]
pub trait OrganizationMutations: Send + Sync {
    /// Wait for a dedicated lease without holding the ordinary query connection.
    async fn acquire(&self, github_id: i64) -> Result<Box<dyn MutationLease>>;
}

struct Lease {
    handle: Arc<Mutex<Box<dyn OrganizationLease>>>,
}

#[async_trait]
impl MutationLease for Lease {
    async fn check(&self) -> Result<()> {
        let handle = self.handle.clone();
        tokio::task::spawn_blocking(move || {
            let Ok(mut handle) = handle.lock() else {
                return Err(Error::OrganizationInactive);
            };
            handle.check_connection().internal()
        })
        .await
        .internal()?
    }
}

#[async_trait]
impl OrganizationMutations for StoreAdapter {
    async fn acquire(&self, github_id: i64) -> Result<Box<dyn MutationLease>> {
        let handle = query(self.store.clone(), move |store| {
            store.lock_github_organization(github_id)
        })
        .await?;
        Ok(Box::new(Lease {
            handle: Arc::new(Mutex::new(handle)),
        }))
    }
}
