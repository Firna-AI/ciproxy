//! Authoritative suspension and immediate verified removal under one lease.

use ciproxy_store_interface::github::GithubInstallationState;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};

use crate::error::{Error, Result};
use crate::installation_state::InstalledApp;
use crate::persistence::mutation::MutationLease;

use super::webhook::InstallationEvent;
use super::{AppService, InstallationOutcome, LifecycleEvent};

impl AppService {
    pub(super) async fn remove_installation(
        &self,
        event: InstallationEvent,
        lease: &dyn MutationLease,
    ) -> Result<InstallationOutcome> {
        let Some(mut organization) = self.store.by_installation(event.installation.id).await?
        else {
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
        self.store.remove(organization, self.runtime.now()).await?;
        organization.app_state = AppState::Removed;
        let last_organization = !self
            .store
            .has_organizations(organization.workspace_id)
            .await?;
        self.lifecycle
            .changed(LifecycleEvent::Removed {
                organization: organization.clone(),
                last_organization,
            })
            .await?;
        Ok(())
    }

    pub(super) async fn lifecycle_event(
        &self,
        event: InstallationEvent,
        current: InstalledApp,
        lease: &dyn MutationLease,
    ) -> Result<InstallationOutcome> {
        let Some(organization) = self.store.by_installation(event.installation.id).await? else {
            return Ok(InstallationOutcome::Ignored);
        };
        if organization.app_state == AppState::Removed {
            return Ok(InstallationOutcome::Ignored);
        }
        let organization = self
            .synchronize(organization, current, None, true, lease)
            .await?;
        Ok(InstallationOutcome::Organization(organization))
    }

    pub(super) async fn synchronize(
        &self,
        mut organization: Organization,
        current: InstalledApp,
        access: Option<RepositoryAccess>,
        notify: bool,
        lease: &dyn MutationLease,
    ) -> Result<Organization> {
        let editing = access.is_some();
        let desired = if current.suspended {
            AppState::Suspended
        } else {
            AppState::Active
        };
        let changed =
            organization.app_state != desired || organization.login != current.account.login;
        if changed {
            lease.check().await?;
            organization = self
                .store
                .save_snapshot(
                    &organization,
                    GithubInstallationState {
                        github_id: current.account.id,
                        installation_id: current.id,
                        login: current.account.login,
                        suspended: current.suspended,
                    },
                )
                .await?;
        }
        if desired == AppState::Active {
            if let Some(access) = &access {
                organization.repositories = access.clone();
            }
            self.reconcile(&organization, lease).await?;
            if let Some(access) = access {
                self.store.save_access(&organization, access).await?;
            }
        }
        if changed || notify {
            let event = if desired == AppState::Active {
                LifecycleEvent::Active(organization.clone())
            } else {
                LifecycleEvent::Suspended(organization.clone())
            };
            self.lifecycle.changed(event).await?;
        }
        if desired == AppState::Suspended && editing {
            return Err(Error::OrganizationInactive);
        }
        Ok(organization)
    }
}
