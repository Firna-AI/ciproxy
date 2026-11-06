//! Atomic lifecycle state writes and replay even when current state already matches.

use ciproxy_store_interface::github_lifecycle::{GithubLifecycleTarget, GithubLifecycleUpdate};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};

use crate::error::{Error, Result};
use crate::installation_state::InstalledApp;
use crate::persistence::mutation::MutationLease;

use super::webhook::InstallationEvent;
use super::{AppService, InstallationOutcome};

impl AppService {
    pub(super) async fn remove_installation(
        &self,
        event: InstallationEvent,
        lease: &dyn MutationLease,
    ) -> Result<InstallationOutcome> {
        let Some(mut organization) = self.store.by_installation(event.installation.id).await?
        else {
            self.deliver_lifecycle(event.installation.account.id, lease)
                .await?;
            return Ok(InstallationOutcome::Ignored);
        };
        self.retire_installation(&mut organization, lease).await?;
        Ok(InstallationOutcome::Organization(organization))
    }

    pub(super) async fn retire_installation(
        &self,
        organization: &mut Organization,
        lease: &dyn MutationLease,
    ) -> Result<()> {
        lease.check().await?;
        *organization = self.outbox.remove(organization, self.runtime.now()).await?;
        self.deliver_lifecycle(organization.github_id, lease).await
    }

    pub(super) async fn lifecycle_event(
        &self,
        event: InstallationEvent,
        current: InstalledApp,
        lease: &dyn MutationLease,
    ) -> Result<InstallationOutcome> {
        let Some(organization) = self.store.by_installation(event.installation.id).await? else {
            self.deliver_lifecycle(event.installation.account.id, lease)
                .await?;
            return Ok(InstallationOutcome::Ignored);
        };
        if organization.app_state == AppState::Removed {
            self.deliver_lifecycle(organization.github_id, lease)
                .await?;
            return Ok(InstallationOutcome::Ignored);
        }
        let organization = self
            .synchronize(organization, current, None, true, lease)
            .await?;
        Ok(InstallationOutcome::Organization(organization))
    }

    pub(super) async fn synchronize(
        &self,
        organization: Organization,
        current: InstalledApp,
        access: Option<RepositoryAccess>,
        notify: bool,
        lease: &dyn MutationLease,
    ) -> Result<Organization> {
        let editing = access.is_some();
        let mut desired = organization.clone();
        desired.login = current.account.login;
        desired.app_state = if current.suspended {
            AppState::Suspended
        } else {
            AppState::Active
        };
        if let Some(access) = &access {
            desired.repositories = access.clone();
        }
        let local = self.store.activation(&organization).await?;
        let target = if current.suspended {
            GithubLifecycleTarget::Suspended
        } else if local.billing_hold || local.admin_approval_hold {
            GithubLifecycleTarget::Inactive
        } else {
            GithubLifecycleTarget::Active(
                self.prepare(
                    &desired,
                    self.store.labels(desired.workspace_id).await?,
                    lease,
                )
                .await?,
            )
        };
        lease.check().await?;
        let saved = self
            .outbox
            .update(
                &organization,
                GithubLifecycleUpdate {
                    github_id: current.account.id,
                    installation_id: current.id,
                    login: desired.login,
                    repositories: if current.suspended { None } else { access },
                    target,
                    notify,
                },
            )
            .await?;
        self.deliver_lifecycle(saved.github_id, lease).await?;
        if current.suspended && editing {
            return Err(Error::OrganizationInactive);
        }
        Ok(saved)
    }
}
