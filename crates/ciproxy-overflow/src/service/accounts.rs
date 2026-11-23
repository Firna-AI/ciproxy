//! Account lifecycle and the public object-safe service implementation.

use async_trait::async_trait;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::account::AccountState;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow::store::AccountRecord;
use ciproxy_store_interface::overflow_provider_accounts::OverflowProviderAccount;
use ciproxy_store_interface::secrets::EncryptedOverflowKey;
use uuid::Uuid;

use crate::api::{Overflow, StartOutcome, StartRequest};
use crate::catalog;
use crate::crypto::{KeyContext, SecretPurpose};
use crate::error::{Error, Result, ResultExt};
use crate::maintenance::MaintenanceCapabilities;
use crate::provider::{Capabilities, KeyAccounts};
use crate::secrets::SecretBytes;
use crate::session::Session;
use crate::view::OverflowView;

use super::OverflowService;

#[async_trait]
impl Overflow for OverflowService {
    async fn check_key(&self, provider: OverflowProvider, key: Secret) -> Result<KeyAccounts> {
        self.dependencies.providers.start_image(provider)?;
        self.dependencies
            .providers
            .get(provider)?
            .factory
            .for_key_check(&key)?
            .check_key()
            .await
    }

    async fn connect(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
        key_id: Uuid,
        key: Secret,
        billing_account: String,
        limit: i64,
    ) -> Result<OverflowProviderAccount> {
        if limit <= 0 {
            return Err(Error::InvalidSpendLimit);
        }
        self.dependencies.providers.start_image(provider)?;
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let grant = session
            .call(move |session| session.begin_connection(key_id))
            .await?
            .ok_or(Error::ConnectionNotPermitted)?;
        let capabilities = self
            .dependencies
            .providers
            .get(provider)?
            .factory
            .capabilities(&key, &billing_account)?;
        let facts = capabilities.account.account().await?;
        let sizes = catalog::offers(capabilities.capacity.account_capacity().await.internal()?)?;
        let now = self.dependencies.clock.now();
        let bytes = self
            .dependencies
            .cipher
            .seal(
                KeyContext {
                    workspace_id,
                    id: key_id,
                    purpose: SecretPurpose::provider_key(provider),
                },
                SecretBytes::new(key.expose_secret().as_bytes().to_vec()),
            )
            .await?;
        let metadata = OverflowProviderAccount {
            id: key_id,
            provider,
            account_name: facts.name,
            connected_at: now,
            retired_at: None,
            rejected_at: None,
        };
        let account = AccountRecord {
            account: metadata.clone(),
            key: EncryptedOverflowKey::new(bytes),
            state: AccountState {
                provider: metadata.provider,
                wallet: facts.billing_account,
                checked_at: Some(now),
                sizes,
                trial: facts.trial,
                start_fee_bound_usd: facts.start_fee_bound_usd,
                billing_required: false,
                retry_at: None,
                refused_sizes: Vec::new(),
            },
        };
        let connected = session
            .call(move |session| session.complete_connection(grant, &account, limit))
            .await?;
        if !connected {
            return Err(Error::ConnectionNotPermitted);
        }
        self.cancel_withdrawn_unsent(&session, provider).await?;
        self.collect(&session).await?;
        Ok(metadata)
    }

    async fn disconnect(&self, workspace_id: Uuid, provider: OverflowProvider) -> Result<()> {
        let session =
            Session::acquire(self.dependencies.store.clone(), workspace_id, false).await?;
        let now = self.dependencies.clock.now();
        session
            .call(move |session| session.disconnect(provider, now))
            .await?;
        self.cancel_withdrawn_unsent(&session, provider).await?;
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
        self.execute(&session, workspace_id, record).await
    }

    async fn report_end(&self, token: Secret, runner_id: Uuid) -> Result<()> {
        self.end_report(token, runner_id).await
    }

    async fn cleanup(&self, workspace_id: Uuid) -> Result<()> {
        self.cleanup_workspace(workspace_id).await
    }
}

impl OverflowService {
    pub(super) async fn capabilities(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
        account: &AccountRecord,
    ) -> Result<Capabilities> {
        if account.account.retired_at.is_some() || account.account.rejected_at.is_some() {
            return Err(Error::RejectedKey);
        }
        let factory = &self.dependencies.providers.get(provider)?.factory;
        let key = self.account_key(workspace_id, account).await?;
        factory.capabilities(&key, &account.state.wallet)
    }

    pub(super) async fn maintenance(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
        account: &AccountRecord,
    ) -> Result<MaintenanceCapabilities> {
        let factory = &self.dependencies.providers.get(provider)?.factory;
        let key = self.account_key(workspace_id, account).await?;
        factory.maintenance(&key, &account.state.wallet)
    }

    async fn account_key(&self, workspace_id: Uuid, account: &AccountRecord) -> Result<Secret> {
        let bytes = self
            .dependencies
            .cipher
            .open(
                KeyContext {
                    workspace_id,
                    id: account.account.id,
                    purpose: SecretPurpose::provider_key(account.account.provider),
                },
                account.key.as_bytes().to_vec(),
            )
            .await?;
        Ok(Secret::new(std::str::from_utf8(bytes.expose()).internal()?))
    }

    pub(super) async fn collect(&self, session: &Session) -> Result<()> {
        let cutoff = self.dependencies.clock.now() - chrono::Duration::days(90);
        session.call(move |session| session.collect(cutoff)).await
    }
}
