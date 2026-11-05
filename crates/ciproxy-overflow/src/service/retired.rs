//! Retired deliveries use inspection to preserve ambiguous or running guests.

use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{ProviderRef, SandboxState};
use uuid::Uuid;

use crate::api::StartOutcome;
use crate::error::{Error, Result};
use crate::maintenance::{LaunchState, MaintenanceCapabilities};
use crate::session::Session;

use super::OverflowService;
use super::execution::create_request;

impl OverflowService {
    pub(super) async fn reconcile_retired(
        &self,
        session: &Session,
        workspace_id: Uuid,
        backend: &MaintenanceCapabilities,
        mut record: StartRecord,
    ) -> Result<StartOutcome> {
        if record.phase == StartPhase::Deleting {
            self.delete(session, backend, &mut record).await?;
            return Ok(StartOutcome::Ended(record));
        }
        if record.phase == StartPhase::Creating {
            let recovered = backend
                .recover_sandbox_create(create_request(workspace_id, &record))
                .await;
            match recovered {
                Ok(Some(sandbox)) => {
                    let state = self.adopt(session, &mut record, sandbox).await?;
                    if matches!(
                        state,
                        SandboxState::Failed | SandboxState::Destroyed | SandboxState::Destroying
                    ) {
                        self.request_delete(session, backend, &mut record).await?;
                        return Ok(StartOutcome::Ended(record));
                    }
                }
                Ok(None) | Err(_) => return Ok(StartOutcome::Pending(record)),
            }
        }
        let sandbox =
            ProviderRef::new(record.sandbox.as_ref().ok_or(Error::OwnershipUnknown)?.id());
        match backend.inspect_sandbox(sandbox.clone()).await {
            Err(sandbox_interface::Error::NotFound { .. }) => {
                self.request_delete(session, backend, &mut record).await?;
                return Ok(StartOutcome::Ended(record));
            }
            Ok(sandbox)
                if matches!(
                    sandbox.state,
                    SandboxState::Failed | SandboxState::Destroyed | SandboxState::Destroying
                ) =>
            {
                self.request_delete(session, backend, &mut record).await?;
                return Ok(StartOutcome::Ended(record));
            }
            Ok(_) => {}
            Err(_) => {
                return if self.unavailable_lifecycle(session, &mut record).await? {
                    Ok(StartOutcome::Ended(record))
                } else {
                    Ok(StartOutcome::Pending(record))
                };
            }
        }
        if record.phase == StartPhase::Running {
            return Ok(StartOutcome::Started(record));
        }
        match backend.inspect_launch(sandbox, record.delivery_id).await {
            Ok(LaunchState::Started) => {
                record.phase = StartPhase::Running;
                let saved = record.clone();
                session
                    .call(move |session| session.save_start(&saved))
                    .await?;
                Ok(StartOutcome::Started(record))
            }
            Ok(LaunchState::NotStarted) => {
                self.request_delete(session, backend, &mut record).await?;
                Ok(StartOutcome::Ended(record))
            }
            Err(_) => Ok(StartOutcome::Pending(record)),
        }
    }
}
