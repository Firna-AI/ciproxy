//! Resolve exact claims and retry pending activation through existing lifecycle paths.

use uuid::Uuid;

use crate::app::webhook::{
    Account, AccountKind, Actor, Installation, InstallationAction, InstallationEvent,
};
use crate::error::{Error, Result};

use super::{AppService, InstallationOutcome};

impl AppService {
    pub(super) async fn resolve_link(&self, id: Uuid) -> Result<InstallationOutcome> {
        let claim = self
            .store
            .claim(id)
            .await?
            .ok_or(Error::OrganizationInactive)?;
        if !claim.recovery().is_ready() {
            return Ok(InstallationOutcome::AwaitingAssignment { claim_id: claim.id });
        }
        self.handle_installation(InstallationEvent {
            action: InstallationAction::Created,
            installation: Installation {
                id: claim.installation_id,
                account: Account {
                    id: claim.github_id,
                    login: claim.login,
                    kind: AccountKind::Organization,
                },
            },
            requester: Some(Actor {
                id: claim.requester_id,
            }),
            sender: Actor {
                id: claim.requester_id,
            },
        })
        .await
    }

    pub(super) async fn activate_workspace(&self, workspace: Uuid) -> Result<()> {
        let mut failure = None;
        for organization in self.store.activation_candidates(workspace).await? {
            if let Err(error) = self.refresh(organization.installation_id).await {
                failure = Some(error);
            }
        }
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(())
    }
}
