//! Exclusive workspace sessions for recoverable external sandbox operations.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use unimock::unimock;
use uuid::Uuid;

use crate::error::Result;
use crate::overflow::account::AccountState;
use crate::overflow::admission::{DispatchFacts, PreparedDispatch, ReservationOutcome};
use crate::overflow::connection::ConnectionGrant;
use crate::overflow::provider::OverflowProvider;
use crate::overflow::record::{EncryptedLaunch, StartRecord};
use crate::overflow_provider_accounts::OverflowProviderAccount;
use crate::overflow_provider_settings::OverflowProviderSettings;
use crate::secrets::{EncryptedOverflowKey, TokenHash};

/// A retained key and its credential-free account observations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountRecord {
    /// Safe customer-visible account metadata.
    pub account: OverflowProviderAccount,
    /// KMS ciphertext, never a plaintext API key.
    pub key: EncryptedOverflowKey,
    /// Durable capacity and refusal facts.
    pub state: AccountState,
}

impl AccountRecord {
    /// Whether this is the key version saved for the delivery's provider.
    pub fn owns_start(&self, record: &StartRecord) -> bool {
        self.account.id == record.key_id && self.account.provider == record.provider
    }
}

/// One coherent workspace view for spend and execution recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverflowSnapshot {
    /// Retained keys, including disconnected keys needed for cleanup.
    pub accounts: Vec<AccountRecord>,
    /// Retained ciproxy starts across every organization.
    pub starts: Vec<StartRecord>,
    /// Whether workspace settings permit new starts.
    pub enabled: bool,
    /// Saved spend limits and offered marks by provider, including disconnected providers.
    pub provider_settings: HashMap<OverflowProvider, OverflowProviderSettings>,
}

impl OverflowSnapshot {
    /// Read the current key version of one provider, including a rejected key.
    pub fn active_account(&self, provider: OverflowProvider) -> Option<&AccountRecord> {
        self.accounts.iter().find(|record| {
            record.account.provider == provider && record.account.retired_at.is_none()
        })
    }
}

/// Durable persistence used only by the hosted overflow service.
#[unimock(api = OverflowStoreMock)]
pub trait OverflowStore: Send + Sync {
    /// Acquire one live workspace session. Closing it releases the lock.
    fn try_lock_overflow(&self, workspace_id: Uuid) -> Result<Option<Box<dyn OverflowSession>>>;
    /// Wait within the store's lock limit for a workspace session.
    fn lock_overflow(&self, workspace_id: Uuid) -> Result<Box<dyn OverflowSession>>;
    /// Derive workspace authority from a callback digest, including consumed tokens.
    fn overflow_token_workspace(&self, hash: &TokenHash) -> Result<Option<Uuid>>;
    /// Consume one live sandbox token and request deletion without a workspace session.
    fn accept_end(
        &self,
        workspace_id: Uuid,
        hash: &TokenHash,
        runner_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<Option<StartRecord>>;
}

/// Queries on the dedicated connection that owns the workspace session lock.
#[unimock(api = OverflowSessionMock)]
pub trait OverflowSession: Send {
    /// Read account, settings, and durable start facts from one snapshot.
    fn snapshot(&mut self) -> Result<OverflowSnapshot>;
    /// Read current startup authority for this workspace's saved delivery in one query.
    /// Require its active organization, enabled overflow, accepted key, and current label size.
    fn startup_account(&mut self, delivery_id: Uuid) -> Result<Option<AccountRecord>>;
    /// Read the current label for one authorized pending demand.
    fn dispatch_facts(&mut self, delivery_id: Uuid) -> Result<Option<DispatchFacts>>;
    /// Persist a fresh pending grant while at least one organization has an App.
    /// Supersede any previous pending grant without restoring a revoked identity.
    fn begin_connection(&mut self, key_id: Uuid) -> Result<Option<ConnectionGrant>>;
    /// Consume a live grant and replace the active key, catalog, and monthly limit.
    /// Return false when last-App removal or a newer operation invalidated the grant.
    fn complete_connection(
        &mut self,
        grant: ConnectionGrant,
        account: &AccountRecord,
        spend_limit_usd_micros: i64,
    ) -> Result<bool>;
    /// Retire this provider's active key and preserve keys for unfinished deliveries.
    fn disconnect(&mut self, provider: OverflowProvider, at: DateTime<Utc>) -> Result<()>;
    /// Persist refreshed capacity or a refusal for one retained key.
    fn save_account(&mut self, account: &AccountRecord) -> Result<()>;
    /// Commit a definitive refusal and its account restriction atomically.
    fn save_refusal(&mut self, account: &AccountRecord, record: &StartRecord) -> Result<()>;
    /// Save authorized pending demand and sealed input without provider parameters or costs.
    fn save_pending(
        &mut self,
        record: &StartRecord,
        encrypted_launch: &EncryptedLaunch,
        token_hash: &TokenHash,
    ) -> Result<bool>;
    /// Commit Creating only for an unsent delivery with current authority and budget.
    /// Preserve every pending field and fix the selected provider plan atomically.
    fn begin_dispatch(
        &mut self,
        pending: &StartRecord,
        dispatch: &PreparedDispatch,
    ) -> Result<ReservationOutcome>;
    /// Read the sealed registration only while startup or cleanup needs it.
    fn encrypted_launch(&mut self, delivery_id: Uuid) -> Result<Option<EncryptedLaunch>>;
    /// Save phase and provider identity, and atomically materialize runner/cost facts.
    /// Keep the first report and end, and do not regress deletion.
    fn save_start(&mut self, record: &StartRecord) -> Result<()>;
    /// Erase unused retired keys and completed operation data older than the cutoff.
    fn collect(&mut self, before: DateTime<Utc>) -> Result<()>;
}
