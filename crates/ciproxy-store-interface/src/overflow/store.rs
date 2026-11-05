//! Exclusive workspace sessions for recoverable external sandbox operations.

use chrono::{DateTime, Utc};
use unimock::unimock;
use uuid::Uuid;

use crate::boat::BoatAccount;
use crate::error::Result;
use crate::overflow::account::AccountState;
use crate::overflow::record::{EncryptedLaunch, StartRecord};
use crate::secrets::{EncryptedBoatKey, TokenHash};

/// A retained key and its credential-free account observations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountRecord {
    /// Safe customer-visible account metadata.
    pub account: BoatAccount,
    /// KMS ciphertext, never a plaintext API key.
    pub key: EncryptedBoatKey,
    /// Durable capacity and refusal facts.
    pub state: AccountState,
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
    /// Positive monthly USD limit in millionths of a dollar.
    pub spend_limit_usd_micros: Option<i64>,
}

/// Durable persistence used only by the hosted overflow service.
#[unimock(api = OverflowStoreMock)]
pub trait OverflowStore: Send + Sync {
    /// Acquire one live workspace session. Closing it releases the lock.
    fn try_lock_overflow(&self, workspace_id: Uuid) -> Result<Option<Box<dyn OverflowSession>>>;
    /// Wait for the workspace session so a guest callback is never lost as busy.
    fn lock_overflow(&self, workspace_id: Uuid) -> Result<Box<dyn OverflowSession>>;
    /// Derive workspace authority from a callback digest, including consumed tokens.
    fn overflow_token_workspace(&self, hash: &TokenHash) -> Result<Option<Uuid>>;
}

/// Queries on the dedicated connection that owns the workspace session lock.
#[unimock(api = OverflowSessionMock)]
pub trait OverflowSession: Send {
    /// Read account, settings, and durable start facts from one snapshot.
    fn snapshot(&mut self) -> Result<OverflowSnapshot>;
    /// Read current startup authority for this workspace's saved delivery in one query.
    /// Require its active organization, enabled overflow, and accepted non-retired key.
    fn startup_account(&mut self, delivery_id: Uuid) -> Result<Option<AccountRecord>>;
    /// Atomically replace the active key, its catalog, and the monthly limit.
    fn connect(&mut self, account: &AccountRecord, spend_limit_usd_micros: i64) -> Result<()>;
    /// Retire active keys and preserve keys referenced by unfinished deliveries.
    fn disconnect(&mut self, at: DateTime<Utc>) -> Result<()>;
    /// Persist refreshed capacity or a refusal for one retained key.
    fn save_account(&mut self, account: &AccountRecord) -> Result<()>;
    /// Commit a definitive refusal and its account restriction atomically.
    fn save_refusal(&mut self, account: &AccountRecord, record: &StartRecord) -> Result<()>;
    /// Reserve a new delivery before dispatch and recheck current tenant authority.
    /// Return false if settings, key, or organization no longer permits a start.
    fn reserve_start(
        &mut self,
        record: &StartRecord,
        encrypted_launch: &EncryptedLaunch,
        token_hash: &TokenHash,
    ) -> Result<bool>;
    /// Read the sealed registration only while startup or cleanup needs it.
    fn encrypted_launch(&mut self, delivery_id: Uuid) -> Result<Option<EncryptedLaunch>>;
    /// Save phase and provider identity, and atomically materialize runner/cost facts.
    fn save_start(&mut self, record: &StartRecord) -> Result<()>;
    /// Consume one live sandbox token and request deletion before provider access.
    fn accept_end(
        &mut self,
        hash: &TokenHash,
        runner_id: Uuid,
        at: DateTime<Utc>,
    ) -> Result<Option<StartRecord>>;
    /// Erase unused retired keys and completed operation data older than the cutoff.
    fn collect(&mut self, before: DateTime<Utc>) -> Result<()>;
}
