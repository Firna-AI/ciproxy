//! One-use end callbacks, marked idle cleanup, and durable delete retries.

use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use sandbox_interface::ProviderRef;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::cost_state;
use crate::error::{Error, Result, ResultExt};
use crate::maintenance::MaintenanceCapabilities;
use crate::secrets::digest;
use crate::session::Session;
use crate::spend;

use super::OverflowService;

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
        let store = self.dependencies.store.clone();
        let now = self.dependencies.clock.now();
        let record = spawn_blocking(move || store.accept_end(workspace_id, &hash, runner_id, now))
            .await
            .internal()?
            .internal()?
            .ok_or(Error::InvalidToken)?;
        let delivery_id = record.delivery_id;
        if let Err(error) = self.delete_reported(workspace_id, delivery_id).await {
            tracing::warn!(%workspace_id, %delivery_id, %error, "Exit report saved. Cleanup deferred.");
        }
        Ok(())
    }

    async fn delete_reported(&self, workspace_id: Uuid, delivery_id: Uuid) -> Result<()> {
        let session = Session::acquire(self.dependencies.store.clone(), workspace_id, true).await?;
        let snapshot = session.call(|session| session.snapshot()).await?;
        let Some(mut record) = snapshot
            .starts
            .into_iter()
            .find(|record| record.delivery_id == delivery_id && record.ends_at.is_none())
        else {
            return Ok(());
        };
        let account = snapshot
            .accounts
            .iter()
            .find(|account| account.account.id == record.key_id)
            .ok_or(Error::OwnershipUnknown)?;
        let maintenance = self.maintenance(workspace_id, account).await?;
        self.delete(&session, &maintenance, &mut record).await?;
        self.collect(&session).await
    }

    pub(super) async fn delete(
        &self,
        session: &Session,
        maintenance: &MaintenanceCapabilities,
        record: &mut StartRecord,
    ) -> Result<()> {
        let sandbox =
            ProviderRef::new(record.sandbox.as_ref().ok_or(Error::OwnershipUnknown)?.id());
        let measured = match self.measure(session, maintenance, record).await {
            Ok(measured) => measured,
            Err(_) => {
                tracing::warn!(
                    provider = record.provider.as_str(),
                    "Runtime observation failed before durable sandbox deletion"
                );
                false
            }
        };
        let destroyed = maintenance.destroy_sandbox(sandbox).await;
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
        maintenance: &MaintenanceCapabilities,
        record: &mut StartRecord,
    ) -> Result<()> {
        record.phase = StartPhase::Deleting;
        let changed = record.clone();
        session
            .call(move |session| session.save_start(&changed))
            .await?;
        self.delete(session, maintenance, record).await
    }
}
