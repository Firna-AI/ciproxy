//! PKCE state consumption and fixed-duration session issuance.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::{LoginState, Session, User};
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::error::{Error, Result};
use crate::state::query;
use crate::wire::access::RemovedOrganization;
use ciproxy_store_interface::organizations::AppState;

use super::authentication::digest;
use super::pending::BrowserState;
use super::{AccessService, LoginAttempt, LoginSuccess};

impl AccessService {
    pub(super) async fn start(&self) -> Result<LoginAttempt> {
        let state = self.runtime.secret()?;
        let verifier = self.runtime.secret()?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.expose_secret().as_bytes()));
        let id = self.runtime.identifier();
        let saved = LoginState {
            id,
            digest: digest(state.expose_secret()),
            verifier: self.cipher.seal(id, verifier).await?,
            expires_at: self.runtime.now() + Duration::minutes(10),
        };
        query(self.store.clone(), move |store| {
            store.save_login_state(&saved)
        })
        .await?;
        Ok(LoginAttempt {
            url: self
                .identity
                .authorization_url(state.expose_secret(), &challenge),
            state,
        })
    }
    pub(super) async fn finish(
        &self,
        code: SecretString,
        state: &str,
        cookie: &str,
        previous: Option<&str>,
    ) -> Result<LoginSuccess> {
        if !bool::from(digest(state).ct_eq(&digest(cookie))) {
            return Err(Error::Unauthorized);
        }
        let hash = digest(state);
        let now = self.runtime.now();
        let state = query(self.store.clone(), move |store| {
            store.consume_login_state(&hash, now)
        })
        .await?
        .ok_or(Error::Unauthorized)?;
        let verifier = self.cipher.open(state.id, state.verifier).await?;
        let grant = self.identity.exchange(code, verifier).await?;
        let profile = self.identity.profile(&grant).await?;
        let memberships = self.identity.memberships(&grant).await?;
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
        let previous_browser = previous;
        let previous = query(self.store.clone(), move |store| {
            store.dashboard_user_by_github(github_id)
        })
        .await?;
        let prior = if let Some(cookie) = previous_browser {
            self.open_browser(cookie)
                .await
                .ok()
                .filter(|browser| browser.github_id == github_id)
        } else {
            None
        };
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
        let retained = query(self.store.clone(), move |store| {
            store.establish_dashboard_session(&user, &memberships, &session)
        })
        .await?;
        let browser = BrowserState {
            user_id: id,
            session_id,
            github_id,
            login: profile.login,
            avatar_url: profile.avatar_url,
            secret: token.expose_secret().to_owned(),
            csrf: csrf.expose_secret().to_owned(),
            expires_at,
            had_access: retained || prior.is_some_and(|browser| browser.had_access),
            removed_organization,
        };
        let cookie = self.browser_cookie(&browser).await?;
        Ok(LoginSuccess {
            cookie,
            csrf,
            expires_at,
        })
    }
}
