//! PKCE state consumption and fixed-duration session issuance.

use chrono::Duration;
use ciproxy_github::app::GitHubApp;
use ciproxy_store_interface::dashboard::records::{Session, User};
use secrecy::{ExposeSecret, SecretString};

use crate::error::{Error, Result};
use crate::state::query;
use crate::wire::access::{RemovedOrganization, SetupOutcome};
use ciproxy_store_interface::organizations::AppState;

use super::authentication::digest;
use super::flow::Purpose;
use super::pending::BrowserState;
use super::{AccessService, InstallContext, LoginSuccess};

impl AccessService {
    pub(super) async fn finish(
        &self,
        code: SecretString,
        state: &str,
        cookie: &str,
        github: &dyn GitHubApp,
    ) -> Result<LoginSuccess> {
        let binding = self.consume_binding(state, cookie).await?;
        let hash = digest(state);
        let now = self.runtime.now();
        let state = query(self.store.clone(), move |store| {
            store.consume_login_state(&hash, now)
        })
        .await?
        .ok_or(Error::Unauthorized)?;
        if state.id != binding.id {
            return Err(Error::Unauthorized);
        }
        let verifier = self.cipher.open(state.id, state.verifier).await?;
        let grant = self.identity.exchange(code, verifier).await?;
        let profile = self.identity.profile(&grant).await?;
        let attempt = match &binding.purpose {
            Purpose::Setup {
                install_state: Some(state),
                ..
            } => self.lookup_attempt(state).await?,
            _ => None,
        };
        if attempt
            .as_ref()
            .is_some_and(|attempt| attempt.requester_id != profile.id)
        {
            return Err(Error::Forbidden);
        }
        let memberships = self.identity.memberships(&grant).await?;
        let setup = match &binding.purpose {
            Purpose::SignIn => None,
            Purpose::Setup {
                installation: Some(installation),
                ..
            } => Some(
                self.prove_setup(installation, &grant, &profile, &memberships)
                    .await?,
            ),
            Purpose::Setup {
                installation: None,
                request_only: true,
                ..
            } => {
                if attempt
                    .as_ref()
                    .is_none_or(|attempt| attempt.requester_id != profile.id)
                {
                    return Err(Error::Forbidden);
                }
                Some(SetupOutcome::WaitingApproval)
            }
            _ => return Err(Error::Forbidden),
        };
        let install_context = if let Some(attempt) = &attempt {
            if matches!(setup.as_ref(), Some(SetupOutcome::OrganizationsOnly { .. }))
                && let Purpose::Setup {
                    installation: Some(personal),
                    ..
                } = &binding.purpose
            {
                let id = attempt.id;
                let person = profile.id;
                let installation = personal.installation_id;
                query(self.store.clone(), move |store| {
                    store.complete_personal_installation_attempt(id, person, installation)
                })
                .await?;
            }
            Some(InstallContext::Attempt(attempt.id))
        } else if matches!(binding.purpose, Purpose::Setup { .. }) {
            let person = profile.id;
            query(self.store.clone(), move |store| {
                store.has_pending_add(person)
            })
            .await?
            .then_some(InstallContext::AmbiguousAdd)
        } else {
            None
        };
        self.observe_existing(&profile, &grant, &memberships)
            .await?;
        let target = match &binding.purpose {
            Purpose::Setup {
                installation: Some(installation),
                install_state,
                ..
            } if matches!(setup.as_ref(), Some(SetupOutcome::Organization { .. })) => {
                Some((installation, install_state.as_deref()))
            }
            _ => None,
        };
        let mut installation_continuation = self
            .prove_links(&profile, &grant, &memberships, target, github)
            .await?;
        installation_continuation.extend(
            self.prove_completed_links(&profile, &grant, &memberships)
                .await?,
        );
        installation_continuation.sort_unstable();
        installation_continuation.dedup();
        let github_id = profile.id;
        let organization_ids = memberships
            .iter()
            .map(|membership| membership.github_id)
            .collect::<Vec<_>>();
        let removed_organization = query(self.store.clone(), move |store| {
            let mut removed = Vec::new();
            for id in organization_ids {
                for organization in store.organizations_by_github_id(id)? {
                    if organization.app_state == AppState::Removed
                        && let Some(delete_after) = organization.delete_after
                        && delete_after > now
                    {
                        removed.push(RemovedOrganization {
                            github_id: id.to_string(),
                            login: organization.login,
                            delete_after,
                        });
                    }
                }
            }
            Ok(removed
                .into_iter()
                .max_by_key(|organization| organization.delete_after))
        })
        .await?;
        let _identity_lease = query(self.store.clone(), move |store| {
            store.lock_github_organization(github_id)
        })
        .await?;
        let previous = query(self.store.clone(), move |store| {
            store.dashboard_user_by_github(github_id)
        })
        .await?;
        let prior = binding
            .previous
            .filter(|browser| browser.github_id == github_id && now < browser.expires_at);
        let install_context = install_context.or_else(|| {
            prior
                .as_ref()
                .and_then(|browser| browser.install_context.clone())
        });
        let setup = setup.or_else(|| prior.as_ref().and_then(|browser| browser.setup.clone()));
        let id = previous.map_or_else(
            || {
                prior
                    .as_ref()
                    .map_or_else(|| self.runtime.identifier(), |browser| browser.user_id)
            },
            |user| user.id,
        );
        let token = self.runtime.secret()?;
        let csrf = self.runtime.secret()?;
        let expires_at = prior
            .as_ref()
            .map_or(now + Duration::days(30), |browser| browser.expires_at);
        let session_id = prior
            .as_ref()
            .map_or_else(|| self.runtime.identifier(), |browser| browser.session_id);
        let session = Session {
            id: session_id,
            user_id: id,
            digest: digest(token.expose_secret()),
            csrf_digest: digest(csrf.expose_secret()),
            expires_at,
        };
        let user = User {
            id,
            github_id,
            login: profile.login.clone(),
            avatar_url: profile.avatar_url.clone(),
            credentials: self.cipher.seal(id, grant.encode()?).await?,
            checked_at: now,
        };
        let mut browser = BrowserState {
            install_context,
            user_id: id,
            session_id,
            github_id,
            login: profile.login,
            avatar_url: profile.avatar_url,
            secret: token.expose_secret().to_owned(),
            csrf: csrf.expose_secret().to_owned(),
            expires_at,
            had_access: prior.is_some_and(|browser| browser.had_access),
            removed_organization,
            setup,
        };
        let cookie = self.browser_cookie(&browser).await?;
        let retained_cookie = if browser.had_access {
            cookie.clone()
        } else {
            browser.had_access = true;
            self.browser_cookie(&browser).await?
        };
        let retained = query(self.store.clone(), move |store| {
            store.establish_dashboard_session(
                &user,
                &memberships,
                &session,
                &installation_continuation,
            )
        })
        .await?;
        Ok(LoginSuccess {
            cookie: if retained { retained_cookie } else { cookie },
            csrf,
            expires_at,
        })
    }
}
