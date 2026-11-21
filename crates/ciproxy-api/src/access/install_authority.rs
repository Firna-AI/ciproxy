//! Exact claim authority and link-time Admin approval without a latest target.

use async_trait::async_trait;
use ciproxy_github::app::{InstallAuthority, InstallationDecision};
use ciproxy_github::error::{Error, Result, ResultExt};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::installation_links::{InstallationClaim, InstallationRecovery};
use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use crate::error;
use crate::state::query;

use super::AccessService;

#[async_trait]
impl InstallAuthority for AccessService {
    async fn installation_decision(
        &self,
        claim: &InstallationClaim,
    ) -> Result<InstallationDecision> {
        let requester = claim.requester_id;
        let github_id = claim.github_id;
        let workspace = match claim.recovery() {
            InstallationRecovery::Direct => {
                return Ok(InstallationDecision::Direct);
            }
            InstallationRecovery::Add { workspace_id } => workspace_id,
            _ => return Ok(InstallationDecision::AwaitingAssignment),
        };
        let approved = if let Some(organization) = claim.organization_id {
            !query(self.store.clone(), move |store| {
                store.organization_activation(workspace, organization)
            })
            .await
            .internal()?
            .admin_approval_hold
        } else {
            if let Some(user) = query(self.store.clone(), move |store| {
                store.dashboard_user_by_github(requester)
            })
            .await
            .internal()?
            {
                match self.refresh_link_identity(user).await {
                    Ok(_) => {}
                    Err(error::Error::Unauthorized) => {
                        let remains = query(self.store.clone(), move |store| {
                            store.dashboard_user_by_github(requester)
                        })
                        .await
                        .internal()?
                        .is_some();
                        if remains {
                            return Err(Error::AccessUnavailable(503));
                        }
                    }
                    Err(_) => return Err(Error::AccessUnavailable(503)),
                }
            }
            query(self.store.clone(), move |store| {
                store.dashboard_role(workspace, requester)
            })
            .await
            .internal()?
                == Some(Role::Admin)
        };
        let id = claim.id;
        let billing_hold = query(self.store.clone(), move |store| {
            store.authorize_installation_claim(id, approved)?;
            let organizations = store.organizations(workspace)?;
            let count = organizations
                .iter()
                .filter(|organization| organization.app_state != AppState::Removed)
                .count()
                + usize::from(!organizations.iter().any(|organization| {
                    organization.github_id == github_id
                        && organization.app_state != AppState::Removed
                }));
            Ok(store.billing(workspace)?.state.tier == Tier::Free && count > 1)
        })
        .await
        .internal()?;
        Ok(InstallationDecision::Target {
            workspace_id: workspace,
            admin_approved: approved,
            billing_hold,
        })
    }

    async fn ensure_initial_admin(&self, workspace: Uuid, user: i64) -> Result<()> {
        let now = self.runtime.now();
        query(self.store.clone(), move |store| {
            store.ensure_dashboard_initial_admin(workspace, user, now)
        })
        .await
        .internal()
    }
}
