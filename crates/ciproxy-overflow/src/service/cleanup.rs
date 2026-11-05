//! One-use end callbacks, marked idle cleanup, and durable delete retries.

use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{ProviderRef, SandboxState};
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::api::Retirement;
use crate::error::{Error, Result, ResultExt};
use crate::provider::Capabilities;
use crate::secrets::digest;
use crate::session::Session;
use crate::spend;

use super::OverflowService;
use super::execution::create_request;

impl OverflowService {
    pub(super) async fn end_report(&self, token: Secret, runner_id: Uuid) -> Result<()> {
        let store = self.dependencies.store.clone();
        let hash = digest(&token);
        let lookup = hash.clone();
        let workspace_id = spawn_blocking(move || store.overflow_token_workspace(&lookup))
            .await
            .internal()?
            .internal()?
            .ok_or(Error::InvalidToken)?;
        let session = Session::acquire(self.dependencies.store.clone(), workspace_id, true).await?;
        let now = self.dependencies.clock.now();
        let mut record = session
            .call(move |session| session.accept_end(&hash, runner_id, now))
            .await?
            .ok_or(Error::InvalidToken)?;
        let snapshot = session.call(|session| session.snapshot()).await?;
        let account = snapshot
            .accounts
            .iter()
            .find(|account| account.account.id == record.key_id)
            .ok_or(Error::OwnershipUnknown)?;
        let backend = self.backend(workspace_id, account).await?;
        self.delete(&session, &backend, &mut record).await?;
        self.collect(&session).await
    }

    pub(super) async fn delete(
        &self,
        session: &Session,
        backend: &Capabilities,
        record: &mut StartRecord,
    ) -> Result<()> {
        let sandbox = ProviderRef::new(record.sandbox_id.as_ref().ok_or(Error::OwnershipUnknown)?);
        let measured = match self.measure(session, backend, record).await {
            Ok(measured) => measured,
            Err(_) => {
                tracing::warn!("Boat runtime observation failed before durable sandbox deletion");
                false
            }
        };
        backend
            .lifecycle
            .destroy_sandbox(sandbox)
            .await
            .internal()?;
        let now = self.dependencies.clock.now();
        if measured {
            spend::extend(record, now)?;
        } else {
            record.runtime_complete = record
                .runtime
                .iter()
                .max_by_key(|charge| charge.until)
                .is_some_and(|charge| !charge.running);
        }
        record.phase = StartPhase::Ended;
        record.ends_at = Some(now);
        let changed = record.clone();
        session
            .call(move |session| session.save_start(&changed))
            .await
    }

    pub(super) async fn cleanup_workspace(&self, workspace_id: Uuid) -> Result<()> {
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let snapshot = session.call(|session| session.snapshot()).await?;
        let mut failed = false;
        for account in &snapshot.accounts {
            let backend = match self.backend(workspace_id, account).await {
                Ok(backend) => backend,
                Err(_) => {
                    failed = true;
                    continue;
                }
            };
            let deployment = format!("{}/{}", self.config.deployment, workspace_id);
            let inventory = match backend.lifecycle.list_managed_sandboxes(deployment).await {
                Ok(inventory) => inventory,
                Err(_) => {
                    failed = true;
                    Vec::new()
                }
            };
            for mut record in snapshot
                .starts
                .iter()
                .filter(|record| record.key_id == account.account.id && record.ends_at.is_none())
                .cloned()
            {
                if matches!(record.phase, StartPhase::Deleting | StartPhase::Writing) {
                    record.phase = StartPhase::Deleting;
                    let changed = record.clone();
                    session
                        .call(move |session| session.save_start(&changed))
                        .await?;
                    if self.delete(&session, &backend, &mut record).await.is_err() {
                        failed = true;
                    }
                    continue;
                }
                if record.phase == StartPhase::Creating {
                    let recovered = backend
                        .lifecycle
                        .recover_sandbox_create(create_request(workspace_id, &record))
                        .await;
                    let sandbox = match recovered {
                        Ok(Some(sandbox)) => sandbox,
                        Ok(None) => continue,
                        Err(_) => {
                            failed = true;
                            continue;
                        }
                    };
                    record.sandbox_id = Some(sandbox.provider_ref.as_str().to_owned());
                    record.phase = StartPhase::Preparing;
                }
                if let Some(id) = record.sandbox_id.as_ref().filter(|id| {
                    !inventory
                        .iter()
                        .any(|sandbox| sandbox.provider_ref.as_str() == *id)
                }) {
                    match backend
                        .lifecycle
                        .inspect_sandbox(ProviderRef::new(id))
                        .await
                    {
                        Err(sandbox_interface::Error::NotFound { .. })
                        | Ok(sandbox_interface::BackendSandbox {
                            state: SandboxState::Destroyed,
                            ..
                        }) => {
                            record.phase = StartPhase::Ended;
                            record.ends_at = Some(self.dependencies.clock.now());
                            record.runtime_complete = record
                                .runtime
                                .iter()
                                .max_by_key(|charge| charge.until)
                                .is_some_and(|charge| !charge.running);
                            let changed = record.clone();
                            session
                                .call(move |session| session.save_start(&changed))
                                .await?;
                            continue;
                        }
                        Ok(_) => {}
                        Err(_) => {
                            failed = true;
                            continue;
                        }
                    }
                }
                match self
                    .dependencies
                    .runners
                    .retire(workspace_id, record.organization_id, record.runner_id)
                    .await
                {
                    Ok(Retirement::Removed) => {}
                    Ok(Retirement::RunningJob | Retirement::WaitingDemand) => continue,
                    Err(_) => {
                        failed = true;
                        continue;
                    }
                }
                record.phase = StartPhase::Deleting;
                let changed = record.clone();
                session
                    .call(move |session| session.save_start(&changed))
                    .await?;
                if self.delete(&session, &backend, &mut record).await.is_err() {
                    failed = true;
                }
            }
        }
        self.collect(&session).await?;
        if failed {
            return Err(Error::ProviderUnavailable);
        }
        Ok(())
    }
}
