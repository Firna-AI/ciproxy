//! First installation, retained reinstall, and requester-owned workspace choice.

use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::vm_size::VmSize;
use ciproxy_store_interface::workspaces::Workspace;

use crate::error::{Error, Result};
use crate::persistence::installation::Registration;

use super::webhook::{AccountKind, InstallationAction, InstallationEvent};
use super::{AppService, InstallationOutcome, LifecycleEvent};

impl AppService {
    pub(super) async fn handle_installation(
        &self,
        event: InstallationEvent,
    ) -> Result<InstallationOutcome> {
        if event.installation.account.kind != AccountKind::Organization {
            return Ok(InstallationOutcome::OrganizationsOnly {
                installation_id: event.installation.id,
                remove_url: format!(
                    "https://github.com/settings/installations/{}",
                    event.installation.id
                ),
            });
        }
        match event.action {
            InstallationAction::Created => self.install(event).await,
            InstallationAction::Deleted
            | InstallationAction::Suspend
            | InstallationAction::Unsuspend => self.lifecycle_event(event).await,
            InstallationAction::NewPermissionsAccepted => {
                self.refresh(event.installation.id).await?;
                Ok(InstallationOutcome::Ignored)
            }
            InstallationAction::Other => Ok(InstallationOutcome::Ignored),
        }
    }

    async fn install(&self, event: InstallationEvent) -> Result<InstallationOutcome> {
        let requester = event.requester.as_ref().unwrap_or(&event.sender).id;
        let github_id = event.installation.account.id;
        let registrations = self.store.by_github_id(github_id).await?;
        if let Some(previous) = registrations.iter().find(|organization| {
            organization.installation_id == event.installation.id
                && organization.app_state != AppState::Active
        }) {
            return Ok(InstallationOutcome::Organization(previous.clone()));
        }
        let target = self
            .authority
            .intended_workspace(requester, github_id)
            .await?;
        let current = registrations
            .iter()
            .find(|organization| organization.app_state != AppState::Removed);
        if let Some(current) = current
            && target.is_some_and(|target| current.workspace_id != target)
        {
            return Err(Error::WorkspaceConflict);
        }
        let retained = registrations
            .iter()
            .filter(|organization| {
                organization.app_state == AppState::Removed
                    && target.is_none_or(|workspace_id| organization.workspace_id == workspace_id)
                    && organization
                        .delete_after
                        .is_some_and(|deadline| deadline > self.runtime.now())
            })
            .max_by_key(|organization| organization.removed_at);
        let previous = current.or(retained);
        let workspace_id = previous
            .map(|organization| organization.workspace_id)
            .or(target)
            .unwrap_or_else(|| self.runtime.uuid());
        let workspace = if previous.is_none() && target.is_none() {
            Some(Workspace {
                id: workspace_id,
                name: event.installation.account.login.clone(),
                delete_after: None,
            })
        } else {
            None
        };
        let defaults = if workspace.is_some() {
            self.default_labels()
        } else {
            Vec::new()
        };
        let organization = Organization {
            id: previous
                .map(|organization| organization.id)
                .unwrap_or_else(|| self.runtime.uuid()),
            workspace_id,
            github_id,
            login: event.installation.account.login,
            installation_id: event.installation.id,
            app_state: AppState::Active,
            repositories: previous
                .map(|organization| organization.repositories.clone())
                .unwrap_or(RepositoryAccess::AllPrivate),
            removed_at: None,
            delete_after: None,
        };
        self.store
            .register(Registration {
                workspace,
                organization: organization.clone(),
                defaults,
                initial_admin: requester,
            })
            .await?;
        if let Some(user_id) = self.store.pending_admin(workspace_id).await? {
            self.authority
                .ensure_initial_admin(workspace_id, user_id)
                .await?;
            self.store.confirm_admin(workspace_id).await?;
        }
        self.reconcile(&organization).await?;
        self.lifecycle
            .changed(LifecycleEvent::Active(organization.clone()))
            .await?;
        Ok(InstallationOutcome::Organization(organization))
    }

    fn default_labels(&self) -> Vec<Label> {
        [
            ("ciproxy", 2, 8, 60),
            ("ciproxy-4vcpu", 4, 16, 80),
            ("ciproxy-8vcpu", 8, 32, 120),
        ]
        .into_iter()
        .map(|(name, vcpus, memory_gb, disk_gb)| Label {
            id: self.runtime.uuid(),
            name: name.to_owned(),
            size: VmSize {
                vcpus,
                memory_gb,
                disk_gb,
            },
            allow_machines: true,
            allow_boat: true,
            when_nothing_fits: WhenNothingFits::Wait,
        })
        .collect()
    }
}
