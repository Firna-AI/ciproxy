//! Reusable gateway Admin checks and delayed GitHub installation authority.

use chrono::Duration;
use ciproxy_gateway::boundary::Authorization;
use ciproxy_store_interface::dashboard::records::Role;
use uuid::Uuid;

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
