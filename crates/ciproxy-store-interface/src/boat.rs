//! Boat account metadata and encrypted key versions.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::secrets::EncryptedBoatKey;

/// Safe metadata for one Boat key version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatAccount {
    /// The key version identifier referenced by its sandbox runners.
    pub id: Uuid,
    /// The account name accepted by Boat.
    pub account_name: String,
    /// When the customer connected this key.
    pub connected_at: DateTime<Utc>,
    /// When this key stopped serving new runners.
    pub retired_at: Option<DateTime<Utc>>,
    /// When Boat rejected this key.
    pub rejected_at: Option<DateTime<Utc>>,
}

/// Queries that retain retired keys while their sandboxes stay live.
pub trait BoatStore: Send + Sync {
    /// Save ciphertext and a positive spend limit, retiring the previous key.
    fn connect_boat(
        &self,
        organization_id: Uuid,
        account: &BoatAccount,
        key: &EncryptedBoatKey,
        spend_limit_usd_micros: i64,
    ) -> Result<()>;
    /// Read safe account metadata, including retained old key versions.
    fn boat_accounts(&self, organization_id: Uuid) -> Result<Vec<BoatAccount>>;
    /// Read ciphertext for an organization's specific key version.
    fn encrypted_boat_key(
        &self,
        organization_id: Uuid,
        id: Uuid,
    ) -> Result<Option<EncryptedBoatKey>>;
    /// Mark the active key as rejected without deleting live sandbox access.
    fn reject_boat_key(&self, organization_id: Uuid, id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Stop new runners from using every currently active key.
    fn disconnect_boat(&self, organization_id: Uuid, at: DateTime<Utc>) -> Result<()>;
    /// Erase retired keys after their last live sandbox runner ends.
    fn delete_unused_boat_keys(&self, organization_id: Uuid) -> Result<usize>;
}
