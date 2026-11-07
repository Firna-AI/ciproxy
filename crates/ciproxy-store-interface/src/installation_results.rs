//! Durable authorization results, separate from installation proof and authority.

use uuid::Uuid;

/// Whether a claim save also records an authorization result for later delivery.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InstallationResultDelivery {
    /// Dashboard proof or bookkeeping creates no authorization result.
    #[default]
    Untracked,
    /// Keep the first or open claim result until a successful session saves it.
    Pending,
}

/// One exact pending fact observed before preparing a browser session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingInstallationResult {
    /// Canonical installation claim identity.
    pub claim_id: Uuid,
    /// Database-issued revision which prevents consumption of a later write.
    pub revision: i64,
}
