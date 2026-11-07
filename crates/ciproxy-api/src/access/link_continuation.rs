//! Fresh browser proof distinguishes approved Direct installs from completed history.

use ciproxy_github::app::webhook::AccountKind;
use ciproxy_store_interface::dashboard::records::Membership;
use uuid::Uuid;

use crate::error::Result;
use crate::identity::{Grant, Profile};

use super::AccessService;

impl AccessService {
    /// Record first own-user proof without changing completed Direct authority.
    pub(super) async fn prove_completed_links(
        &self,
        profile: &Profile,
        grant: &Grant,
        memberships: &[Membership],
    ) -> Result<Vec<Uuid>> {
        let mut continuation = Vec::new();
        for mut claim in self.owned_claims(profile.id).await? {
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
            claim.user_verified = true;
            claim.verified_at = Some(self.runtime.now());
            let claim = self.save_claim(claim, None).await?;
            continuation.push(claim.id);
        }
        Ok(continuation)
    }
}
