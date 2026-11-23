//! Startup capabilities minted from the saved delivery's current durable authority.

use ciproxy_store_interface::overflow::record::StartRecord;
use sandbox_interface::{BackendCreateSandboxRequest, BackendSandbox, SandboxFiles};
use uuid::Uuid;

use crate::error::Result;
use crate::provider::{Capabilities, GuestLauncher};
use crate::session::Session;

use super::OverflowService;

/// Only this module can construct authority for create, input, or launch.
pub(super) struct LaunchAuthority {
    capabilities: Capabilities,
}

impl LaunchAuthority {
    pub(super) fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
    pub(super) async fn create(
        &self,
        request: BackendCreateSandboxRequest,
    ) -> sandbox_interface::Result<BackendSandbox> {
        self.capabilities.lifecycle.create_sandbox(request).await
    }

    pub(super) fn files(&self) -> &dyn SandboxFiles {
        &*self.capabilities.files
    }

    pub(super) fn launcher(&self) -> &dyn GuestLauncher {
        &*self.capabilities.launcher
    }
}

impl OverflowService {
    pub(super) async fn launch_authority(
        &self,
        session: &Session,
        workspace_id: Uuid,
        record: &StartRecord,
    ) -> Result<Option<LaunchAuthority>> {
        let delivery_id = record.delivery_id;
        let Some(account) = session
            .call(move |session| session.startup_account(delivery_id))
            .await?
            .filter(|account| account.owns_start(record))
        else {
            return Ok(None);
        };
        let capabilities = self
            .capabilities(workspace_id, record.provider, &account)
            .await?;
        if !session
            .call(move |session| session.startup_account(delivery_id))
            .await?
            .is_some_and(|account| account.owns_start(record))
        {
            return Ok(None);
        }
        Ok(Some(LaunchAuthority { capabilities }))
    }
}
