//! One-use end callbacks, marked idle cleanup, and durable delete retries.

use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::{ProviderRef, SandboxState};
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::api::{Retirement, StartOutcome};
use crate::cost_state;
use crate::error::{Error, Result, ResultExt};
use crate::maintenance::MaintenanceCapabilities;
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
        let backend = self.maintenance_backend(workspace_id, account).await?;
        self.delete(&session, &backend, &mut record).await?;
        self.collect(&session).await
    }

    pub(super) async fn delete(
        &self,
        session: &Session,
        backend: &MaintenanceCapabilities,
        record: &mut StartRecord,
    ) -> Result<()> {
        let sandbox =
            ProviderRef::new(record.sandbox.as_ref().ok_or(Error::OwnershipUnknown)?.id());
        let measured = match self.measure(session, backend, record).await {
            Ok(measured) => measured,
            Err(_) => {
                tracing::warn!("Boat runtime observation failed before durable sandbox deletion");
                false
            }
        };
        let destroyed = backend.destroy_sandbox(sandbox).await;
        if destroyed.is_err() {
            let still_billing = measured
                && record
                    .runtime
                    .iter()
                    .max_by_key(|charge| charge.until)
                    .is_some_and(|charge| charge.running);
            if !still_billing && let Some(bound) = self.proven_lifetime_end(record) {
                return self.end_record(session, record, bound).await;
            }
            destroyed.internal()?;
        }
        let now = self.dependencies.clock.now();
        if measured {
            spend::extend(record, now)?;
        } else {
            cost_state::missing_observation(record);
        }
        self.end_record(session, record, now).await
    }

    pub(super) async fn request_delete(
        &self,
        session: &Session,
        backend: &MaintenanceCapabilities,
        record: &mut StartRecord,
    ) -> Result<()> {
        record.phase = StartPhase::Deleting;
        let changed = record.clone();
        session
            .call(move |session| session.save_start(&changed))
            .await?;
        self.delete(session, backend, record).await
    }

    pub(super) async fn cleanup_workspace(&self, workspace_id: Uuid) -> Result<()> {
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let snapshot = session.call(|session| session.snapshot()).await?;
        let mut failed = false;
        for account in &snapshot.accounts {
            let backend = match self.maintenance_backend(workspace_id, account).await {
                Ok(backend) => backend,
                Err(_) => {
                    for mut record in snapshot
                        .starts
                        .iter()
                        .filter(|record| {
                            record.key_id == account.account.id && record.ends_at.is_none()
                        })
                        .cloned()
                    {
                        if !self.unavailable_lifecycle(&session, &mut record).await? {
                            failed = true;
                        }
                    }
                    continue;
                }
            };
            let deployment = format!("{}/{}", self.config.deployment, workspace_id);
            let inventory = match backend.list_managed_sandboxes(deployment).await {
                Ok(inventory) => inventory,
                Err(_) => {
                    tracing::warn!(
                        "Boat inventory is unavailable; known lifecycle records will be reconciled"
                    );
                    Vec::new()
                }
            };
            for mut record in snapshot
                .starts
                .iter()
                .filter(|record| record.key_id == account.account.id && record.ends_at.is_none())
                .cloned()
            {
                let delivery = record.delivery_id;
                if !matches!(record.phase, StartPhase::Running | StartPhase::Deleting)
                    && session
                        .call(move |session| session.startup_account(delivery))
                        .await?
                        .is_none()
                {
                    match self
                        .reconcile_retired(&session, workspace_id, &backend, record)
                        .await
                    {
                        Ok(StartOutcome::Pending(record)) => {
                            failed |= record.phase != StartPhase::Creating;
                        }
                        Ok(_) => {}
                        Err(_) => failed = true,
                    }
                    continue;
                }
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
                let mut observed_state = None;
                if record.phase == StartPhase::Creating {
                    let recovered = backend
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
                    observed_state = Some(self.adopt(&session, &mut record, sandbox).await?);
                }
                let listed = record.sandbox.as_ref().and_then(|known| {
                    inventory
                        .iter()
                        .find(|sandbox| sandbox.provider_ref.as_str() == known.id())
                });
                let observed_state = observed_state.or_else(|| listed.map(|sandbox| sandbox.state));
                if matches!(
                    observed_state,
                    Some(SandboxState::Destroyed | SandboxState::Failed | SandboxState::Destroying)
                ) {
                    if self
                        .request_delete(&session, &backend, &mut record)
                        .await
                        .is_err()
                    {
                        failed = true;
                    }
                    continue;
                }
                if let Some(known) = record.sandbox.as_ref().filter(|known| {
                    !inventory
                        .iter()
                        .any(|sandbox| sandbox.provider_ref.as_str() == known.id())
                }) {
                    match backend.inspect_sandbox(ProviderRef::new(known.id())).await {
                        Err(sandbox_interface::Error::NotFound { .. })
                        | Ok(sandbox_interface::BackendSandbox {
                            state: SandboxState::Destroyed,
                            ..
                        }) => {
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
                        Ok(_) => {}
                        Err(_) => {
                            if self.proven_lifetime_end(&record).is_some() {
                                record.phase = StartPhase::Deleting;
                                let changed = record.clone();
                                session
                                    .call(move |session| session.save_start(&changed))
                                    .await?;
                                if self.delete(&session, &backend, &mut record).await.is_err() {
                                    failed = true;
                                }
                            } else {
                                failed = true;
                            }
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
