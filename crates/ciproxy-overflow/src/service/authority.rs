//! Startup capabilities minted from the saved delivery's current durable authority.

use sandbox_interface::{BackendCreateSandboxRequest, BackendSandbox, SandboxFiles};
use uuid::Uuid;

use crate::error::Result;
use crate::provider::{Capabilities, GuestLauncher};
use crate::session::Session;

use super::OverflowService;

/// Only this module can construct authority for create, input, or launch.
pub(super) struct LaunchAuthority {
    backend: Capabilities,
}

impl LaunchAuthority {
    pub(super) fn backend(&self) -> &Capabilities {
        &self.backend
    }
    pub(super) async fn create(
        &self,
        request: BackendCreateSandboxRequest,
    ) -> sandbox_interface::Result<BackendSandbox> {
        self.backend.lifecycle.create_sandbox(request).await
    }

    pub(super) fn files(&self) -> &dyn SandboxFiles {
        &*self.backend.files
    }

    pub(super) fn launcher(&self) -> &dyn GuestLauncher {
        &*self.backend.launcher
    }
}

impl OverflowService {
    pub(super) async fn launch_authority(
        &self,
        session: &Session,
        workspace_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Option<LaunchAuthority>> {
        let Some(account) = session
            .call(move |session| session.startup_account(delivery_id))
            .await?
        else {
            return Ok(None);
        };
        let backend = self.backend(workspace_id, &account).await?;
        if session
            .call(move |session| session.startup_account(delivery_id))
            .await?
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(LaunchAuthority { backend }))
    }
}
