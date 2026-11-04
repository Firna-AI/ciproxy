//! App state changes that preserve running jobs and shared workspace resources.

use ciproxy_store_interface::organizations::AppState;

use crate::error::Result;

use super::webhook::{InstallationAction, InstallationEvent};
use super::{AppService, InstallationOutcome, LifecycleEvent};

impl AppService {
    pub(super) async fn lifecycle_event(
        &self,
        event: InstallationEvent,
    ) -> Result<InstallationOutcome> {
        let Some(mut organization) = self.store.by_installation(event.installation.id).await?
        else {
            return Ok(InstallationOutcome::Ignored);
        };
        match event.action {
            InstallationAction::Deleted => {
                self.store.remove(&organization, self.runtime.now()).await?;
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
            }
            InstallationAction::Suspend if organization.app_state != AppState::Removed => {
                self.store.suspend(&organization, true).await?;
                organization.app_state = AppState::Suspended;
                self.lifecycle
                    .changed(LifecycleEvent::Suspended(organization.clone()))
                    .await?;
            }
            InstallationAction::Unsuspend if organization.app_state != AppState::Removed => {
                self.store.suspend(&organization, false).await?;
                organization.app_state = AppState::Active;
                self.reconcile(&organization).await?;
                self.lifecycle
                    .changed(LifecycleEvent::Active(organization.clone()))
                    .await?;
            }
            _ => return Ok(InstallationOutcome::Ignored),
        }
        Ok(InstallationOutcome::Organization(organization))
    }
}
