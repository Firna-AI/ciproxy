//! Persistent adoption shared by create execution and recover-only cleanup.

use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{BackendSandbox, SandboxState};

use crate::error::{Error, Result};
use crate::session::Session;

use super::OverflowService;

impl OverflowService {
    pub(super) async fn adopt(
        &self,
        session: &Session,
        record: &mut StartRecord,
        sandbox: BackendSandbox,
    ) -> Result<SandboxState> {
        record.sandbox = Some(KnownSandbox::observed(
            sandbox.provider_ref.as_str().to_owned(),
            self.dependencies.clock.now(),
            record
                .dispatch
                .as_ref()
                .ok_or(Error::OwnershipUnknown)?
                .lifetime_seconds,
        ));
        record.phase = StartPhase::Preparing;
        let changed = record.clone();
        session
            .call(move |session| session.save_start(&changed))
            .await?;
        Ok(sandbox.state)
    }
}
