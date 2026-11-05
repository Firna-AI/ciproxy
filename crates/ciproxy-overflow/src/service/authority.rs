//! Startup capabilities minted from the workspace's current durable authority.

use ciproxy_store_interface::overflow::store::OverflowSnapshot;
use sandbox_interface::{BackendCreateSandboxRequest, BackendSandbox, SandboxFiles};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::provider::{Capabilities, GuestLauncher};
use crate::session::Session;

use super::OverflowService;

/// Only this module can construct authority for create, input, or launch.
pub(super) struct LaunchAuthority {
    backend: Capabilities,
}

impl LaunchAuthority {
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

pub(super) fn active(snapshot: &OverflowSnapshot, key_id: Uuid) -> bool {
    snapshot.enabled
        && snapshot.accounts.iter().any(|account| {
            account.account.id == key_id
                && account.account.retired_at.is_none()
                && account.account.rejected_at.is_none()
        })
}

impl OverflowService {
    pub(super) async fn launch_authority(
        &self,
        session: &Session,
        workspace_id: Uuid,
        key_id: Uuid,
    ) -> Result<Option<LaunchAuthority>> {
        let snapshot = session.call(|session| session.snapshot()).await?;
        if !active(&snapshot, key_id) {
            return Ok(None);
        }
        let account = snapshot
            .accounts
            .iter()
            .find(|account| account.account.id == key_id)
            .ok_or(Error::OwnershipUnknown)?;
        let backend = self.backend(workspace_id, account).await?;
        let current = session.call(|session| session.snapshot()).await?;
        if !active(&current, key_id) {
            return Ok(None);
        }
        Ok(Some(LaunchAuthority { backend }))
    }
}
