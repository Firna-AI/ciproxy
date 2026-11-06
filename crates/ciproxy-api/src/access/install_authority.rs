//! Exact claim authority and link-time Admin approval without a latest target.

use async_trait::async_trait;
use ciproxy_github::app::{InstallAuthority, InstallationDecision};
use ciproxy_github::error::{Error, Result, ResultExt};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use crate::error;
use crate::state::query;

use super::AccessService;

#[async_trait]
impl InstallAuthority for AccessService {
    async fn installation_decision(
        &self,
        requester: i64,
        github_id: i64,
        installation: i64,
    ) -> Result<InstallationDecision> {
        let claim = query(self.store.clone(), move |store| {
            store.claim_for_installation(installation)
        })
        .await
        .internal()?;
        if let Some(claim) = claim {
            if claim.requester_id != requester
                || claim.github_id != github_id
                || !claim.user_verified
            {
                return Ok(InstallationDecision::AwaitingAssignment);
            }
            if claim.direct {
                return Ok(InstallationDecision::Direct);
            }
            let Some(workspace) = claim.workspace_id else {
                return Ok(InstallationDecision::AwaitingAssignment);
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
            return Ok(InstallationDecision::Target {
                workspace_id: workspace,
                admin_approved: approved,
                billing_hold,
            });
        }
        let pending = query(self.store.clone(), move |store| {
            Ok(store.has_pending_add(requester)?
                || store.has_add_installation(requester, installation)?)
        })
        .await
        .internal()?;
        Ok(if pending {
            InstallationDecision::AwaitingAssignment
        } else {
            InstallationDecision::Direct
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
