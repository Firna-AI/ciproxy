//! Exact results collected where setup proof, open proof, or Direct completion occurs.

use std::collections::BTreeSet;

use ciproxy_github::app::webhook::AccountKind;
use ciproxy_store_interface::dashboard::records::Membership;
use ciproxy_store_interface::installation_links::InstallationClaim;
use uuid::Uuid;

use crate::error::Result;
use crate::identity::{Grant, Profile};

use super::AccessService;

/// One authorization's exact results, excluding previously proved completed claims.
#[derive(Default)]
pub(super) struct ProofResults {
    claims: BTreeSet<Uuid>,
}

impl ProofResults {
    /// Keep successful proof or completion unless the claim was already proved and completed.
    pub(super) fn record(&mut self, previous: Option<&InstallationClaim>, id: Uuid) {
        if previous.is_none_or(|claim| !claim.user_verified || claim.organization_id.is_none()) {
            self.claims.insert(id);
        }
    }

    /// Return every recorded canonical ID once in stable order.
    pub(super) fn into_claims(self) -> Vec<Uuid> {
        self.claims.into_iter().collect()
    }
}

impl AccessService {
    /// Preserve the before-state and record the canonical ID returned by the store.
    pub(super) async fn save_claim_proof(
        &self,
        mut claim: InstallationClaim,
        previous: Option<&InstallationClaim>,
        attempt: Option<Uuid>,
        results: &mut ProofResults,
    ) -> Result<InstallationClaim> {
        claim.user_verified = true;
        claim.verified_at = Some(self.runtime.now());
        let saved = self.save_claim(claim, attempt).await?;
        results.record(previous, saved.id);
        Ok(saved)
    }

    /// Record first own-user proof without changing completed Direct authority.
    pub(super) async fn prove_completed_links(
        &self,
        profile: &Profile,
        grant: &Grant,
        memberships: &[Membership],
        results: &mut ProofResults,
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
            let previous = claim.clone();
            self.save_claim_proof(claim, Some(&previous), None, results)
                .await?;
        }
        Ok(())
    }
}
