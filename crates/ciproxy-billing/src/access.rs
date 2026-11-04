//! Authorization supplied by the authenticated dashboard boundary.

use uuid::Uuid;

use crate::error::{Error, Result};

/// A trusted, current membership role, resolved by the sign-in service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    /// Workspace administration, including payment.
    Admin,
    /// Payment and invoices without runner administration.
    Billing,
    /// Read-only access, including billing banners.
    Viewer,
}

/// Authenticated workspace authority, never accepted from request JSON.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BillingAccess {
    /// The workspace selected by the membership check.
    pub workspace_id: Uuid,
    /// The current role of the authenticated person.
    pub role: Role,
}

impl BillingAccess {
    pub(crate) fn require_payer(self) -> Result<Uuid> {
        match self.role {
            Role::Admin | Role::Billing => Ok(self.workspace_id),
            Role::Viewer => Err(Error::Forbidden),
        }
    }
}
