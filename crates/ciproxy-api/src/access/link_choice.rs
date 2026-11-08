//! Select one previously offered workspace with fresh installed proof.

use ciproxy_github::app::GitHubApp;
use ciproxy_github::app::webhook::AccountKind;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use uuid::Uuid;

use crate::error::{Error, Result, ServiceResultExt};
use crate::state::query;

use super::{AccessService, Principal};

impl AccessService {
    pub(super) async fn choose_link(
        &self,
        principal: &Principal,
        id: Uuid,
        workspace: Uuid,
        github: &dyn GitHubApp,
    ) -> Result<()> {
        if principal.browser_only() {
            return Err(Error::Unauthorized);
        }
        let person = principal.github_id();
        let (claim, offered, user) = query(self.store.clone(), move |store| {
            Ok((
                store.installation_claim(id)?,
                store.installation_workspace_offers(id, person)?,
                store.dashboard_user_by_github(person)?,
            ))
        })
        .await?;
        let mut claim = claim.ok_or(Error::NotFound)?;
        if claim.requester_id != person || !claim.event_seen || !offered.contains(&workspace) {
            return Err(Error::Forbidden);
        }
        if claim.workspace_id.is_some_and(|saved| saved != workspace) {
            return Err(Error::Conflict);
        }
        let (grant, memberships) = self
            .refresh_link_identity(user.ok_or(Error::Unauthorized)?)
            .await?;
        let profile = self.identity.profile(&grant).await?;
        if profile.id != person {
            return Err(Error::Forbidden);
        }
        let actual = self
            .identity
            .installation(&grant, claim.installation_id)
            .await?
            .ok_or(Error::Forbidden)?;
        if actual.id != claim.installation_id
            || actual.account.id != claim.github_id
            || actual.account.kind != AccountKind::Organization
            || !memberships
                .iter()
                .any(|member| member.github_id == claim.github_id)
        {
            return Err(Error::Forbidden);
        }
        claim.user_verified = true;
        claim.verified_at = Some(self.runtime.now());
        self.save_claim(claim, None, InstallationResultDelivery::Untracked)
            .await?;
        match query(self.store.clone(), move |store| {
            store.choose_installation_workspace(id, person, workspace)
        })
        .await
        {
            Ok(()) => {}
            Err(Error::NotFound | Error::Conflict) => return Err(Error::Conflict),
            Err(error) => return Err(error),
        }
        github.resolve_installation_link(id).await.api()?;
        Ok(())
    }
}
