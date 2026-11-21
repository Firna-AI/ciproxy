//! Operation-specific liveness authority for a pending Boat connection.

use uuid::Uuid;

/// A fresh store-issued grant that last-App removal can revoke before activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionGrant {
    /// Opaque operation identity, independent of the caller's key-version identity.
    pub id: Uuid,
}
