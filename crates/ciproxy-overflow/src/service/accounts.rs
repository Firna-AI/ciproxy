//! Account lifecycle and the public object-safe service implementation.

use async_trait::async_trait;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::boat::BoatAccount;
use ciproxy_store_interface::overflow::account::AccountState;
use ciproxy_store_interface::overflow::store::AccountRecord;
use ciproxy_store_interface::secrets::EncryptedBoatKey;
use uuid::Uuid;

use crate::api::{Overflow, StartOutcome, StartRequest};
use crate::catalog;
use crate::crypto::{KeyContext, SecretPurpose};
use crate::error::{Error, Result, ResultExt};
use crate::provider::Capabilities;
use crate::secrets::SecretBytes;
use crate::session::Session;
use crate::view::OverflowView;

use super::OverflowService;

#[async_trait]
impl Overflow for OverflowService {
    async fn connect(
        &self,
        workspace_id: Uuid,
        key_id: Uuid,
        key: Secret,
        wallet: String,
        limit: i64,
    ) -> Result<BoatAccount> {
        if limit <= 0 {
            return Err(Error::InvalidSpendLimit);
        }
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let backend = self.dependencies.factory.backend(&key, &wallet)?;
        let facts = backend.account.account().await?;
        let sizes = catalog::offers(backend.capacity.account_capacity().await.internal()?)?;
        let now = self.dependencies.clock.now();
        let bytes = self
            .dependencies
            .cipher
            .seal(
                KeyContext {
                    workspace_id,
                    id: key_id,
                    purpose: SecretPurpose::BoatKey,
                },
                SecretBytes::new(key.expose_secret().as_bytes().to_vec()),
            )
            .await?;
        let metadata = BoatAccount {
            id: key_id,
            account_name: facts.name,
            connected_at: now,
            retired_at: None,
            rejected_at: None,
        };
        let account = AccountRecord {
            account: metadata.clone(),
            key: EncryptedBoatKey::new(bytes),
            state: AccountState {
                wallet: facts.wallet,
                checked_at: Some(now),
                sizes,
                trial: facts.trial,
                billing_required: false,
                retry_at: None,
                refused_sizes: Vec::new(),
            },
        };
        session
            .call(move |session| session.connect(&account, limit))
            .await?;
        self.collect(&session).await?;
        Ok(metadata)
    }

    async fn disconnect(&self, workspace_id: Uuid) -> Result<()> {
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let now = self.dependencies.clock.now();
        session.call(move |session| session.disconnect(now)).await?;
        self.collect(&session).await
    }

    async fn view(&self, workspace_id: Uuid) -> Result<OverflowView> {
        self.read_view(workspace_id).await
    }

    async fn start(&self, workspace_id: Uuid, request: StartRequest) -> Result<StartOutcome> {
        self.start_request(workspace_id, request).await
    }

    async fn resume(&self, workspace_id: Uuid, delivery_id: Uuid) -> Result<StartOutcome> {
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let snapshot = session.call(|session| session.snapshot()).await?;
        let record = snapshot
            .starts
            .into_iter()
            .find(|record| record.delivery_id == delivery_id)
            .ok_or(Error::DeliveryConflict)?;
        self.execute(&session, workspace_id, record, false).await
    }

    async fn report_end(&self, token: Secret, runner_id: Uuid) -> Result<()> {
        self.end_report(token, runner_id).await
    }

    async fn cleanup(&self, workspace_id: Uuid) -> Result<()> {
        self.cleanup_workspace(workspace_id).await
    }
}

impl OverflowService {
    pub(super) async fn backend(
        &self,
        workspace_id: Uuid,
        account: &AccountRecord,
    ) -> Result<Capabilities> {
        let bytes = self
            .dependencies
            .cipher
            .open(
                KeyContext {
                    workspace_id,
                    id: account.account.id,
                    purpose: SecretPurpose::BoatKey,
                },
                account.key.as_bytes().to_vec(),
            )
            .await?;
        let key = Secret::new(std::str::from_utf8(bytes.expose()).internal()?);
        self.dependencies
            .factory
            .backend(&key, &account.state.wallet)
    }

    pub(super) async fn collect(&self, session: &Session) -> Result<()> {
        let cutoff = self.dependencies.clock.now() - chrono::Duration::days(90);
        session.call(move |session| session.collect(cutoff)).await
    }
}
