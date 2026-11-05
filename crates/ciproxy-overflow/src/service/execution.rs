//! Single-dispatch create, recover-only reconciliation, and lifetime preservation.

use std::time::Duration;

use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{
    BackendCreateSandboxRequest, OperationId, ProviderRef, ResourceOwner, SandboxConsumer,
    SandboxId, SandboxLifetime, SandboxNetworkPolicy, SandboxSizeSelection, SandboxState,
};
use uuid::Uuid;

use crate::api::{StartOutcome, WaitReason};
use crate::cost_state;
use crate::error::{Error, Result, ResultExt};
use crate::session::Session;

use super::OverflowService;

pub(super) fn create_request(
    workspace_id: Uuid,
    record: &StartRecord,
) -> BackendCreateSandboxRequest {
    BackendCreateSandboxRequest {
        sandbox_id: SandboxId::from_uuid(record.runner_id),
        operation_id: OperationId::from_uuid(record.delivery_id),
        owner: ResourceOwner::platform(workspace_id),
        consumer: SandboxConsumer::Runtime,
        lifetime: SandboxLifetime::OneShot {
            max_lifetime: Duration::from_secs(u64::from(record.lifetime_seconds)),
        },
        size: Some(SandboxSizeSelection::ProviderSize(record.size.name.clone())),
        deployment_id: record.deployment.clone(),
        profile: "ciproxy".to_owned(),
        network: SandboxNetworkPolicy::Open,
        snapshot_provider_ref: Some(ProviderRef::new(&record.snapshot)),
    }
}

impl OverflowService {
    pub(super) async fn execute(
        &self,
        session: &Session,
        workspace_id: Uuid,
        mut record: StartRecord,
        dispatch: bool,
    ) -> Result<StartOutcome> {
        if matches!(record.phase, StartPhase::Ended | StartPhase::Refused) {
            return Ok(StartOutcome::Ended(record));
        }
        let snapshot = session.call(|session| session.snapshot()).await?;
        let mut account = snapshot
            .accounts
            .into_iter()
            .find(|account| account.account.id == record.key_id)
            .ok_or(Error::OwnershipUnknown)?;
        let backend = self.backend(workspace_id, &account).await?;
        if matches!(record.phase, StartPhase::Deleting | StartPhase::Writing) {
            record.phase = StartPhase::Deleting;
            let changed = record.clone();
            session
                .call(move |session| session.save_start(&changed))
                .await?;
            self.delete(session, &backend, &mut record).await?;
            return Ok(StartOutcome::Ended(record));
        }
        if record.phase == StartPhase::Creating {
            let request = create_request(workspace_id, &record);
            let result = if dispatch {
                backend.lifecycle.create_sandbox(request).await.map(Some)
            } else {
                backend.lifecycle.recover_sandbox_create(request).await
            };
            let sandbox = match result {
                Ok(Some(sandbox)) => sandbox,
                Ok(None) => return Ok(StartOutcome::Pending(record)),
                Err(sandbox_interface::Error::SandboxStartRefused { refusal, .. }) => {
                    self.save_refusal(session, &mut account, &mut record, &refusal)
                        .await?;
                    return Ok(StartOutcome::Waiting(WaitReason::Refused(refusal)));
                }
                Err(_) => {
                    tracing::warn!("Boat create outcome requires recover-only reconciliation");
                    return Ok(StartOutcome::Pending(record));
                }
            };
            record.sandbox_id = Some(sandbox.provider_ref.as_str().to_owned());
            cost_state::observe_existence(&mut record, self.dependencies.clock.now());
            record.phase = StartPhase::Preparing;
            let changed = record.clone();
            session
                .call(move |session| session.save_start(&changed))
                .await?;
            if sandbox.state != SandboxState::Ready {
                return Ok(StartOutcome::Pending(record));
            }
        }
        let provider_ref =
            ProviderRef::new(record.sandbox_id.as_ref().ok_or(Error::OwnershipUnknown)?);
        let sandbox = match backend.lifecycle.inspect_sandbox(provider_ref).await {
            Ok(sandbox) => sandbox,
            Err(sandbox_interface::Error::NotFound { .. }) => {
                record.phase = StartPhase::Deleting;
                let changed = record.clone();
                session
                    .call(move |session| session.save_start(&changed))
                    .await?;
                self.delete(session, &backend, &mut record).await?;
                return Ok(StartOutcome::Ended(record));
            }
            result => result.internal()?,
        };
        match sandbox.state {
            SandboxState::Ready => {}
            SandboxState::Failed | SandboxState::Destroyed | SandboxState::Destroying => {
                record.phase = StartPhase::Deleting;
                let changed = record.clone();
                session
                    .call(move |session| session.save_start(&changed))
                    .await?;
                self.delete(session, &backend, &mut record).await?;
                return Ok(StartOutcome::Ended(record));
            }
            _ => return Ok(StartOutcome::Pending(record)),
        }
        self.launch(session, workspace_id, &backend, &mut record)
            .await?;
        Ok(StartOutcome::Started(record))
    }
}
