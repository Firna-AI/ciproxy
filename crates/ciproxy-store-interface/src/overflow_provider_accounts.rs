//! Overflow provider account metadata and encrypted key versions.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::overflow::provider::OverflowProvider;
use crate::secrets::EncryptedOverflowKey;

/// Safe metadata for one provider key version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverflowProviderAccount {
    /// The key version identifier referenced by its sandbox runners.
    pub id: Uuid,
    /// The provider that issued the key.
    pub provider: OverflowProvider,
    /// The account name accepted by the provider.
    pub account_name: String,
    /// When the customer connected this key.
    pub connected_at: DateTime<Utc>,
    /// When this key stopped serving new runners.
    pub retired_at: Option<DateTime<Utc>>,
    /// When the provider rejected this key.
    pub rejected_at: Option<DateTime<Utc>>,
}

/// Queries that retain retired keys while their sandboxes stay live.
pub trait OverflowProviderAccountStore: Send + Sync {
    /// Save ciphertext and a positive limit, retiring this provider's previous key.
    fn connect_overflow_provider(
        &self,
        workspace_id: Uuid,
        account: &OverflowProviderAccount,
        key: &EncryptedOverflowKey,
        spend_limit_usd_micros: i64,
    ) -> Result<()>;
    /// Read safe account metadata, including retained old key versions.
    fn overflow_provider_accounts(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OverflowProviderAccount>>;
    /// Read ciphertext for an workspace's specific key version.
    fn encrypted_overflow_key(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<EncryptedOverflowKey>>;
    /// Mark the active key as rejected without deleting live sandbox access.
    fn reject_overflow_key(&self, workspace_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Stop new runners from using this provider's active key.
    fn disconnect_overflow_provider(
        &self,
        workspace_id: Uuid,
        provider: OverflowProvider,
        at: DateTime<Utc>,
    ) -> Result<()>;
    /// Erase retired keys after their last live sandbox runner ends.
    fn delete_unused_overflow_keys(&self, workspace_id: Uuid) -> Result<usize>;
}
