//! Installation and repository persistence used by the App service.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::installation_association::SignedInstallation;
use ciproxy_store_interface::installation_links::{InstallationClaim, OrganizationActivation};
use ciproxy_store_interface::labels::Label;
use ciproxy_store_interface::organizations::{AppState, Organization};
use ciproxy_store_interface::scale_sets::{ScaleSet, ScaleSetBinding};
use uuid::Uuid;

use crate::error::Result;

use super::session::{ListenerTarget, SessionLease, SessionStore};
use super::{StoreAdapter, query};

/// Store operations used only by App installation orchestration.
#[cfg_attr(test, unimock::unimock(api = InstallationStoreMock))]
#[async_trait]
pub trait InstallationStore: Send + Sync {
    /// Read current and retained registrations for workspace label reconciliation.
    async fn organizations(&self, workspace_id: Uuid) -> Result<Vec<Organization>>;
    /// Read the exact binding's retirement and completion state.
    async fn binding(&self, target: ListenerTarget) -> Result<Option<ScaleSetBinding>>;
    /// Own the listener session before deleting a retired remote resource.
    async fn lock_binding(&self, target: ListenerTarget) -> Result<Option<Box<dyn SessionLease>>>;
    /// Confirm cleanup without deleting the binding or its job history.
    async fn complete_binding(
        &self,
        workspace_id: Uuid,
        set: ScaleSet,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Read separate local admission for retained links.
    async fn activation(&self, organization: &Organization) -> Result<OrganizationActivation>;
    /// Read an exact requester-owned claim.
    async fn claim(&self, id: Uuid) -> Result<Option<InstallationClaim>>;
    /// Join signed ownership and flow before admission or remote preparation.
    async fn associate(&self, signed: SignedInstallation) -> Result<InstallationClaim>;
    /// Reconcile plan holds and return the workspace's current registrations.
    async fn activation_candidates(&self, workspace_id: Uuid) -> Result<Vec<Organization>>;
    /// Look up the registration named by a verified installation.
    async fn by_installation(&self, installation_id: i64) -> Result<Option<Organization>>;
    /// Look up current and retained registrations of a GitHub organization.
    async fn by_github_id(&self, github_id: i64) -> Result<Vec<Organization>>;
    /// Read an undelivered initial Admin assignment.
    async fn pending_admin(&self, workspace_id: Uuid) -> Result<Option<i64>>;
    /// Confirm delivery to the role service.
    async fn confirm_admin(&self, workspace_id: Uuid) -> Result<()>;
    /// Read retained workspace labels, including edits and deletions.
    async fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>>;
    /// Read active and retired bindings until their remote cleanup is confirmed.
    async fn scale_sets(&self, organization: &Organization) -> Result<Vec<ScaleSet>>;
    /// Read a retained managed group across this GitHub account's registrations.
    async fn runner_group(&self, github_id: i64) -> Result<Option<i64>>;
}

#[async_trait]
impl InstallationStore for StoreAdapter {
    async fn organizations(&self, workspace_id: Uuid) -> Result<Vec<Organization>> {
        query(self.store.clone(), move |store| {
            store.organizations(workspace_id)
        })
        .await
    }
    async fn binding(&self, target: ListenerTarget) -> Result<Option<ScaleSetBinding>> {
        query(self.store.clone(), move |store| {
            store.scale_set_binding(
                target.workspace_id,
                target.organization_id,
                target.scale_set_id,
            )
        })
        .await
    }
    async fn lock_binding(&self, target: ListenerTarget) -> Result<Option<Box<dyn SessionLease>>> {
        SessionStore::lock(self, target).await
    }
    async fn complete_binding(
        &self,
        workspace_id: Uuid,
        set: ScaleSet,
        at: DateTime<Utc>,
    ) -> Result<()> {
        query(self.store.clone(), move |store| {
            store.complete_scale_set_removal(workspace_id, &set, at)
        })
        .await
    }
    async fn activation(&self, organization: &Organization) -> Result<OrganizationActivation> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.organization_activation(organization.workspace_id, organization.id)
        })
        .await
    }
    async fn claim(&self, id: Uuid) -> Result<Option<InstallationClaim>> {
        query(self.store.clone(), move |store| {
            store.installation_claim(id)
        })
        .await
    }
    async fn associate(&self, signed: SignedInstallation) -> Result<InstallationClaim> {
        query(self.store.clone(), move |store| {
            store.associate_signed_installation(&signed)
        })
        .await
    }
    async fn activation_candidates(&self, workspace: Uuid) -> Result<Vec<Organization>> {
        query(self.store.clone(), move |store| {
            store.reconcile_organization_billing(workspace)?;
            let mut pending = Vec::new();
            for organization in store.organizations(workspace)? {
                if organization.app_state == AppState::Active
                    && !store
                        .organization_activation(workspace, organization.id)?
                        .admitted
                {
                    pending.push(organization);
                }
            }
            Ok(pending)
        })
        .await
    }
    async fn by_installation(&self, installation_id: i64) -> Result<Option<Organization>> {
        query(self.store.clone(), move |store| {
            store.organization_by_installation_id(installation_id)
        })
        .await
    }
    async fn by_github_id(&self, github_id: i64) -> Result<Vec<Organization>> {
        query(self.store.clone(), move |store| {
            store.organizations_by_github_id(github_id)
        })
        .await
    }
    async fn pending_admin(&self, workspace_id: Uuid) -> Result<Option<i64>> {
        query(self.store.clone(), move |store| {
            store.pending_initial_admin(workspace_id)
        })
        .await
    }
    async fn confirm_admin(&self, workspace_id: Uuid) -> Result<()> {
        query(self.store.clone(), move |store| {
            store.confirm_initial_admin(workspace_id)
        })
        .await
    }
    async fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>> {
        query(self.store.clone(), move |store| store.labels(workspace_id)).await
    }
    async fn scale_sets(&self, organization: &Organization) -> Result<Vec<ScaleSet>> {
        let organization = organization.clone();
        query(self.store.clone(), move |store| {
            store.dashboard_scale_sets(organization.workspace_id, organization.id)
        })
        .await
    }
    async fn runner_group(&self, github_id: i64) -> Result<Option<i64>> {
        query(self.store.clone(), move |store| {
            store.github_runner_group(github_id)
        })
        .await
    }
}
