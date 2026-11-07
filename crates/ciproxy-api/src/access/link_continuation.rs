//! Authorization proof and completion record durable pending results in one collector.

use ciproxy_github::app::webhook::AccountKind;
use ciproxy_store_interface::dashboard::records::Membership;
use ciproxy_store_interface::installation_links::InstallationClaim;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use uuid::Uuid;

use crate::error::Result;
use crate::identity::{Grant, Profile};

use super::AccessService;

/// Select durable authorization collection independently from dashboard proof refresh.
#[derive(Default)]
pub(super) struct ProofResults {
    delivery: InstallationResultDelivery,
}

impl ProofResults {
    /// Retain first and open results until a successful authorization saves them.
    pub(super) fn authorization() -> Self {
        Self {
            delivery: InstallationResultDelivery::Pending,
        }
    }

    /// Dashboard operations do not create authorization results.
    pub(super) fn delivery(&self) -> InstallationResultDelivery {
        self.delivery
    }
}

impl AccessService {
    /// Save proof and its pending canonical result in the same transaction.
    pub(super) async fn save_claim_proof(
        &self,
        mut claim: InstallationClaim,
        attempt: Option<Uuid>,
        results: &ProofResults,
    ) -> Result<InstallationClaim> {
        claim.user_verified = true;
        claim.verified_at = Some(self.runtime.now());
        self.save_claim(claim, attempt, results.delivery()).await
    }

    /// Record first own-user proof without changing completed Direct authority.
    pub(super) async fn prove_completed_links(
        &self,
        profile: &Profile,
        grant: &Grant,
        memberships: &[Membership],
        results: &ProofResults,
    ) -> Result<()> {
        for claim in self.owned_claims(profile.id).await? {
            if !claim.direct
                || !claim.event_seen
                || claim.organization_id.is_none()
                || claim.user_verified
                || !memberships
                    .iter()
                    .any(|membership| membership.github_id == claim.github_id)
            {
                continue;
            }
            let Ok(Some(actual)) = self
                .identity
                .installation(grant, claim.installation_id)
                .await
            else {
                continue;
            };
            if actual.id != claim.installation_id
                || actual.account.id != claim.github_id
                || actual.account.kind != AccountKind::Organization
            {
                continue;
            }
            self.save_claim_proof(claim, None, results).await?;
        }
        Ok(())
    }
}
