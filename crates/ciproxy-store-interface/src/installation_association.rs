//! Signed installation identity owns flow, independently of browser proof.

use uuid::Uuid;

use crate::error::{Error, Result};
use crate::installation_links::InstallationClaim;

/// Current installation identity from one authenticated GitHub creation event.
#[derive(Clone, Debug)]
pub struct SignedInstallation {
    /// Fresh recovery identity when no provisional claim exists.
    pub id: Uuid,
    /// Exact installation verified with the App identity.
    pub installation_id: i64,
    /// Exact organization account.
    pub github_id: i64,
    /// Current authoritative organization name.
    pub login: String,
    /// Original requester from the signed event.
    pub requester_id: i64,
}

impl SignedInstallation {
    /// Join signed identity with only this requester's own installation evidence.
    pub fn associate(
        &self,
        existing: Option<InstallationClaim>,
        has_add_evidence: bool,
    ) -> Result<InstallationClaim> {
        let mut claim = InstallationClaim {
            id: self.id,
            installation_id: self.installation_id,
            github_id: self.github_id,
            login: self.login.clone(),
            requester_id: self.requester_id,
            event_seen: true,
            user_verified: false,
            direct: !has_add_evidence,
            workspace_id: None,
            organization_id: None,
            verified_at: None,
        };
        if let Some(existing) = existing {
            if existing.installation_id != self.installation_id
                || existing.github_id != self.github_id
                || (existing.event_seen && existing.requester_id != self.requester_id)
            {
                return Err(Error::Conflict);
            }
            claim.id = existing.id;
            if existing.requester_id == self.requester_id {
                claim.user_verified = existing.user_verified;
                claim.verified_at = existing.verified_at;
                if existing.event_seen {
                    claim.direct = existing.direct;
                    claim.workspace_id = existing.workspace_id;
                    claim.organization_id = existing.organization_id;
                }
            }
        }
        Ok(claim)
    }
}
