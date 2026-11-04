//! Atomic lifecycle writes and durable ordered delivery queries.

mod queue;
mod rows;
mod state;
mod transitions;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::error::Result;
use ciproxy_store_interface::github_lifecycle::{
    GithubLifecycleStore, GithubLifecycleUpdate, GithubRegistration, LifecycleDelivery,
};
use ciproxy_store_interface::organizations::Organization;
use uuid::Uuid;

use crate::connection::PgStore;

impl GithubLifecycleStore for PgStore {
    fn register_github_installation(
        &self,
        registration: &GithubRegistration,
    ) -> Result<Organization> {
        transitions::register(self, registration)
    }
    fn update_github_installation(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        update: &GithubLifecycleUpdate,
    ) -> Result<Organization> {
        transitions::update(self, workspace_id, organization_id, update)
    }
    fn remove_github_installation(
        &self,
        workspace_id: Uuid,
        organization_id: Uuid,
        installation_id: i64,
        at: DateTime<Utc>,
    ) -> Result<Organization> {
        transitions::remove(self, workspace_id, organization_id, installation_id, at)
    }
    fn pending_github_lifecycle_organizations(&self) -> Result<Vec<i64>> {
        queue::organizations(self)
    }
    fn pending_github_lifecycle(&self, github_id: i64) -> Result<Vec<LifecycleDelivery>> {
        queue::pending(self, github_id)
    }
    fn confirm_github_lifecycle(&self, github_id: i64, sequence: i64) -> Result<()> {
        queue::confirm(self, github_id, sequence)
    }
}
