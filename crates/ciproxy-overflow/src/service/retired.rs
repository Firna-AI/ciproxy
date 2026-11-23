//! Retired deliveries use inspection to preserve ambiguous or running guests.

use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{ProviderRef, SandboxState};
use uuid::Uuid;

use crate::api::StartOutcome;
use crate::error::{Error, Result};
use crate::maintenance::{LaunchState, MaintenanceCapabilities};
use crate::session::Session;

use super::OverflowService;

impl OverflowService {
    pub(super) async fn reconcile_retired(
        &self,
        session: &Session,
        workspace_id: Uuid,
        maintenance: &MaintenanceCapabilities,
        mut record: StartRecord,
    ) -> Result<StartOutcome> {
        if record.phase == StartPhase::Deleting {
            self.delete(session, maintenance, &mut record).await?;
            return Ok(StartOutcome::Ended(record));
        }
        if record.phase == StartPhase::Creating {
            let recovered = maintenance
                .recover_sandbox_create(self.create_request(workspace_id, &record)?)
                .await;
            match recovered {
                Ok(Some(sandbox)) => {
                    let state = self.adopt(session, &mut record, sandbox).await?;
                    if matches!(
                        state,
                        SandboxState::Failed | SandboxState::Destroyed | SandboxState::Destroying
                    ) {
                        self.request_delete(session, maintenance, &mut record)
                            .await?;
                        return Ok(StartOutcome::Ended(record));
                    }
                }
                Ok(None) | Err(_) => return Ok(StartOutcome::Pending(record)),
            }
        }
        let sandbox =
            ProviderRef::new(record.sandbox.as_ref().ok_or(Error::OwnershipUnknown)?.id());
        match maintenance.inspect_sandbox(sandbox.clone()).await {
            Err(sandbox_interface::Error::NotFound { .. }) => {
                self.request_delete(session, maintenance, &mut record)
                    .await?;
                return Ok(StartOutcome::Ended(record));
            }
            Ok(sandbox)
                if matches!(
                    sandbox.state,
                    SandboxState::Failed | SandboxState::Destroyed | SandboxState::Destroying
                ) =>
            {
                self.request_delete(session, maintenance, &mut record)
                    .await?;
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
        match maintenance
            .inspect_launch(sandbox, record.delivery_id)
            .await
        {
            Ok(LaunchState::Started) => {
                self.save_confirmed_launch(session, &mut record).await?;
                Ok(StartOutcome::Started(record))
            }
            Ok(LaunchState::NotStarted) => {
                self.request_delete(session, maintenance, &mut record)
                    .await?;
                Ok(StartOutcome::Ended(record))
            }
            Err(_) => Ok(StartOutcome::Pending(record)),
        }
    }
}
