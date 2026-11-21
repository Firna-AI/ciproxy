//! Atomic lifecycle writes and ordered outbox queries through blocking workers.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::github_lifecycle::{
    GithubLifecycleUpdate, GithubRegistration, LifecycleDelivery,
};
use ciproxy_store_interface::organizations::Organization;

use crate::error::Result;

use super::{StoreAdapter, query};

/// One persistence seam for every App lifecycle state transition and notification.
#[cfg_attr(test, unimock::unimock(api = LifecycleStoreMock))]
#[async_trait]
pub trait LifecycleStore: Send + Sync {
    /// Register prepared resources and its immutable lifecycle event atomically.
    async fn register(&self, registration: GithubRegistration) -> Result<Organization>;
    /// Save verified state and its immutable lifecycle event atomically.
    async fn update(
        &self,
        organization: &Organization,
        update: GithubLifecycleUpdate,
    ) -> Result<Organization>;
    /// Save removal/credential bookkeeping and immutable cleanup atomically.
    async fn remove(&self, organization: &Organization, at: DateTime<Utc>) -> Result<Organization>;
    /// Discover stable GitHub identities with pending delivery after a restart.
    async fn organizations(&self) -> Result<Vec<i64>>;
    /// Read the next events in immutable sequence order.
    async fn pending(&self, github_id: i64) -> Result<Vec<LifecycleDelivery>>;
    /// Delete an event only after its consumer accepted the exact saved snapshot.
    async fn confirm(&self, github_id: i64, sequence: i64) -> Result<()>;
}

#[async_trait]
impl LifecycleStore for StoreAdapter {
    async fn register(&self, registration: GithubRegistration) -> Result<Organization> {
        query(self.store.clone(), move |store| {
            store.register_github_installation(&registration)
        })
        .await
    }
    async fn update(
        &self,
        organization: &Organization,
        update: GithubLifecycleUpdate,
    ) -> Result<Organization> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.update_github_installation(organization.workspace_id, organization.id, &update)
        })
        .await
    }
    async fn remove(&self, organization: &Organization, at: DateTime<Utc>) -> Result<Organization> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.remove_github_installation(
                organization.workspace_id,
                organization.id,
                organization.installation_id,
                at,
            )
        })
        .await
    }
    async fn organizations(&self) -> Result<Vec<i64>> {
        query(self.store.clone(), move |store| {
            store.pending_github_lifecycle_organizations()
        })
        .await
    }
    async fn pending(&self, github_id: i64) -> Result<Vec<LifecycleDelivery>> {
        query(self.store.clone(), move |store| {
            store.pending_github_lifecycle(github_id)
        })
        .await
    }
    async fn confirm(&self, github_id: i64, sequence: i64) -> Result<()> {
        query(self.store.clone(), move |store| {
            store.confirm_github_lifecycle(github_id, sequence)
        })
        .await
    }
}
