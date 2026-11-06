//! Reusable gateway Admin checks and delayed GitHub installation authority.

use async_trait::async_trait;
use chrono::Duration;
use ciproxy_gateway::boundary::Authorization;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use crate::state::query;

use super::AccessService;

impl Authorization for AccessService {
    fn require_admin(
        &self,
        principal: Uuid,
        workspace: Uuid,
    ) -> ciproxy_gateway::error::Result<()> {
        let user = match self.store.dashboard_user(principal) {
            Ok(Some(user)) => user,
            Ok(None) => return Err(ciproxy_gateway::error::Error::Forbidden),
            Err(source) => return Err(ciproxy_gateway::error::Error::internal(source)),
        };
        if self.runtime.now() >= user.checked_at + Duration::hours(1) {
            return Err(ciproxy_gateway::error::Error::Forbidden);
        }
        match self.store.dashboard_role(workspace, user.github_id) {
            Ok(Some(Role::Admin)) => Ok(()),
            Ok(_) => Err(ciproxy_gateway::error::Error::Forbidden),
            Err(source) => Err(ciproxy_gateway::error::Error::internal(source)),
        }
    }
}

#[async_trait]
impl ciproxy_github::app::InstallAuthority for AccessService {
    async fn intended_workspace(
        &self,
        requester: i64,
        github_organization_id: i64,
    ) -> ciproxy_github::error::Result<Option<Uuid>> {
        let target = query(self.store.clone(), move |store| {
            store.install_intent(requester, github_organization_id)
        })
        .await;
        let workspace = match target {
            Ok(Some(workspace)) => workspace,
            Ok(None) => return Ok(None),
            Err(_) => return Err(ciproxy_github::error::Error::OrganizationInactive),
        };
        let user = match self.store.dashboard_user_by_github(requester) {
            Ok(Some(user)) => user,
            _ => return Err(ciproxy_github::error::Error::OrganizationInactive),
        };
        if self.refresh_user(user).await.is_err() {
            return Err(ciproxy_github::error::Error::OrganizationInactive);
        }
        if self
            .store
            .dashboard_role(workspace, requester)
            .ok()
            .flatten()
            != Some(Role::Admin)
        {
            return Err(ciproxy_github::error::Error::OrganizationInactive);
        }
        let organizations = match self.store.organizations(workspace) {
            Ok(value) => value,
            Err(_) => return Err(ciproxy_github::error::Error::OrganizationInactive),
        };
        let existing = organizations.iter().any(|organization| {
            organization.github_id == github_organization_id
                && organization.app_state != AppState::Removed
        });
        let free = match self.store.billing(workspace) {
            Ok(record) => record.state.tier == Tier::Free,
            Err(_) => return Err(ciproxy_github::error::Error::OrganizationInactive),
        };
        if !existing
            && free
            && organizations
                .iter()
                .any(|organization| organization.app_state != AppState::Removed)
        {
            return Err(ciproxy_github::error::Error::OrganizationInactive);
        }
        Ok(Some(workspace))
    }
    async fn ensure_initial_admin(
        &self,
        workspace: Uuid,
        github_user_id: i64,
    ) -> ciproxy_github::error::Result<()> {
        let at = self.runtime.now();
        match query(self.store.clone(), move |store| {
            store.ensure_dashboard_initial_admin(workspace, github_user_id, at)
        })
        .await
        {
            Ok(()) => Ok(()),
            Err(_) => Err(ciproxy_github::error::Error::OrganizationInactive),
        }
    }
}
