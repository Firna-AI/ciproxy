//! Universal sealed browser sign-in metadata with no GitHub credential.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::dashboard::records::{Session, User};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::wire::access::RemovedOrganization;

use super::authentication::digest;
use super::{AccessService, Principal};

#[derive(Deserialize, Serialize)]
pub(super) struct BrowserState {
    pub(super) user_id: Uuid,
    pub(super) session_id: Uuid,
    pub(super) github_id: i64,
    pub(super) login: String,
    pub(super) avatar_url: String,
    pub(super) secret: String,
    pub(super) csrf: String,
    pub(super) expires_at: DateTime<Utc>,
    pub(super) had_access: bool,
    #[serde(default)]
    pub(super) removed_organization: Option<RemovedOrganization>,
}

impl AccessService {
    pub(super) async fn browser_cookie(&self, browser: &BrowserState) -> Result<SecretString> {
        let value = serde_json::to_string(browser).internal()?;
        let sealed = self.cipher.seal(browser.session_id, value.into()).await?;
        let cookie = format!(
            "v1.{}.{}",
            browser.session_id,
            URL_SAFE_NO_PAD.encode(sealed)
        );
        if cookie.len() > 3800 {
            return Err(Error::Unavailable);
        }
        Ok(cookie.into())
    }

    pub(super) async fn open_browser(&self, cookie: &str) -> Result<BrowserState> {
        let mut pieces = cookie.splitn(3, '.');
        if pieces.next() != Some("v1") {
            return Err(Error::Unauthorized);
        }
        let Some(id) = pieces.next().and_then(|piece| Uuid::parse_str(piece).ok()) else {
            return Err(Error::Unauthorized);
        };
        let Some(encoded) = pieces.next() else {
            return Err(Error::Unauthorized);
        };
        let bytes = match URL_SAFE_NO_PAD.decode(encoded) {
            Ok(value) => value,
            Err(_) => return Err(Error::Unauthorized),
        };
        let opened = match self.cipher.open(id, bytes).await {
            Ok(value) => value,
            Err(_) => return Err(Error::Unauthorized),
        };
        let browser: BrowserState = match serde_json::from_str(opened.expose_secret()) {
            Ok(value) => value,
            Err(_) => return Err(Error::Unauthorized),
        };
        if id != browser.session_id || self.runtime.now() >= browser.expires_at {
            return Err(Error::Unauthorized);
        }
        Ok(browser)
    }
}

impl BrowserState {
    pub(super) fn principal(self, user: Option<User>) -> Principal {
        let pending = user.is_none();
        let session = Session {
            id: self.session_id,
            user_id: self.user_id,
            digest: digest(&self.secret),
            csrf_digest: digest(&self.csrf),
            expires_at: self.expires_at,
        };
        Principal {
            user: user.unwrap_or(User {
                id: self.user_id,
                github_id: self.github_id,
                login: self.login,
                avatar_url: self.avatar_url,
                credentials: Vec::new(),
                checked_at: DateTime::<Utc>::MIN_UTC,
            }),
            session,
            pending,
            had_access: self.had_access,
            removed_organization: self.removed_organization,
            csrf: self.csrf,
        }
    }
}
