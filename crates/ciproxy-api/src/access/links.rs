//! Fresh own-user proof, exact eligible offers, and credential-free recovery.

use ciproxy_github::app::GitHubApp;
use ciproxy_github::app::webhook::AccountKind;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::installation_links::{
    InstallationAttempt, InstallationClaim, InstallationRecovery,
};
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::store::Store;
use uuid::Uuid;

use crate::error::{Error, Result, ServiceResultExt};
use crate::identity::{Grant, Profile};
use crate::state::query;

use super::authentication::digest;
use super::link_continuation::ProofResults;
use super::setup::SetupTarget;
use super::{AccessService, Principal};

impl AccessService {
    pub(super) async fn lookup_attempt(&self, state: &str) -> Result<Option<InstallationAttempt>> {
        let hash = digest(state);
        query(self.store.clone(), move |store| {
            store.installation_attempt(&hash)
        })
        .await
    }

    pub(super) async fn owned_claims(&self, person: i64) -> Result<Vec<InstallationClaim>> {
        query(self.store.clone(), move |store| {
            store.installation_claims(person)
        })
        .await
    }

    pub(super) async fn save_claim(
        &self,
        claim: InstallationClaim,
        attempt: Option<Uuid>,
    ) -> Result<InstallationClaim> {
        query(self.store.clone(), move |store| {
            store.save_installation_claim(&claim, attempt)
        })
        .await
    }

    /// Observe the existing person's memberships, then release this lease before App work.
    pub(super) async fn observe_existing(
        &self,
        profile: &Profile,
        grant: &Grant,
        memberships: &[Membership],
    ) -> Result<()> {
        let person = profile.id;
        let _lease = query(self.store.clone(), move |store| {
            store.lock_github_organization(person)
        })
        .await?;
        let Some(mut user) = query(self.store.clone(), move |store| {
            store.dashboard_user_by_github(person)
        })
        .await?
        else {
            return Ok(());
        };
        user.credentials = self.cipher.seal(user.id, grant.encode()?).await?;
        user.checked_at = self.runtime.now();
        let memberships = memberships.to_vec();
        query(self.store.clone(), move |store| {
            store.save_dashboard_user(&user, &memberships)
        })
        .await
    }

    pub(super) async fn prove_links(
        &self,
        profile: &Profile,
        grant: &Grant,
        memberships: &[Membership],
        setup: Option<(&SetupTarget, Option<&str>)>,
        github: &dyn GitHubApp,
        results: &mut ProofResults,
    ) -> Result<()> {
        if let Some((target, state)) = setup {
            let attempt = match state {
                Some(state) => self.lookup_attempt(state).await?,
                None => None,
            };
            if attempt
                .as_ref()
                .is_some_and(|attempt| attempt.requester_id != profile.id)
            {
                return Err(Error::Forbidden);
            }
            let installation = target.installation_id;
            let person = profile.id;
            let (previous, relevant) = query(self.store.clone(), move |store| {
                let previous = store.claim_for_installation(installation)?;
                let relevant = previous.is_some() || store.has_pending_add(person)?;
                Ok((previous, relevant))
            })
            .await?;
            if attempt.is_some() || relevant {
                let claim = InstallationClaim {
                    id: self.runtime.identifier(),
                    installation_id: target.installation_id,
                    github_id: target.account_id,
                    login: target.login.clone(),
                    requester_id: profile.id,
                    event_seen: false,
                    user_verified: false,
                    direct: attempt.as_ref().is_some_and(|attempt| attempt.direct),
                    workspace_id: None,
                    organization_id: None,
                    verified_at: None,
                };
                self.save_claim_proof(
                    claim,
                    previous.as_ref(),
                    attempt.map(|attempt| attempt.id),
                    results,
                )
                .await?;
            }
        }
        for mut claim in self.owned_claims(profile.id).await? {
            if claim.organization_id.is_some() {
                continue;
            }
            if claim.recovery() == InstallationRecovery::Direct {
                github.resolve_installation_link(claim.id).await.api()?;
                results.record(Some(&claim), claim.id);
                continue;
            }
            let Some(actual) = self
                .identity
                .installation(grant, claim.installation_id)
                .await?
            else {
                claim.user_verified = false;
                claim.verified_at = Some(self.runtime.now());
                self.save_claim(claim, None).await?;
                continue;
            };
            if actual.id != claim.installation_id
                || actual.account.id != claim.github_id
                || actual.account.kind != AccountKind::Organization
                || !memberships
                    .iter()
                    .any(|member| member.github_id == claim.github_id)
            {
                return Err(Error::Forbidden);
            }
            let previous = claim.clone();
            let claim = self
                .save_claim_proof(claim, Some(&previous), None, results)
                .await?;
            if matches!(
                claim.recovery(),
                InstallationRecovery::DirectAwaitingEvent | InstallationRecovery::AddAwaitingEvent
            ) {
                continue;
            }
            let id = claim.id;
            let person = profile.id;
            let observed = memberships.to_vec();
            let current = query(self.store.clone(), move |store| {
                if claim.recovery() == InstallationRecovery::AddAwaitingWorkspace {
                    let eligible = eligible_workspaces(store, person, &observed)?;
                    store.offer_installation_workspaces(id, person, &eligible)?;
                    if eligible.len() == 1 {
                        store.choose_installation_workspace(id, person, eligible[0])?;
                    }
                }
                store.installation_claim(id)
            })
            .await?
            .ok_or(Error::NotFound)?;
            if current.recovery().is_ready() {
                github.resolve_installation_link(id).await.api()?;
            }
        }
        Ok(())
    }

    pub(super) async fn refresh_links(
        &self,
        principal: &Principal,
        github: &dyn GitHubApp,
    ) -> Result<Vec<InstallationClaim>> {
        let person = principal.github_id();
        if principal.browser_only() {
            return self.owned_claims(person).await;
        }
        if let Some(user) = query(self.store.clone(), move |store| {
            store.dashboard_user_by_github(person)
        })
        .await?
        {
            let (grant, memberships) = self.refresh_link_identity(user).await?;
            let profile = self.identity.profile(&grant).await?;
            if profile.id != person {
                return Err(Error::Forbidden);
            }
            self.prove_links(
                &profile,
                &grant,
                &memberships,
                None,
                github,
                &mut ProofResults::default(),
            )
            .await?;
        }
        self.owned_claims(person).await
    }
}

fn eligible_workspaces(
    store: &dyn Store,
    person: i64,
    memberships: &[Membership],
) -> ciproxy_store_interface::error::Result<Vec<Uuid>> {
    let Some(user) = store.dashboard_user_by_github(person)? else {
        return Ok(Vec::new());
    };
    let mut eligible = Vec::new();
    for workspace in store.dashboard_workspaces(user.id)? {
        if store.dashboard_role(workspace.id, person)? != Some(Role::Admin) {
            continue;
        }
        if store
            .organizations(workspace.id)?
            .iter()
            .any(|organization| {
                organization.app_state != AppState::Removed
                    && memberships
                        .iter()
                        .any(|membership| membership.github_id == organization.github_id)
            })
        {
            eligible.push(workspace.id);
        }
    }
    Ok(eligible)
}
