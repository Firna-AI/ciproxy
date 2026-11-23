//! Single-dispatch create, recover-only reconciliation, and lifetime preservation.

use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{BackendCreateSandboxRequest, ProviderRef, SandboxState};
use uuid::Uuid;

use crate::api::{StartOutcome, WaitReason};
use crate::error::{Error, Result, ResultExt};
use crate::provider::SandboxStart;
use crate::session::Session;

use super::OverflowService;
use super::preparation::DispatchOutcome;

impl OverflowService {
    pub(super) fn create_request(
        &self,
        workspace_id: Uuid,
        record: &StartRecord,
    ) -> Result<BackendCreateSandboxRequest> {
        let parameters = record.dispatch.as_ref().ok_or(Error::OwnershipUnknown)?;
        let provider = self.dependencies.providers.get(record.provider)?;
        Ok(provider.factory.create_request(SandboxStart {
            workspace_id,
            runner_id: record.runner_id,
            delivery_id: record.delivery_id,
            deployment: &record.deployment,
            size: &parameters.size.name,
            image: &record.snapshot,
            lifetime_seconds: parameters.lifetime_seconds,
        }))
    }

    pub(super) async fn execute(
        &self,
        session: &Session,
        workspace_id: Uuid,
        mut record: StartRecord,
    ) -> Result<StartOutcome> {
        if matches!(
            record.phase,
            StartPhase::Ended | StartPhase::Refused | StartPhase::CancelledUnsent
        ) {
            return Ok(StartOutcome::Ended(record));
        }
        let delivery = record.delivery_id;
        if record.phase == StartPhase::Unsent
            && session
                .call(move |session| session.startup_account(delivery))
                .await?
                .is_none()
        {
            return self.cancel_unsent(session, &mut record).await;
        }
        let snapshot = session.call(|session| session.snapshot()).await?;
        let mut account = snapshot
            .accounts
            .iter()
            .find(|account| account.owns_start(&record))
            .ok_or(Error::OwnershipUnknown)?
            .clone();
        let maintenance = self
            .maintenance(workspace_id, record.provider, &account)
            .await?;
        let delivery = record.delivery_id;
        if session
            .call(move |session| session.startup_account(delivery))
            .await?
            .is_none()
        {
            if record.phase == StartPhase::Unsent {
                return self.cancel_unsent(session, &mut record).await;
            }
            return self
                .reconcile_retired(session, workspace_id, &maintenance, record)
                .await;
        }
        if matches!(record.phase, StartPhase::Deleting | StartPhase::Writing) {
            record.phase = StartPhase::Deleting;
            let changed = record.clone();
            session
                .call(move |session| session.save_start(&changed))
                .await?;
            self.delete(session, &maintenance, &mut record).await?;
            return Ok(StartOutcome::Ended(record));
        }
        if matches!(record.phase, StartPhase::Unsent | StartPhase::Creating) {
            let result = if record.phase == StartPhase::Unsent {
                match self
                    .dispatch_unsent(session, workspace_id, &mut record, &mut account)
                    .await?
                {
                    DispatchOutcome::Submitted(result) => result,
                    DispatchOutcome::Stopped(outcome) => return Ok(*outcome),
                }
            } else {
                maintenance
                    .recover_sandbox_create(self.create_request(workspace_id, &record)?)
                    .await
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
                    tracing::warn!(
                        provider = record.provider.as_str(),
                        "Create outcome requires recover-only reconciliation"
                    );
                    return Ok(StartOutcome::Pending(record));
                }
            };
            let state = self.adopt(session, &mut record, sandbox).await?;
            match state {
                SandboxState::Ready => {}
                SandboxState::Failed | SandboxState::Destroyed | SandboxState::Destroying => {
                    self.request_delete(session, &maintenance, &mut record)
                        .await?;
                    return Ok(StartOutcome::Ended(record));
                }
                _ => return Ok(StartOutcome::Pending(record)),
            }
        }

        let provider_ref =
            ProviderRef::new(record.sandbox.as_ref().ok_or(Error::OwnershipUnknown)?.id());
        let sandbox = match maintenance.inspect_sandbox(provider_ref).await {
            Ok(sandbox) => sandbox,
            Err(sandbox_interface::Error::NotFound { .. }) => {
                record.phase = StartPhase::Deleting;
                let changed = record.clone();
                session
                    .call(move |session| session.save_start(&changed))
                    .await?;
                self.delete(session, &maintenance, &mut record).await?;
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
                self.delete(session, &maintenance, &mut record).await?;
                return Ok(StartOutcome::Ended(record));
            }
            _ => return Ok(StartOutcome::Pending(record)),
        }
        self.launch(session, workspace_id, &maintenance, &mut record)
            .await
    }
}
